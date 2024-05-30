use gears::types::address::BaseAddress;

pub fn length_prefixed_addr_key<const PREFIX: u8>(addr: &BaseAddress<PREFIX>) -> Vec<u8> {
    let addr_str = addr.to_string();
    let bytes = addr_str.as_bytes();
    let mut bytes_prefix = bytes.len().to_ne_bytes().to_vec();
    bytes_prefix.extend_from_slice(&bytes);
    bytes_prefix
}

pub fn length_prefixed_addr_pair_key<const PREFIX1: u8, const PREFIX2: u8>(
    prefix_addr: &BaseAddress<PREFIX1>,
    postfix_addr: &BaseAddress<PREFIX2>,
) -> Vec<u8> {
    let mut prefix = length_prefixed_addr_key(prefix_addr);
    let postfix = length_prefixed_addr_key(postfix_addr);
    prefix.extend_from_slice(&postfix);
    prefix
}
