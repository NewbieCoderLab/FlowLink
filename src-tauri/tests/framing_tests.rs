use assert_matches::assert_matches;
use bytes::{BufMut, BytesMut};
use flowlink_lib::{
    network::framing::{decode_frame, encode_frame, FrameError},
    protocol::{
        frame::{Frame, FrameHeader, HEADER_LEN, MAX_FRAME_LENGTH},
        messages::MessageType,
        version::PROTOCOL_VERSION,
    },
};
use pretty_assertions::assert_eq;

fn test_frame(payload: Vec<u8>) -> Frame {
    Frame {
        header: FrameHeader {
            version: PROTOCOL_VERSION,
            message_type: MessageType::HeartbeatPing as u16,
            flags: 0,
            seq: 42,
        },
        payload,
    }
}

#[test]
fn encode_decode_round_trip_preserves_frame() {
    let frame = test_frame(vec![1, 2, 3, 4, 5]);

    let encoded = encode_frame(&frame).expect("encode");
    let mut buffer = BytesMut::from(encoded.as_slice());
    let decoded = decode_frame(&mut buffer).expect("decode");

    assert_eq!(decoded, frame);
    assert!(buffer.is_empty());
}

#[test]
fn encode_rejects_payloads_above_max_frame_length() {
    let frame = test_frame(vec![0; MAX_FRAME_LENGTH - HEADER_LEN + 1]);

    let result = encode_frame(&frame);

    assert_matches!(result, Err(FrameError::FrameTooLarge));
}

#[test]
fn decode_returns_incomplete_for_truncated_frame() {
    let frame = test_frame(vec![1, 2, 3]);
    let encoded = encode_frame(&frame).expect("encode");
    let mut truncated = BytesMut::from(&encoded[..encoded.len() - 1]);

    let result = decode_frame(&mut truncated);

    assert_matches!(result, Err(FrameError::Incomplete));
}

#[test]
fn decode_rejects_length_smaller_than_header() {
    let mut buffer = BytesMut::new();
    buffer.put_u32((HEADER_LEN - 1) as u32);
    buffer.extend_from_slice(&[0; HEADER_LEN - 1]);

    let result = decode_frame(&mut buffer);

    assert_matches!(result, Err(FrameError::InvalidFrameLength));
}
