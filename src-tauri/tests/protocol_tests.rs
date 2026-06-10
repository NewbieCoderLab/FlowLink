use bytes::BytesMut;
use flowlink::{
    network::framing::{decode_frame, encode_frame},
    protocol::{
        frame::{Frame, FrameHeader},
        messages::MessageType,
        version::PROTOCOL_VERSION,
    },
};

#[test]
fn frame_round_trip_preserves_header_and_payload() {
    let frame = Frame {
        header: FrameHeader {
            version: PROTOCOL_VERSION,
            message_type: MessageType::HeartbeatPing as u16,
            flags: 0,
            seq: 7,
        },
        payload: vec![1, 2, 3, 4],
    };

    let encoded = encode_frame(&frame).expect("encode");
    let mut buffer = BytesMut::from(encoded.as_slice());
    let decoded = decode_frame(&mut buffer).expect("decode");

    assert_eq!(decoded, frame);
}

