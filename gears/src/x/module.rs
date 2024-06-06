use crate::types::address::AccAddress;

/// Marks module key in application.
pub trait ModuleKey:
    std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash + Send + Sync + 'static
{
    /// Get string representation of key.
    fn key(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct Module {
    name: String,
    address: AccAddress,
    permissions: Vec<String>,
}

impl Module {
    /// Create module with a concrete account address.
    // TODO: to keep compatibility with the first implementation added parameter `address`. It can
    // be removed and used logic from sdk
    pub fn new(name: String, address: AccAddress, permissions: Vec<String>) -> Self {
        Self {
            name,
            address,
            permissions,
        }
    }

    /// Get module name.
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// Get module address
    pub fn get_address(&self) -> &AccAddress {
        &self.address
    }
    /// Module permissions. Default value is empty list.
    pub fn get_permissions(&self) -> &Vec<String> {
        &self.permissions
    }
}
