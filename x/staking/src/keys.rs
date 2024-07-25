use crate::consts::keeper::{HISTORICAL_INFO_KEY, UNBONDING_QUEUE_KEY, VALIDATOR_QUEUE_KEY};
use gears::{
    tendermint::types::time::Timestamp,
    types::address::{AccAddress, ValAddress},
};

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

pub fn historical_info_key(height: u32) -> Vec<u8> {
    let mut res = Vec::with_capacity(9);
    res.extend_from_slice(&HISTORICAL_INFO_KEY);
    res.extend_from_slice(&height.to_le_bytes());
    res
}

pub(super) fn validator_queue_key(end_time: &Timestamp, end_height: u32) -> Vec<u8> {
    let height_bz = end_height.to_le_bytes();
    let time_bz = end_time.format_bytes_rounded();

    let mut bz = VALIDATOR_QUEUE_KEY.to_vec();
    bz.extend_from_slice(&(time_bz.len() as u64).to_le_bytes());
    bz.extend_from_slice(&time_bz);
    bz.extend_from_slice(&height_bz);
    bz
}

pub(super) fn parse_validator_queue_key(key: &[u8]) -> anyhow::Result<(Timestamp, u32)> {
    // TODO: there are no checks on index out of bounds
    let prefix_len = VALIDATOR_QUEUE_KEY.len();
    if key[..prefix_len] != VALIDATOR_QUEUE_KEY {
        return Err(anyhow::anyhow!(
            "Invalid validators queue key. Invalid prefix."
        ));
    }
    let time_len = u64::from_le_bytes(key[prefix_len..prefix_len + 8].try_into()?);
    let time_bytes = key[prefix_len + 8..prefix_len + 8 + time_len as usize].to_vec();
    let time = Timestamp::try_from_formatted_bytes(&time_bytes)?;
    let height = u32::from_le_bytes(key[prefix_len + 8 + time_len as usize..].try_into()?);
    Ok((time, height))
}

pub(super) fn unbonding_delegation_time_key(time: &Timestamp) -> Vec<u8> {
    let tbz = time.format_bytes_rounded();
    let mut bz = UNBONDING_QUEUE_KEY.to_vec();
    bz.extend(tbz.iter());
    bz
}
