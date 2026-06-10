# Data Structure Design

All structures below are logical schemas. Rust structs should derive `Serialize`, `Deserialize`, `Debug`, and `Clone` where appropriate.

## 1. Primitive Types

```rust
pub type DeviceId = String;      // UUID v4
pub type PeerId = String;        // Same as DeviceId in V1
pub type PairingId = String;     // UUID v4
pub type SessionId = String;     // UUID v4
pub type TimestampMs = u64;
```

## 2. Enums

```rust
pub enum OsType {
    Macos,
    Windows,
    Unknown,
}

pub enum CpuArch {
    X86_64,
    Aarch64,
    Unknown,
}

pub enum LayoutDirection {
    Left,
    Right,
    Top,
    Bottom,
}

pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u8),
}

pub enum PermissionKind {
    Accessibility,
    InputMonitoring,
    ScreenRecording,
    WindowsInput,
}

pub enum PermissionState {
    Granted,
    Denied,
    NotDetermined,
    Unsupported,
    Unknown,
}
```

## 3. Local Identity

```rust
pub struct DeviceIdentity {
    pub schema_version: u16,
    pub device_id: DeviceId,
    pub device_name: String,
    pub os: OsType,
    pub arch: CpuArch,
    pub app_version: String,
    pub protocol_version: u16,
    pub public_key: Vec<u8>,
    pub private_key_ref: PrivateKeyRef,
    pub created_at_ms: TimestampMs,
}

pub enum PrivateKeyRef {
    FileEncrypted { path: String },
    Keychain { service: String, account: String },
    WindowsCredential { target: String },
}
```

V1 storage guidance:

- Prefer OS secure storage for private key if available through a mature Rust crate.
- If secure storage increases implementation risk, store a local key file with restrictive permissions and document the limitation for MVP.

## 4. Trusted Peer

```rust
pub struct TrustedPeer {
    pub peer_id: PeerId,
    pub device_name: String,
    pub os: OsType,
    pub arch: CpuArch,
    pub public_key: Vec<u8>,
    pub last_known_addresses: Vec<String>,
    pub last_seen_ms: Option<TimestampMs>,
    pub paired_at_ms: TimestampMs,
    pub app_version_at_pairing: String,
    pub protocol_version: u16,
    pub trust_state: TrustState,
}

pub enum TrustState {
    Trusted,
    Blocked,
    Removed,
}
```

## 5. Discovered Peer

```rust
pub struct DiscoveredPeer {
    pub device_id: DeviceId,
    pub device_name: String,
    pub os: OsType,
    pub arch: CpuArch,
    pub app_version: String,
    pub protocol_version: u16,
    pub addresses: Vec<String>,
    pub service_port: u16,
    pub pairing_available: bool,
    pub last_seen_ms: TimestampMs,
    pub source: DiscoverySource,
}

pub enum DiscoverySource {
    Mdns,
    UdpBroadcast,
    ManualIp,
}
```

## 6. Layout Config

```rust
pub struct LayoutConfig {
    pub peer_id: PeerId,
    pub direction: LayoutDirection,
    pub edge_thickness_px: u32,
    pub corner_guard_px: u32,
    pub enabled: bool,
    pub updated_at_ms: TimestampMs,
}
```

Defaults:

- `direction`: `Right`.
- `edge_thickness_px`: `1`.
- `corner_guard_px`: `32`.
- `enabled`: `true`.

Corner guard:

- Prevents accidental handoff at corners.
- For left/right edges, ignore the top and bottom `corner_guard_px`.
- For top/bottom edges, ignore the left and right `corner_guard_px`.

## 7. Screen Topology

```rust
pub struct ScreenTopology {
    pub virtual_bounds: Rect,
    pub primary_display_id: String,
    pub displays: Vec<DisplayInfo>,
    pub scale_factor: f64,
    pub captured_at_ms: TimestampMs,
}

pub struct DisplayInfo {
    pub display_id: String,
    pub name: Option<String>,
    pub bounds: Rect,
    pub work_area: Rect,
    pub scale_factor: f64,
    pub is_primary: bool,
}

pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub struct Point {
    pub x: f64,
    pub y: f64,
}
```

Coordinate policy:

- Internally use logical pixels for edge detection.
- Convert to platform-specific coordinate systems at capture/injection boundaries.
- Include scale factor to support Retina and Windows DPI.

## 8. Permission Status

```rust
pub struct PermissionStatus {
    pub accessibility: PermissionState,
    pub input_monitoring: PermissionState,
    pub screen_recording: PermissionState,
    pub windows_input: PermissionState,
    pub can_capture_mouse: bool,
    pub can_inject_mouse: bool,
    pub updated_at_ms: TimestampMs,
}
```

macOS mapping:

- `can_capture_mouse` requires Input Monitoring and possibly Accessibility depending on tap location/options.
- `can_inject_mouse` requires Accessibility.
- Screen Recording should be `Unsupported` or `NotDetermined` in V1 UI unless future feature enables it.

Windows mapping:

- `windows_input` is `Granted` if hook and injection smoke tests pass.
- `accessibility`, `input_monitoring`, and `screen_recording` are `Unsupported`.

## 9. Mouse Events

Local capture:

```rust
pub enum LocalMouseEvent {
    Move(MouseMove),
    Down(MouseButtonEvent),
    Up(MouseButtonEvent),
    Wheel(MouseWheelEvent),
}

pub struct MouseMove {
    pub position: Point,
    pub delta: Delta,
    pub timestamp_ms: TimestampMs,
}

pub struct Delta {
    pub dx: f64,
    pub dy: f64,
}

pub struct MouseButtonEvent {
    pub button: MouseButton,
    pub position: Point,
    pub timestamp_ms: TimestampMs,
}

pub struct MouseWheelEvent {
    pub delta_x: f64,
    pub delta_y: f64,
    pub position: Point,
    pub timestamp_ms: TimestampMs,
}
```

Remote wire event:

```rust
pub enum RemoteMouseEvent {
    MoveDelta {
        dx: f32,
        dy: f32,
        seq: u32,
        timestamp_ms: TimestampMs,
    },
    MoveAbsolute {
        x: f32,
        y: f32,
        seq: u32,
        timestamp_ms: TimestampMs,
    },
    Down {
        button: MouseButton,
        seq: u32,
        timestamp_ms: TimestampMs,
    },
    Up {
        button: MouseButton,
        seq: u32,
        timestamp_ms: TimestampMs,
    },
    Wheel {
        dx: f32,
        dy: f32,
        seq: u32,
        timestamp_ms: TimestampMs,
    },
}
```

V1 recommendation:

- Use `MoveDelta` for active remote control.
- Use `MoveAbsolute` only on control enter to place the remote pointer at the entry edge.

## 10. Session State

```rust
pub struct SessionSnapshot {
    pub session_id: Option<SessionId>,
    pub peer_id: Option<PeerId>,
    pub state: SessionState,
    pub control_owner: ControlOwner,
    pub local_pointer: Option<Point>,
    pub remote_pointer: Option<Point>,
    pub last_heartbeat_rtt_ms: Option<u32>,
    pub connected_since_ms: Option<TimestampMs>,
    pub updated_at_ms: TimestampMs,
}

pub enum ControlOwner {
    Local,
    Remote,
    None,
}

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
    Error { code: String, message: String },
}
```

## 11. App Config

```rust
pub struct AppConfig {
    pub schema_version: u16,
    pub local_device_name_override: Option<String>,
    pub network: NetworkConfig,
    pub discovery: DiscoveryConfig,
    pub layouts: Vec<LayoutConfig>,
    pub trusted_peers: Vec<TrustedPeer>,
    pub ui: UiConfig,
    pub updated_at_ms: TimestampMs,
}

pub struct NetworkConfig {
    pub listen_port: u16,
    pub connect_timeout_ms: u64,
    pub heartbeat_interval_ms: u64,
    pub heartbeat_timeout_ms: u64,
    pub reconnect_min_delay_ms: u64,
    pub reconnect_max_delay_ms: u64,
}

pub struct DiscoveryConfig {
    pub mdns_enabled: bool,
    pub udp_broadcast_enabled: bool,
    pub udp_port: u16,
    pub announce_interval_ms: u64,
    pub stale_after_ms: u64,
}

pub struct UiConfig {
    pub start_minimized: bool,
    pub show_diagnostics: bool,
    pub last_selected_peer_id: Option<PeerId>,
}
```

Default network config:

```json
{
  "listen_port": 42424,
  "connect_timeout_ms": 3000,
  "heartbeat_interval_ms": 1000,
  "heartbeat_timeout_ms": 3000,
  "reconnect_min_delay_ms": 500,
  "reconnect_max_delay_ms": 10000
}
```

Default discovery config:

```json
{
  "mdns_enabled": true,
  "udp_broadcast_enabled": true,
  "udp_port": 42425,
  "announce_interval_ms": 1500,
  "stale_after_ms": 10000
}
```

## 12. UI View Models

```ts
export interface UiDevice {
  deviceId: string;
  name: string;
  os: "macos" | "windows" | "unknown";
  arch: "x86_64" | "aarch64" | "unknown";
  appVersion: string;
  protocolVersion: number;
  addressLabel: string;
  status: "available" | "paired" | "connected" | "stale";
  lastSeenLabel: string;
}

export interface UiLayoutConfig {
  peerId: string;
  direction: "left" | "right" | "top" | "bottom";
  enabled: boolean;
}

export interface UiAppStatus {
  localDevice: UiDevice;
  permission: UiPermissionStatus;
  session: UiSessionStatus;
  discoveredDevices: UiDevice[];
}
```

