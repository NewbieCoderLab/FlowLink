use flowlink::pairing::code::generate_pairing_code;

#[test]
fn pairing_code_is_stable_when_device_order_is_swapped() {
    let code_a = generate_pairing_code(
        "device-a", "device-b", b"nonce-a", b"nonce-b", b"key-a", b"key-b",
    );
    let code_b = generate_pairing_code(
        "device-b", "device-a", b"nonce-b", b"nonce-a", b"key-b", b"key-a",
    );

    assert_eq!(code_a, code_b);
}

#[test]
fn pairing_code_changes_when_nonce_changes() {
    let code_a = generate_pairing_code(
        "device-a", "device-b", b"nonce-a", b"nonce-b", b"key-a", b"key-b",
    );
    let code_b = generate_pairing_code(
        "device-a",
        "device-b",
        b"nonce-a",
        b"different",
        b"key-a",
        b"key-b",
    );

    assert_ne!(code_a, code_b);
}

#[test]
fn pairing_code_is_always_six_decimal_digits() {
    let code = generate_pairing_code(
        "device-a", "device-b", b"nonce-a", b"nonce-b", b"key-a", b"key-b",
    );

    assert_eq!(code.len(), 6);
    assert!(code.chars().all(|ch| ch.is_ascii_digit()));
}
