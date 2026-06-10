use sha2::{Digest, Sha256};

pub fn generate_pairing_code(
    device_id_a: &str,
    device_id_b: &str,
    nonce_a: &[u8],
    nonce_b: &[u8],
    public_key_a: &[u8],
    public_key_b: &[u8],
) -> String {
    let (left_id, right_id) = if device_id_a <= device_id_b {
        (device_id_a, device_id_b)
    } else {
        (device_id_b, device_id_a)
    };

    let mut hasher = Sha256::new();
    hasher.update(left_id.as_bytes());
    hasher.update(right_id.as_bytes());
    hasher.update(nonce_a);
    hasher.update(nonce_b);
    hasher.update(public_key_a);
    hasher.update(public_key_b);

    let digest = hasher.finalize();
    let digits = u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]]) % 1_000_000;
    format!("{digits:06}")
}

