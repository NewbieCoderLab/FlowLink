use flowlink::pairing::{code::generate_pairing_code, flow::PairingFlow};

#[test]
fn pairing_code_is_stable_for_same_inputs() {
    let code_a = generate_pairing_code("a", "b", &[1, 2], &[3, 4], &[5, 6], &[7, 8]);
    let code_b = generate_pairing_code("b", "a", &[3, 4], &[1, 2], &[7, 8], &[5, 6]);
    assert_eq!(code_a, code_b);
    assert_eq!(code_a.len(), 6);
}

#[test]
fn pairing_flow_expires_after_timeout() {
    let flow = PairingFlow::new();
    assert!(flow.is_expired(flow.expires_at_ms + 1));
}
