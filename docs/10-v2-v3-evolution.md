# V2/V3 Evolution Plan

## 1. V1 Baseline

V1 delivers:

- macOS and Windows clients.
- LAN discovery.
- Pairing code and IP direct connect.
- Trusted peer persistence.
- Two-device screen layout.
- Mouse move/down/up/wheel sharing.
- TCP realtime transport.
- Heartbeat and reconnect.
- Local-only logs and config.

V1 deliberately excludes:

- Keyboard sharing.
- Clipboard sync.
- File transfer.
- Multi-device layout beyond one peer.
- Accounts.
- Public relay.
- NAT traversal.
- Remote screen streaming.

## 2. V2 Candidate Features

### 2.1 Keyboard Sharing

Value:

- Makes the product feel closer to a complete software KVM.

Technical additions:

- Capture key down/up.
- Preserve modifier state.
- Handle IME and international keyboard layouts.
- Secure attention sequences remain unsupported.

Risks:

- macOS and Windows keyboard layout differences.
- Security perception increases because key capture is sensitive.
- Permissions become more sensitive and must be explained clearly.

### 2.2 Clipboard Sync

Value:

- High user productivity gain.

Scope:

- Text clipboard first.
- Images/files later.

Technical additions:

- Clipboard watcher.
- Deduplication and loop prevention.
- Size limits.
- User toggle.

Security:

- Clipboard can contain secrets.
- Add clear privacy copy and disable-by-default option for early beta.

### 2.3 Multi-Device Layout

Value:

- Supports three or more computers.

Technical additions:

- Layout graph instead of single peer direction.
- Peer routing.
- Conflict handling when multiple peers share an edge.

Data model shift:

```rust
pub struct LayoutGraph {
    pub nodes: Vec<DeviceNode>,
    pub edges: Vec<LayoutEdge>,
}
```

### 2.4 QUIC Transport

Value:

- Better future support for multiple streams, unreliable datagrams, and transport migration.

Use cases:

- Separate control stream from telemetry.
- Optional unreliable high-rate pointer deltas.
- Better behavior across network changes.

Recommendation:

- Move only after V1 profiling identifies TCP limitations or additional stream types justify it.

### 2.5 System Tray First UX

Value:

- Product behaves like utility software.

Technical additions:

- Tray status.
- Quick enable/disable.
- Connected peer menu.
- Emergency disconnect.

### 2.6 Auto-Update

Value:

- Keeps paired clients protocol-compatible.

Technical additions:

- Tauri updater.
- Code signing.
- Release channels.
- Protocol compatibility warnings.

## 3. V3 Candidate Features

### 3.1 Account System

Value:

- Easier device management across networks.

Technical additions:

- Auth service.
- Device registry.
- Revocation.
- Recovery.

Do not add until:

- Local MVP retention is proven.
- Security model is reviewed.
- Update/signing pipeline is stable.

### 3.2 Public Relay And NAT Traversal

Value:

- Works outside same LAN.

Options:

- Relay server.
- STUN/TURN-style traversal.
- QUIC-based connectivity.

Risks:

- Latency may exceed mouse-control expectations.
- Security and abuse prevention become central.
- Operating cost appears.

### 3.3 File Transfer

Value:

- Complements cross-device workflow.

Technical additions:

- Transfer protocol.
- Progress UI.
- Conflict handling.
- Malware scanning considerations.
- Permission and destination prompts.

### 3.4 Remote Screen Preview

Value:

- Helps orientation and setup.

Technical additions:

- Screen capture.
- Encoding.
- Screen Recording permission on macOS.
- More CPU/GPU load.

Recommendation:

- Keep separate from mouse-control hot path.

### 3.5 Policy And Enterprise Controls

Value:

- Makes product acceptable in managed environments.

Technical additions:

- Admin policy.
- Allow/block device lists.
- Logging controls.
- Signed configuration.
- MDM deployment support.

## 4. Architecture Evolution

### 4.1 Extract Core Daemon

Why:

- Utility should continue working when settings UI is closed.
- Easier privilege separation.
- Cleaner auto-start behavior.

Future process model:

```text
Tauri UI <-> Local IPC <-> Core daemon/service <-> Network/input
```

macOS:

- LaunchAgent for user session.

Windows:

- User-level background process first.
- Optional service/helper only if elevated app control becomes required.

### 4.2 Plugin-Like Platform Layer

Why:

- Input APIs are OS-specific and risky.
- Isolate platform changes.

Shape:

```rust
trait PlatformBackend {
    fn permissions(&self) -> PermissionStatus;
    fn capture(&self) -> Result<CaptureHandle>;
    fn inject(&self, event: RemoteMouseEvent) -> Result<()>;
    fn screen_topology(&self) -> Result<ScreenTopology>;
}
```

### 4.3 Protocol Compatibility

V2+ rules:

- Add feature negotiation in `Hello`.
- Keep major/minor protocol version.
- Unknown optional messages can be ignored.
- Unknown required features reject connection with clear error.

Feature flags:

```text
mouse.v1
keyboard.v1
clipboard.text.v1
layout.graph.v1
transport.quic.v1
```

## 5. Security Evolution

V1:

- Pairing confirmation.
- Trusted peer identity.
- Local-only operation.

V2:

- Mandatory encrypted/authenticated transport if not completed in V1.
- Key rotation.
- Peer revocation.
- Trust reset UI.

V3:

- External security review.
- Signed update pipeline.
- Account-backed device trust.
- Audit logs for enterprise mode.

## 6. Product Evolution Strategy

Recommended sequence:

1. Stabilize mouse sharing on LAN.
2. Add tray controls and emergency UX.
3. Add keyboard sharing.
4. Add clipboard text sync.
5. Add multi-device layout.
6. Add auto-update and signing.
7. Consider account/relay only after local workflow has strong retention.

Reasoning:

- Mouse reliability is the trust foundation.
- Keyboard and clipboard dramatically increase usefulness but also increase privacy/security sensitivity.
- Public networking should come only after local protocol and trust model are mature.

