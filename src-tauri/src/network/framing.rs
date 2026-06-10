use bytes::{Buf, BufMut, BytesMut};
use thiserror::Error;

use crate::protocol::frame::{Frame, FrameHeader, HEADER_LEN, MAX_FRAME_LENGTH};

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("frame exceeds maximum length")]
    FrameTooLarge,
    #[error("buffer does not contain a full frame yet")]
    Incomplete,
}

pub fn encode_frame(frame: &Frame) -> Result<Vec<u8>, FrameError> {
    let payload_len = frame.payload.len();
    let total_len = HEADER_LEN + payload_len;
    if total_len > MAX_FRAME_LENGTH {
        return Err(FrameError::FrameTooLarge);
    }

    let mut buffer = BytesMut::with_capacity(4 + total_len);
    buffer.put_u32(total_len as u32);
    buffer.put_u16(frame.header.version);
    buffer.put_u16(frame.header.message_type);
    buffer.put_u16(frame.header.flags);
    buffer.put_u32(frame.header.seq);
    buffer.extend_from_slice(&frame.payload);
    Ok(buffer.to_vec())
}

pub fn decode_frame(buffer: &mut BytesMut) -> Result<Frame, FrameError> {
    if buffer.len() < 4 {
        return Err(FrameError::Incomplete);
    }

    let frame_len = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;
    if frame_len > MAX_FRAME_LENGTH {
        return Err(FrameError::FrameTooLarge);
    }
    if buffer.len() < 4 + frame_len {
        return Err(FrameError::Incomplete);
    }

    buffer.advance(4);
    let version = buffer.get_u16();
    let message_type = buffer.get_u16();
    let flags = buffer.get_u16();
    let seq = buffer.get_u32();
    let payload_len = frame_len - HEADER_LEN;
    let payload = buffer.split_to(payload_len).to_vec();

    Ok(Frame {
        header: FrameHeader {
            version,
            message_type,
            flags,
            seq,
        },
        payload,
    })
}

