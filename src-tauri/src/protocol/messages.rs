use serde::{Deserialize, Serialize};

pub type TimestampMs = u64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OsType {
    Macos,
    Windows,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ArchType {
    X86_64,
    Aarch64,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LayoutDirection {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u8),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DiscoverySource {
    Mdns,
    UdpBroadcast,
    ManualIp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[repr(u16)]
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

