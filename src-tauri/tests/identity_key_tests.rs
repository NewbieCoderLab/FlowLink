use flowlink_lib::identity::keys::{
    generate_device_keypair, keypair_from_private_key, serialize_private_key, serialize_public_key,
    verifying_key_from_public_key, ED25519_PRIVATE_KEY_LEN, ED25519_PUBLIC_KEY_LEN,
};

#[test]
fn generated_device_keypair_has_stable_serialized_public_key() {
    let keypair = generate_device_keypair();
    let public_key = serialize_public_key(&keypair);

    assert_eq!(public_key.len(), ED25519_PUBLIC_KEY_LEN);
    let verifying = verifying_key_from_public_key(&public_key).expect("valid verifying key");
    assert_eq!(verifying.to_bytes().to_vec(), public_key);
}

#[test]
fn private_key_round_trip_recovers_same_public_key() {
    let keypair = generate_device_keypair();
    let private_key = serialize_private_key(&keypair);
    let public_key = serialize_public_key(&keypair);

    assert_eq!(private_key.expose().len(), ED25519_PRIVATE_KEY_LEN);
    let restored = keypair_from_private_key(private_key.expose()).expect("private key restores");

    assert_eq!(serialize_public_key(&restored), public_key);
}
