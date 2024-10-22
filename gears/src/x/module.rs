use crate::types::address::AccAddress;

/// For declaring modules on app level.
pub trait Module: std::fmt::Debug + Clone + Send + Sync + 'static {
    /// Get module name.
    fn name(&self) -> String;
    /// Get module address.
    fn address(&self) -> AccAddress {
        use sha2::Digest;
        let hash = sha2::Sha256::digest(self.name());
        // sdk behavior. It gets slice of first 20 bytes from sha256 hash
        let addr_bytes = &hash[..20];

        AccAddress::try_from(addr_bytes.to_vec())
            .expect("vector of 20 bytes can't produce error because 0 < 20 < MAX_ADDR_LEN")
    }
    /// Module permissions. Default value is empty list.
    fn permissions(&self) -> Vec<String> {
        Vec::new()
    }
}
