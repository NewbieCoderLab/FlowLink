use rand::{thread_rng, RngCore};

pub fn generate_public_key_stub() -> Vec<u8> {
    let mut bytes = vec![0_u8; 32];
    thread_rng().fill_bytes(&mut bytes);
    bytes
}
