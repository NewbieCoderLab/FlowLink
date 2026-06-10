use serde::{Deserialize, Serialize};

pub const HEADER_LEN: usize = 10;
pub const MAX_FRAME_LENGTH: usize = 65_536;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrameHeader {
    pub version: u16,
    pub message_type: u16,
    pub flags: u16,
    pub seq: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Frame {
    pub header: FrameHeader,
    pub payload: Vec<u8>,
}

