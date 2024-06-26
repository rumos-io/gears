use gears::{tendermint::types::time::Timestamp, types::address::ConsAddress};
use serde::{Deserialize, Serialize};

/// ValidatorSigningInfo defines a validator's signing info for monitoring their
/// liveness activity.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ValidatorSigningInfo {
    pub address: ConsAddress,
    /// Height at which validator was first a candidate OR was unjailed
    pub start_height: i64,
    /// Index which is incremented each time the validator was a bonded
    /// in a block and may have signed a precommit or not. This in conjunction with the
    /// `signed_blocks_window` param determines the index in the `missed_blocks_bit_array`.
    pub index_offset: i64,
    /// Timestamp until which the validator is jailed due to liveness downtime.
    pub jailed_until: Timestamp,
    /// Whether or not a validator has been tombstoned (killed out of validator set). It is set
    /// once the validator commits an equivocation or for any other configured misbehiavor.
    pub tombstoned: bool,
    /// A counter kept to avoid unnecessary array reads.
    /// Note that `Sum(missed_blocks_bit_array)` always equals `missed_blocks_counter`.
    pub missed_blocks_counter: i64,
}

/// SigningInfo stores validator signing info of corresponding address.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SignerInfo {
    pub address: ConsAddress,
    pub validator_signing_info: ValidatorSigningInfo,
}
