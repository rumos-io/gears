use crate::types::address::AccAddress;

/// For declaring modules on app level.
pub trait Module: std::fmt::Debug + Clone + Send + Sync + 'static {
    /// Get module name.
    fn get_name(&self) -> String;
    /// Get module address.
    fn get_address(&self) -> AccAddress;
    /// Module permissions. Default value is empty list.
    fn get_permissions(&self) -> Vec<String> {
        vec![]
    }
}
