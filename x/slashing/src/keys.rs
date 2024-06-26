use crate::keeper::*;
use gears::types::address::ConsAddress;

pub(crate) fn addr_pubkey_relation_key(addr: ConsAddress) -> Vec<u8> {
    let mut key = ADDR_PUBKEY_RELATION_KEY_PREFIX.to_vec();
    let addr_bytes = Vec::from(addr);
    let postfix = must_length_prefixed(&addr_bytes);
    key.extend_from_slice(&postfix);
    key
}

pub(crate) fn validator_signing_info_key(addr: ConsAddress) -> Vec<u8> {
    let mut key = VALIDATOR_SIGNING_INFO_KEY_PREFIX.to_vec();
    let addr_bytes = Vec::from(addr);
    let postfix = must_length_prefixed(&addr_bytes);
    key.extend_from_slice(&postfix);
    key
}

pub(crate) fn validator_missed_block_bit_array_key(addr: ConsAddress, index: i64) -> Vec<u8> {
    let mut key = VALIDATOR_MISSED_BLOCK_BIT_ARRAY_KEY_PREFIX.to_vec();

    let addr_bytes = Vec::from(addr);
    let index_bytes = index.to_le_bytes();
    let postfix = must_length_prefixed(&addr_bytes);

    key.extend_from_slice(&postfix);
    key.extend_from_slice(&index_bytes);
    key
}

fn must_length_prefixed(bytes: &[u8]) -> Vec<u8> {
    // TODO: from gears
    const MAX_ADDR_LEN: u8 = 255;
    let len = bytes.len();
    assert!(0 < len && (len as u8) < MAX_ADDR_LEN);
    let mut prefix = len.to_le_bytes().to_vec();
    prefix.extend_from_slice(bytes);
    prefix
}
