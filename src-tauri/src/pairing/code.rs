use sha2::{Digest, Sha256};

pub fn generate_pairing_code(
    device_id_a: &str,
    device_id_b: &str,
    nonce_a: &[u8],
    nonce_b: &[u8],
    public_key_a: &[u8],
    public_key_b: &[u8],
) -> String {
    let (left, right) = if device_id_a <= device_id_b {
        (
            (device_id_a, nonce_a, public_key_a),
            (device_id_b, nonce_b, public_key_b),
        )
    } else {
        (
            (device_id_b, nonce_b, public_key_b),
            (device_id_a, nonce_a, public_key_a),
        )
    };

    let mut hasher = Sha256::new();
    hash_field(&mut hasher, b"left_device_id", left.0.as_bytes());
    hash_field(&mut hasher, b"left_nonce", left.1);
    hash_field(&mut hasher, b"left_public_key", left.2);
    hash_field(&mut hasher, b"right_device_id", right.0.as_bytes());
    hash_field(&mut hasher, b"right_nonce", right.1);
    hash_field(&mut hasher, b"right_public_key", right.2);

    let digest = hasher.finalize();
    let digits = u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]]) % 1_000_000;
    format!("{digits:06}")
}

fn hash_field(hasher: &mut Sha256, label: &[u8], value: &[u8]) {
    hasher.update((label.len() as u32).to_be_bytes());
    hasher.update(label);
    hasher.update((value.len() as u32).to_be_bytes());
    hasher.update(value);
}
