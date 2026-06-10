# Network Protocol Design

## 1. Protocol Goals

- Low latency for mouse movement.
- Reliable ordered delivery for button and wheel events.
- Simple implementation for V1.
- Versioned and extensible.
- Safe parsing with bounded frame size.
- Works over LAN without public infrastructure.

## 2. Transport Choice

V1 uses **TCP**.

Required socket options:

- Enable `TCP_NODELAY`.
- Use keepalive if available, but rely on application heartbeat for fast failure detection.
- Use bounded read/write buffers.

Rejected for V1:

- WebSocket: unnecessary overhead and browser-centric framing.
- QUIC: promising but too much complexity for MVP.
- UDP custom protocol: requires custom reliability/ordering and adds risk around event correctness.

## 3. Ports

Defaults:

- Control TCP port: `42424`.
- UDP discovery fallback port: `42425`.
- mDNS service type: `_mac22win._tcp.local`.

Port behavior:

- Allow user-configurable port in config file.
- If port is occupied, app should show an actionable error.
- Do not silently pick a random port for V1 because discovery TXT and direct IP flows need predictability.

## 4. Frame Format

Use length-prefixed binary frames:

```text
0                   1                   2                   3
0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+---------------------------------------------------------------+
|                        length u32 BE                          |
+---------------+---------------+-------------------------------+
| version u16 BE | type u16 BE   | flags u16 BE                  |
+---------------+---------------+-------------------------------+
| seq u32 BE                                                     |
+---------------------------------------------------------------+
| payload ...                                                    |
+---------------------------------------------------------------+
```

Length:

- Number of bytes after the `length` field.
- Includes header fields after length plus payload.
- V1 max length: `65536`.

Header after length:

- `version`: protocol version, V1 is `1`.
- `type`: message type.
- `flags`: reserved, set `0` in V1.
- `seq`: monotonically increasing sender sequence number.

Payload encoding:

- V1 recommendation: `bincode` or `postcard` with Serde.
- Alternative: MessagePack if easier to debug.
- Avoid JSON for high-frequency mouse events.

## 5. Message Types

```rust
pub enum MessageType {
    Hello = 1,
    HelloAck = 2,
    PairingRequest = 10,
    PairingResponse = 11,
    PairingConfirm = 12,
    PairingReject = 13,
    SessionStart = 20,
    SessionState = 21,
    ControlEnter = 30,
    ControlLeave = 31,
    MouseMove = 40,
    MouseButton = 41,
    MouseWheel = 42,
    HeartbeatPing = 50,
    HeartbeatPong = 51,
    Error = 90,
    Goodbye = 99,
}
```

## 6. Hello

Purpose:

- Protocol negotiation.
- Identity exchange.
- Trusted peer validation.

Payload:

```rust
pub struct Hello {
    pub device_id: DeviceId,
    pub device_name: String,
    pub os: OsType,
    pub arch: CpuArch,
    pub app_version: String,
    pub protocol_version: u16,
    pub public_key: Vec<u8>,
    pub nonce: Vec<u8>,
    pub supported_features: Vec<String>,
}
```

Rules:

- If `protocol_version` major is unsupported, send `Error(UnsupportedProtocol)` and close.
- If peer is not trusted and not in pairing mode, send `Error(UntrustedPeer)` and close.
- If public key does not match trusted record, send `Error(PeerIdentityMismatch)` and close.

## 7. Pairing Messages

Pairing request:

```rust
pub struct PairingRequest {
    pub pairing_id: PairingId,
    pub from: DeviceIdentityPublic,
    pub nonce: Vec<u8>,
    pub requested_at_ms: TimestampMs,
    pub expires_at_ms: TimestampMs,
}
```

Pairing response:

```rust
pub struct PairingResponse {
    pub pairing_id: PairingId,
    pub from: DeviceIdentityPublic,
    pub nonce: Vec<u8>,
    pub code_fingerprint: String,
    pub accepted_for_confirmation: bool,
}
```

Pairing confirm:

```rust
pub struct PairingConfirm {
    pub pairing_id: PairingId,
    pub confirmed: bool,
    pub confirmed_at_ms: TimestampMs,
}
```

Code generation:

```text
code = first_6_decimal_digits(
  SHA256(
    min(device_id_a, device_id_b) ||
    max(device_id_a, device_id_b) ||
    nonce_a ||
    nonce_b ||
    public_key_a ||
    public_key_b
  )
)
```

Rules:

- Both screens display the same six-digit code.
- Both users must confirm.
- Expired pairing closes the temporary connection.
- Confirmed pairing stores peer identity and public key.

## 8. Session Messages

Session start:

```rust
pub struct SessionStart {
    pub session_id: SessionId,
    pub peer_id: PeerId,
    pub layout: LayoutDirection,
    pub local_screen: ScreenTopologySummary,
    pub started_at_ms: TimestampMs,
}
```

Session state:

```rust
pub struct SessionStateMsg {
    pub session_id: SessionId,
    pub state: SessionStateWire,
    pub control_owner: ControlOwnerWire,
    pub pointer: Option<PointWire>,
    pub sent_at_ms: TimestampMs,
}
```

Control enter:

```rust
pub struct ControlEnter {
    pub session_id: SessionId,
    pub entry_edge: Edge,
    pub initial_position: PointWire,
    pub sent_at_ms: TimestampMs,
}
```

Control leave:

```rust
pub struct ControlLeave {
    pub session_id: SessionId,
    pub reason: ControlLeaveReason,
    pub exit_edge: Option<Edge>,
    pub sent_at_ms: TimestampMs,
}
```

## 9. Mouse Messages

Mouse move:

```rust
pub struct MouseMoveMsg {
    pub session_id: SessionId,
    pub dx: f32,
    pub dy: f32,
    pub absolute_x: Option<f32>,
    pub absolute_y: Option<f32>,
    pub sent_at_ms: TimestampMs,
}
```

Rules:

- Use `dx` and `dy` for normal movement.
- Use `absolute_x` and `absolute_y` only when entering control or resyncing pointer.
- Sender may coalesce consecutive move events if the outbound queue is full.

Mouse button:

```rust
pub struct MouseButtonMsg {
    pub session_id: SessionId,
    pub button: MouseButton,
    pub action: ButtonAction,
    pub sent_at_ms: TimestampMs,
}

pub enum ButtonAction {
    Down,
    Up,
}
```

Rules:

- Never drop button events.
- Preserve ordering relative to previous move events.
- If queue pressure exists, flush/coalesce moves before enqueuing button event.

Mouse wheel:

```rust
pub struct MouseWheelMsg {
    pub session_id: SessionId,
    pub dx: f32,
    pub dy: f32,
    pub sent_at_ms: TimestampMs,
}
```

Rules:

- Wheel events may be coalesced only if they are consecutive and no button event is between them.

## 10. Heartbeat

Heartbeat ping:

```rust
pub struct HeartbeatPing {
    pub session_id: Option<SessionId>,
    pub ping_id: u32,
    pub sent_at_ms: TimestampMs,
}
```

Heartbeat pong:

```rust
pub struct HeartbeatPong {
    pub session_id: Option<SessionId>,
    pub ping_id: u32,
    pub sent_at_ms: TimestampMs,
    pub received_ping_at_ms: TimestampMs,
}
```

Defaults:

- Ping interval: `1000ms`.
- Timeout: `3000ms`.
- Missed heartbeat threshold: 3 pings or timeout window, whichever is reached first.

## 11. Reconnect

Reconnect algorithm:

```text
delay = min(max_delay, min_delay * 2^attempt)
delay = delay + random_jitter(0..250ms)
```

Defaults:

- Min delay: `500ms`.
- Max delay: `10000ms`.
- Reset attempt count after `30s` stable connection.

Reconnect steps:

1. Mark session `Reconnecting`.
2. Stop forwarding local input to remote.
3. Keep paired peer identity.
4. Try last known addresses first.
5. Fall back to discovery cache.
6. On success, run `Hello` validation and `SessionStart`.
7. Return to `ConnectedIdle`.

## 12. Backpressure

Outbound queues:

- High-priority queue: button, wheel, control, heartbeat, session messages.
- Low-priority queue: mouse move.

Rules:

- Low-priority move queue can coalesce.
- High-priority queue must be bounded but should block briefly before failing.
- If high-priority queue cannot send within timeout, treat connection as unhealthy.

Suggested bounds:

- Move queue: `128` entries.
- High-priority queue: `256` entries.

## 13. Error Messages

```rust
pub struct ErrorMsg {
    pub code: ErrorCode,
    pub message: String,
    pub recoverable: bool,
    pub sent_at_ms: TimestampMs,
}

pub enum ErrorCode {
    UnsupportedProtocol,
    UntrustedPeer,
    PeerIdentityMismatch,
    PairingExpired,
    PermissionMissing,
    InvalidSession,
    InvalidFrame,
    Busy,
    Internal,
}
```

Rules:

- `InvalidFrame` closes connection.
- `PermissionMissing` does not remove trust.
- `PeerIdentityMismatch` should mark peer as suspicious and require manual action.

## 14. Protocol Tests

Required tests:

- Encode/decode each message type.
- Reject frames larger than max.
- Reject unknown critical message type.
- Reject invalid protocol version.
- Parser does not panic on truncated frame.
- Button event ordering is preserved.
- Move coalescing never crosses button boundary.

