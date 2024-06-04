use gears::types::address::{AccAddress, ValAddress};

/// Converts a type to length prefixed key.
pub fn length_prefixed_bytes_key<T: Into<Vec<u8>>>(addr: T) -> Vec<u8> {
    let bytes = addr.into();
    let mut bytes_prefix = bytes.len().to_le_bytes().to_vec();
    bytes_prefix.extend_from_slice(&bytes);
    bytes_prefix
}

/// Create a key from validator and delegator address.
pub fn length_prefixed_val_del_addrs_key(
    prefix_addr: &ValAddress,
    postfix_addr: &AccAddress,
) -> Vec<u8> {
    let mut prefix = length_prefixed_bytes_key(prefix_addr.clone());
    let postfix = length_prefixed_bytes_key(postfix_addr.clone());
    prefix.extend_from_slice(&postfix);
    prefix
}
