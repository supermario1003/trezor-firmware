pub trait CredentialStore {
    fn lookup(ephemeral: &[u8], masked_static: &[u8]) -> Option<(&[u8], &[u8])>; // (privkey, credential_bytes)
    fn store()
}
