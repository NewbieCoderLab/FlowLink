pub trait SecretStore {
    fn load_private_key(&self) -> Result<Vec<u8>, String>;
    fn save_private_key(&self, _key: &[u8]) -> Result<String, String>;
}
