use gears::{
    tendermint::types::time::timestamp::Timestamp,
    types::address::{AccAddress, ValAddress},
};

use crate::consts::keeper::{
    REDELEGATION_BY_VAL_DST_INDEX_KEY, REDELEGATION_BY_VAL_SRC_INDEX_KEY, REDELEGATION_KEY,
    UNBONDING_DELEGATION_BY_VAL_INDEX_KEY, UNBONDING_DELEGATION_KEY, UNBONDING_QUEUE_KEY,
};

/// Returns a key prefix for indexing a redelegation from a delegator
/// and source validator to a destination validator.
pub fn redelegation_key(
    del_addr: &AccAddress,
    val_src_addr: &ValAddress,
    val_dst_addr: &ValAddress,
) -> Vec<u8> {
    // key is of the form redelegations_key || val_src_addr.len() (1 byte) || val_src_addr || val_dst_addr.len() (1 byte) || val_dst_addr

    let a = redelegations_key(del_addr);
    let b = val_src_addr.prefix_len_bytes();
    let c = val_dst_addr.prefix_len_bytes();

    [a.as_slice(), b.as_slice(), c.as_slice()].concat()
}

/// Returns a key prefix for indexing a redelegation from a delegator address.
fn redelegations_key(del_addr: &AccAddress) -> Vec<u8> {
    [&REDELEGATION_KEY, del_addr.prefix_len_bytes().as_slice()].concat()
}

/// Creates the index-key for a redelegation, stored by source-validator-index
pub fn redelegation_by_val_src_index_key(
    del_addr: &AccAddress,
    val_src_addr: &ValAddress,
    val_dst_addr: &ValAddress,
) -> Vec<u8> {
    // key is of the form redelegations_from_val_src_index_key || del_addr.len() (1 byte) || del_addr || val_dst_addr.len() (1 byte) || val_dst_addr

    let a = redelegations_from_val_src_index_key(val_src_addr);
    let b = del_addr.prefix_len_bytes();
    let c = val_dst_addr.prefix_len_bytes();

    [a.as_slice(), b.as_slice(), c.as_slice()].concat()
}

// Returns a key prefix for indexing a redelegation to
// a source validator.
fn redelegations_from_val_src_index_key(val_src_addr: &ValAddress) -> Vec<u8> {
    [
        &REDELEGATION_BY_VAL_SRC_INDEX_KEY,
        val_src_addr.prefix_len_bytes().as_slice(),
    ]
    .concat()
}

/// Creates the index-key for a redelegation, stored by destination-validator-index
pub fn redelegation_by_val_dst_index_key(
    del_addr: &AccAddress,
    val_src_addr: &ValAddress,
    val_dst_addr: &ValAddress,
) -> Vec<u8> {
    // key is of the form redelegations_by_val_dst_index_key || del_addr.len() (1 byte) || del_addr || val_src_addr.len() (1 byte) || val_src_addr

    let a = redelegations_by_val_dst_index_key(val_dst_addr);
    let b = del_addr.prefix_len_bytes();
    let c = val_src_addr.prefix_len_bytes();

    [a.as_slice(), b.as_slice(), c.as_slice()].concat()
}

/// Returns a key prefix for indexing a redelegation to a
/// destination (target) validator.
pub fn redelegations_by_val_dst_index_key(val_dst_addr: &ValAddress) -> Vec<u8> {
    [
        &REDELEGATION_BY_VAL_DST_INDEX_KEY,
        val_dst_addr.prefix_len_bytes().as_slice(),
    ]
    .concat()
}

/// Creates the key for an unbonding delegation by delegator and validator addr
/// VALUE: staking/UnbondingDelegation
pub fn get_ubd_key(del_addr: &AccAddress, val_addr: &ValAddress) -> Vec<u8> {
    [
        get_ubds_key(del_addr).as_slice(),
        val_addr.prefix_len_bytes().as_slice(),
    ]
    .concat()
}

/// Creates the prefix for all unbonding delegations from a delegator
pub fn get_ubds_key(del_addr: &AccAddress) -> Vec<u8> {
    [
        &UNBONDING_DELEGATION_KEY,
        del_addr.prefix_len_bytes().as_slice(),
    ]
    .concat()
}

/// Creates the index-key for an unbonding delegation, stored by validator-index
/// VALUE: none (key rearrangement used)
pub fn get_ubd_by_val_index_key(del_addr: &AccAddress, val_addr: &ValAddress) -> Vec<u8> {
    [
        get_ubds_by_val_index_key(val_addr).as_slice(),
        del_addr.prefix_len_bytes().as_slice(),
    ]
    .concat()
}

/// Creates the prefix keyspace for the indexes of unbonding delegations for a validator
fn get_ubds_by_val_index_key(val_addr: &ValAddress) -> Vec<u8> {
    [
        &UNBONDING_DELEGATION_BY_VAL_INDEX_KEY,
        val_addr.prefix_len_bytes().as_slice(),
    ]
    .concat()
}

/// Creates the prefix for all unbonding delegations from a delegator
pub fn get_unbonding_delegation_time_key(timestamp: Timestamp) -> Vec<u8> {
    let bz = timestamp.format_bytes_rounded();
    [&UNBONDING_QUEUE_KEY, bz.as_slice()].concat()
}

// // GetUnbondingDelegationTimeKey creates the prefix for all unbonding delegations from a delegator
// func GetUnbondingDelegationTimeKey(timestamp time.Time) []byte {
// 	bz := sdk.FormatTimeBytes(timestamp)
// 	return append(UnbondingQueueKey, bz...)
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redelegation_key() {
        let del_addr =
            AccAddress::from_bech32("cosmos15qzm75pjh0jqsv3u40hzp2vzs2hdp47fkz7j5q").unwrap();
        let val_src_addr =
            ValAddress::from_bech32("cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4")
                .unwrap();
        let val_dst_addr =
            ValAddress::from_bech32("cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4")
                .unwrap();

        let key = redelegation_key(&del_addr, &val_src_addr, &val_dst_addr);
        assert_eq!(
            key,
            vec![
                52, 20, 160, 5, 191, 80, 50, 187, 228, 8, 50, 60, 171, 238, 32, 169, 130, 130, 174,
                208, 215, 201, 20, 129, 58, 194, 42, 97, 73, 22, 85, 226, 120, 106, 224, 209, 39,
                214, 153, 11, 251, 251, 222, 20, 129, 58, 194, 42, 97, 73, 22, 85, 226, 120, 106,
                224, 209, 39, 214, 153, 11, 251, 251, 222
            ]
        );
    }

    #[test]
    fn test_redelegations_key() {
        let del_addr =
            AccAddress::from_bech32("cosmos15qzm75pjh0jqsv3u40hzp2vzs2hdp47fkz7j5q").unwrap();

        let key = redelegations_key(&del_addr);
        assert_eq!(
            key,
            vec![
                52, 20, 160, 5, 191, 80, 50, 187, 228, 8, 50, 60, 171, 238, 32, 169, 130, 130, 174,
                208, 215, 201
            ]
        );
    }

    #[test]
    fn test_redelegation_by_val_src_index_key() {
        let del_addr =
            AccAddress::from_bech32("cosmos15qzm75pjh0jqsv3u40hzp2vzs2hdp47fkz7j5q").unwrap();
        let val_src_addr =
            ValAddress::from_bech32("cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4")
                .unwrap();
        let val_dst_addr =
            ValAddress::from_bech32("cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4")
                .unwrap();

        let key = redelegation_by_val_src_index_key(&del_addr, &val_src_addr, &val_dst_addr);
        assert_eq!(
            key,
            vec![
                53, 20, 129, 58, 194, 42, 97, 73, 22, 85, 226, 120, 106, 224, 209, 39, 214, 153,
                11, 251, 251, 222, 20, 160, 5, 191, 80, 50, 187, 228, 8, 50, 60, 171, 238, 32, 169,
                130, 130, 174, 208, 215, 201, 20, 129, 58, 194, 42, 97, 73, 22, 85, 226, 120, 106,
                224, 209, 39, 214, 153, 11, 251, 251, 222
            ]
        );
    }

    #[test]
    fn test_redelegations_from_val_src_index_key() {
        let val_src_addr =
            ValAddress::from_bech32("cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4")
                .unwrap();

        let key = redelegations_from_val_src_index_key(&val_src_addr);
        assert_eq!(
            key,
            vec![
                53, 20, 129, 58, 194, 42, 97, 73, 22, 85, 226, 120, 106, 224, 209, 39, 214, 153,
                11, 251, 251, 222
            ]
        );
    }

    #[test]
    fn test_redelegation_by_val_dst_index_key() {
        let del_addr =
            AccAddress::from_bech32("cosmos15qzm75pjh0jqsv3u40hzp2vzs2hdp47fkz7j5q").unwrap();
        let val_src_addr =
            ValAddress::from_bech32("cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4")
                .unwrap();
        let val_dst_addr =
            ValAddress::from_bech32("cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4")
                .unwrap();

        let key = redelegation_by_val_dst_index_key(&del_addr, &val_src_addr, &val_dst_addr);
        assert_eq!(
            key,
            vec![
                54, 20, 129, 58, 194, 42, 97, 73, 22, 85, 226, 120, 106, 224, 209, 39, 214, 153,
                11, 251, 251, 222, 20, 160, 5, 191, 80, 50, 187, 228, 8, 50, 60, 171, 238, 32, 169,
                130, 130, 174, 208, 215, 201, 20, 129, 58, 194, 42, 97, 73, 22, 85, 226, 120, 106,
                224, 209, 39, 214, 153, 11, 251, 251, 222
            ]
        );
    }

    #[test]
    fn test_redelegations_by_val_dst_index_key() {
        let val_dst_addr =
            ValAddress::from_bech32("cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4")
                .unwrap();

        let key = redelegations_by_val_dst_index_key(&val_dst_addr);
        assert_eq!(
            key,
            vec![
                54, 20, 129, 58, 194, 42, 97, 73, 22, 85, 226, 120, 106, 224, 209, 39, 214, 153,
                11, 251, 251, 222
            ]
        );
    }
}
