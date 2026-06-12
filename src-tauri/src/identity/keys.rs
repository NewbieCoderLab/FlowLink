use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

pub const ED25519_PRIVATE_KEY_LEN: usize = 32;
pub const ED25519_PUBLIC_KEY_LEN: usize = 32;

#[derive(Debug)]
pub struct DeviceKeypair {
    pub signing: SigningKey,
    pub verifying: VerifyingKey,
}

#[derive(Debug, Error)]
pub enum KeyError {
    #[error("invalid Ed25519 private key length: expected 32 bytes, got {0}")]
    InvalidPrivateKeyLength(usize),
    #[error("invalid Ed25519 public key: {0}")]
    InvalidPublicKey(String),
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct PrivateKeyBytes([u8; ED25519_PRIVATE_KEY_LEN]);

impl PrivateKeyBytes {
    pub fn from_slice(value: &[u8]) -> Result<Self, KeyError> {
        let bytes: [u8; ED25519_PRIVATE_KEY_LEN] = value
            .try_into()
            .map_err(|_| KeyError::InvalidPrivateKeyLength(value.len()))?;
        Ok(Self(bytes))
    }

    pub fn expose(&self) -> &[u8; ED25519_PRIVATE_KEY_LEN] {
        &self.0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

pub fn generate_device_keypair() -> DeviceKeypair {
    let signing = SigningKey::generate(&mut OsRng);
    let verifying = signing.verifying_key();
    DeviceKeypair { signing, verifying }
}

pub fn keypair_from_private_key(private_key: &[u8]) -> Result<DeviceKeypair, KeyError> {
    let private_key = PrivateKeyBytes::from_slice(private_key)?;
    let signing = SigningKey::from_bytes(private_key.expose());
    let verifying = signing.verifying_key();
    Ok(DeviceKeypair { signing, verifying })
}

pub fn serialize_private_key(keypair: &DeviceKeypair) -> PrivateKeyBytes {
    PrivateKeyBytes(keypair.signing.to_bytes())
}

pub fn serialize_public_key(keypair: &DeviceKeypair) -> Vec<u8> {
    keypair.verifying.to_bytes().to_vec()
}

pub fn verifying_key_from_public_key(public_key: &[u8]) -> Result<VerifyingKey, KeyError> {
    let bytes: [u8; ED25519_PUBLIC_KEY_LEN] = public_key.try_into().map_err(|_| {
        KeyError::InvalidPublicKey(format!("expected 32 bytes, got {}", public_key.len()))
    })?;
    VerifyingKey::from_bytes(&bytes).map_err(|err| KeyError::InvalidPublicKey(err.to_string()))
}
