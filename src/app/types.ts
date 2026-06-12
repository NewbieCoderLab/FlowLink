export type OsType = "macos" | "windows" | "unknown";
export type ArchType = "x86_64" | "aarch64" | "unknown";
export type LayoutDirection = "left" | "right" | "top" | "bottom";
export type PermissionState = "granted" | "denied" | "not_determined" | "unsupported" | "unknown";
export type SessionState =
  | "disconnected"
  | "discovered"
  | "pairing"
  | "paired"
  | "connecting"
  | "connected_idle"
  | "controlling_remote"
  | "controlled_by_remote"
  | "reconnecting"
  | "error";

export interface UiDevice {
  deviceId: string;
  name: string;
  os: OsType;
  arch: ArchType;
  appVersion: string;
  protocolVersion: number;
  addressLabel: string;
  status: "available" | "paired" | "connected" | "stale";
  lastSeenLabel: string;
}

export interface UiPermissionStatus {
  accessibility: PermissionState;
  inputMonitoring: PermissionState;
  screenRecording: PermissionState;
  windowsInput: PermissionState;
  windowsIntegrityLevel: "low" | "medium" | "high" | "system" | "unknown" | null;
  canCaptureMouse: boolean;
  canInjectMouse: boolean;
  updatedAtMs: number;
}

export interface UiSessionStatus {
  state: SessionState;
  controlOwner: "local" | "remote" | "none";
  peerName: string | null;
  lastHeartbeatRttMs: number | null;
  connectedSinceMs: number | null;
  updatedAtMs: number;
}

export interface UiLayoutConfig {
  peerId: string;
  direction: LayoutDirection;
  enabled: boolean;
}

export interface UiDiagnostics {
  discoveredPeerCount: number;
  trustedPeerCount: number;
  layoutCount: number;
  configPath: string;
}

export interface UiRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface UiDisplayInfo {
  id: number;
  bounds: UiRect;
  scaleFactor: number;
  isPrimary: boolean;
}

export interface UiScreenTopology {
  displays: UiDisplayInfo[];
  virtualBounds: UiRect;
}

export interface UiAppStatus {
  localDevice: UiDevice;
  permission: UiPermissionStatus;
  session: UiSessionStatus;
  discoveredDevices: UiDevice[];
  savedLayouts: UiLayoutConfig[];
  diagnostics: UiDiagnostics;
}

