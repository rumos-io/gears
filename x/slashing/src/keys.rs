use crate::keeper::*;
use gears::types::address::ConsAddress;

pub(crate) fn addr_pubkey_relation_key(addr: ConsAddress) -> Vec<u8> {
    let key = ADDR_PUBKEY_RELATION_KEY_PREFIX.to_vec();
    let postfix = must_length_prefixed(addr);
    [key, postfix].concat()
}

pub(crate) fn validator_signing_info_key(addr: ConsAddress) -> Vec<u8> {
    let key = VALIDATOR_SIGNING_INFO_KEY_PREFIX.to_vec();
    let postfix = must_length_prefixed(addr);
    [key, postfix].concat()
}

pub(crate) fn validator_missed_block_bit_array_key(addr: ConsAddress, index: u32) -> Vec<u8> {
    let key = VALIDATOR_MISSED_BLOCK_BIT_ARRAY_KEY_PREFIX.to_vec();
    // TODO: maybe need conversion to i64 to have 8 bytes
    let index_bytes = index.to_le_bytes().to_vec();
    let postfix = must_length_prefixed(addr);
    [key, postfix, index_bytes].concat()
}

fn must_length_prefixed(addr: ConsAddress) -> Vec<u8> {
    [vec![addr.len()], addr.as_ref().to_vec()].concat()
}
