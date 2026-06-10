# Module Breakdown

## 1. Module Map

```text
src-tauri/src/
  main.rs
  app/
  config/
  discovery/
  identity/
  input/
  network/
  pairing/
  protocol/
  session/
  storage/
  telemetry/
  ui_api/
```

Frontend:

```text
src/
  app/
  components/
  hooks/
  lib/
  routes/
  styles/
```

## 2. `app`

Responsibilities:

- Initialize runtime.
- Wire modules together.
- Own global app handle and event emitter.
- Start/stop background services.
- Handle graceful shutdown.

Key types:

```rust
pub struct AppContext {
    pub local_identity: DeviceIdentity,
    pub config: Arc<RwLock<AppConfig>>,
    pub session: SessionController,
    pub discovery: DiscoveryService,
    pub pairing: PairingService,
    pub network: NetworkManager,
    pub input: PlatformInputManager,
}
```

Acceptance:

- App can start with missing permissions.
- App can shut down without leaving hooks or sockets open.
- Background services can restart after recoverable error.

## 3. `identity`

Responsibilities:

- Generate stable `device_id` on first run.
- Store local device name, OS, arch, and app version.
- Generate or load local cryptographic identity.

Key types:

```rust
pub struct DeviceIdentity {
    pub device_id: DeviceId,
    pub device_name: String,
    pub os: OsType,
    pub arch: CpuArch,
    pub app_version: String,
    pub protocol_version: u16,
    pub public_key: Vec<u8>,
}
```

Implementation notes:

- `device_id` should be a UUID v4 persisted locally.
- Device name defaults to OS hostname.
- Let user rename local display name later; not required for V1.

## 4. `config`

Responsibilities:

- Define serializable config structs.
- Validate config.
- Provide defaults.
- Expose update methods used by UI commands.

Acceptance:

- Invalid layout cannot be saved.
- Unknown peers are ignored when loading layout.
- Backward-compatible schema version is present.

## 5. `storage`

Responsibilities:

- Read/write config file.
- Read/write trusted peers.
- Read/write local identity.
- Atomic writes.

Implementation:

- Use Tauri app config directory.
- Use JSON or TOML for human-readable V1 storage.
- Write to temp file then rename.

Acceptance:

- Corrupt config backs up to `*.corrupt.<timestamp>` and app starts with defaults.
- Missing config creates defaults.

## 6. `discovery`

Responsibilities:

- Publish local mDNS service.
- Browse remote services.
- Maintain peer cache.
- Emit UI updates.
- Fallback UDP broadcast if mDNS is unavailable.

Key types:

```rust
pub struct DiscoveredPeer {
    pub device_id: DeviceId,
    pub device_name: String,
    pub os: OsType,
    pub arch: CpuArch,
    pub app_version: String,
    pub protocol_version: u16,
    pub addresses: Vec<SocketAddr>,
    pub pairing_available: bool,
    pub last_seen_ms: u64,
}
```

Acceptance:

- Peer appears within `3s`.
- Peer becomes stale after `10s`.
- Self is filtered by `device_id`.

## 7. `pairing`

Responsibilities:

- Start pairing listener.
- Generate pairing code.
- Handle incoming pairing requests.
- Require local confirmation.
- Persist trusted peer.

Pairing flow:

```text
Device A selects Device B
A opens PairingRequest
B displays confirmation with code
User confirms on B
A displays matching confirmation
User confirms on A
Both store TrustedPeer
Transport upgrades to trusted session
```

Key rules:

- Pairing code expires after `120s`.
- Pairing request includes nonce.
- Code derived from both nonces and public keys.
- Reject repeated stale nonce.

Acceptance:

- Pairing cannot complete if either side rejects.
- Pairing cannot complete after expiry.
- Trusted peer persists across restart.

## 8. `network`

Responsibilities:

- Open outgoing TCP connection.
- Accept incoming TCP connection.
- Authenticate trusted peer.
- Frame encode/decode.
- Heartbeats.
- Reconnect.

Submodules:

- `listener`.
- `connector`.
- `framing`.
- `heartbeat`.
- `reconnect`.

Acceptance:

- `TCP_NODELAY` is enabled.
- Connection drops are detected within `3s`.
- Reconnect uses exponential backoff with jitter.
- Button events are never dropped during normal connected operation.

## 9. `protocol`

Responsibilities:

- Define wire message schema.
- Encode/decode binary frames.
- Version negotiation.
- Validate message size and type.

Rules:

- Max frame size for V1 control channel: `64 KiB`.
- Mouse move frames should be small, typically `< 32 bytes` payload.
- Unknown message type closes connection if major protocol version matches but message is invalid.

Acceptance:

- Fuzz or property tests for frame parser.
- Malformed frame cannot panic.

## 10. `session`

Responsibilities:

- Own connection state.
- Own current control owner.
- Process local edge triggers.
- Process remote control enter/leave.
- Coordinate input capture and injection.

Key types:

```rust
pub enum SessionState {
    Disconnected,
    Discovered,
    Pairing,
    Paired,
    Connecting,
    ConnectedIdle,
    ControllingRemote,
    ControlledByRemote,
    Reconnecting,
    Error(SessionError),
}
```

Acceptance:

- Illegal transitions are rejected and logged.
- UI receives state changes.
- Emergency stop returns to `ConnectedIdle` or `Disconnected`.

## 11. `input`

Responsibilities:

- Provide platform-neutral input API.
- Start/stop mouse capture.
- Get screen topology.
- Detect edge hits.
- Inject remote input.
- Filter self-injected events.

Submodules:

```text
input/
  mod.rs
  edge.rs
  types.rs
  macos.rs
  windows.rs
```

Platform-neutral event types:

```rust
pub enum LocalMouseEvent {
    Move { x: f64, y: f64, dx: f64, dy: f64, ts_ms: u64 },
    Down { button: MouseButton, x: f64, y: f64, ts_ms: u64 },
    Up { button: MouseButton, x: f64, y: f64, ts_ms: u64 },
    Wheel { dx: f64, dy: f64, ts_ms: u64 },
}
```

Acceptance:

- Capture starts only when permission checks pass.
- Injection failure is surfaced to session and UI.
- High-frequency move events use bounded channels.

## 12. `ui_api`

Responsibilities:

- Expose Tauri commands.
- Convert internal errors into UI-safe errors.
- Emit frontend events.

Commands:

```rust
#[tauri::command]
async fn get_app_status() -> UiAppStatus;

#[tauri::command]
async fn list_discovered_devices() -> Vec<UiDiscoveredDevice>;

#[tauri::command]
async fn start_pairing(device_id: String) -> Result<UiPairingSession, UiError>;

#[tauri::command]
async fn confirm_pairing(pairing_id: String) -> Result<(), UiError>;

#[tauri::command]
async fn connect_peer(peer_id: String) -> Result<(), UiError>;

#[tauri::command]
async fn save_layout(layout: UiLayoutConfig) -> Result<(), UiError>;

#[tauri::command]
async fn disconnect() -> Result<(), UiError>;

#[tauri::command]
async fn open_permission_settings(permission: PermissionKind) -> Result<(), UiError>;
```

Frontend events:

```text
device:discovered
device:stale
pairing:request
pairing:updated
session:state
permission:updated
network:metrics
```

## 13. `telemetry`

Responsibilities:

- Structured logs.
- Local diagnostic metrics.
- Optional debug export.

V1:

- No remote analytics.
- No personal data upload.
- Logs are local only.

