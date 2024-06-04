use chrono::Utc;
use gears::{error::AppError, types::address::BaseAddress};

use crate::consts::{
    error::TIMESTAMP_NANOS_EXPECT,
    keeper::{HISTORICAL_INFO_KEY, VALIDATORS_QUEUE_KEY},
};

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
