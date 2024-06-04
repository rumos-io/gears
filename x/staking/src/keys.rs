use crate::consts::{
    error::TIMESTAMP_NANOS_EXPECT,
    keeper::{HISTORICAL_INFO_KEY, VALIDATORS_QUEUE_KEY},
};
use chrono::Utc;
use gears::{
    error::AppError,
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

pub fn historical_info_key(height: u64) -> Vec<u8> {
    let mut res = Vec::with_capacity(9);
    res.extend_from_slice(&HISTORICAL_INFO_KEY);
    res.extend_from_slice(&height.to_le_bytes());
    res
}

pub(super) fn validator_queue_key(end_time: chrono::DateTime<Utc>, end_height: u64) -> Vec<u8> {
    let height_bz = end_height.to_ne_bytes();
    let time_bz = end_time
        .timestamp_nanos_opt()
        .expect(TIMESTAMP_NANOS_EXPECT)
        .to_ne_bytes();

    let mut bz = VALIDATORS_QUEUE_KEY.to_vec();
    bz.extend_from_slice(&(time_bz.len() as u64).to_ne_bytes());
    bz.extend_from_slice(&time_bz);
    bz.extend_from_slice(&height_bz);
    bz
}

pub(super) fn parse_validator_queue_key(
    key: &[u8],
) -> anyhow::Result<(chrono::DateTime<Utc>, u64)> {
    let prefix_len = VALIDATORS_QUEUE_KEY.len();
    if key[..prefix_len] != VALIDATORS_QUEUE_KEY {
        return Err(
            AppError::Custom("Invalid validators queue key. Invalid prefix.".into()).into(),
        );
    }
    let time_len = u64::from_ne_bytes(key[prefix_len..prefix_len + 8].try_into()?);
    let time = chrono::DateTime::from_timestamp_nanos(i64::from_ne_bytes(
        key[prefix_len + 8..prefix_len + 8 + time_len as usize].try_into()?,
    ));
    let height = u64::from_ne_bytes(key[prefix_len + 8 + time_len as usize..].try_into()?);
    Ok((time, height))
}

pub(super) fn unbonding_delegation_time_key(time: chrono::DateTime<Utc>) -> [u8; 8] {
    time.timestamp_nanos_opt()
        .expect(TIMESTAMP_NANOS_EXPECT)
        .to_ne_bytes()
}
