use gears::{
    core::errors::CoreError,
    error::IBC_ENCODE_UNWRAP,
    tendermint::types::{proto::Protobuf, time::Timestamp},
    types::address::ConsAddress,
};
use prost::Message;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Deserialize, Serialize, Message)]
pub struct ValidatorSigningInfoRaw {
    #[prost(bytes)]
    pub address: Vec<u8>,
    #[prost(uint32)]
    pub start_height: u32,
    #[prost(uint32)]
    pub index_offset: u32,
    #[prost(bytes)]
    pub jailed_until: Vec<u8>,
    #[prost(bool)]
    pub tombstoned: bool,
    #[prost(uint32)]
    pub missed_blocks_counter: u32,
}

impl From<ValidatorSigningInfo> for ValidatorSigningInfoRaw {
    fn from(src: ValidatorSigningInfo) -> Self {
        Self {
            address: src.address.as_ref().to_vec(),
            start_height: src.start_height,
            index_offset: src.index_offset,
            jailed_until: src.jailed_until.encode_vec().expect(IBC_ENCODE_UNWRAP),
            tombstoned: src.tombstoned,
            missed_blocks_counter: src.missed_blocks_counter,
        }
    }
}

/// ValidatorSigningInfo defines a validator's signing info for monitoring their
/// liveness activity.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ValidatorSigningInfo {
    pub address: ConsAddress,
    /// Height at which validator was first a candidate OR was unjailed
    pub start_height: u32,
    /// Index which is incremented each time the validator was a bonded
    /// in a block and may have signed a precommit or not. This in conjunction with the
    /// `signed_blocks_window` param determines the index in the `missed_blocks_bit_array`.
    pub index_offset: u32,
    /// Timestamp until which the validator is jailed due to liveness downtime.
    pub jailed_until: Timestamp,
    /// Whether or not a validator has been tombstoned (killed out of validator set). It is set
    /// once the validator commits an equivocation or for any other configured misbehiavor.
    pub tombstoned: bool,
    /// A counter kept to avoid unnecessary array reads.
    /// Note that `Sum(missed_blocks_bit_array)` always equals `missed_blocks_counter`.
    pub missed_blocks_counter: u32,
}

impl TryFrom<ValidatorSigningInfoRaw> for ValidatorSigningInfo {
    type Error = CoreError;

    fn try_from(src: ValidatorSigningInfoRaw) -> Result<Self, Self::Error> {
        Ok(ValidatorSigningInfo {
            address: ConsAddress::try_from(src.address)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            start_height: src.start_height,
            index_offset: src.index_offset,
            jailed_until: Timestamp::decode_vec(&src.jailed_until)
                .map_err(|e| CoreError::DecodeProtobuf(e.to_string()))?,
            tombstoned: src.tombstoned,
            missed_blocks_counter: src.missed_blocks_counter,
        })
    }
}

impl Protobuf<ValidatorSigningInfoRaw> for ValidatorSigningInfo {}

#[derive(Clone, PartialEq, Deserialize, Serialize, Message)]
pub struct SignerInfoRaw {
    #[prost(bytes)]
    pub address: Vec<u8>,
    #[prost(bytes)]
    pub validator_signing_info: Vec<u8>,
}

impl From<SignerInfo> for SignerInfoRaw {
    fn from(src: SignerInfo) -> Self {
        Self {
            address: src.address.as_ref().to_vec(),
            validator_signing_info: src
                .validator_signing_info
                .encode_vec()
                .expect(IBC_ENCODE_UNWRAP),
        }
    }
}

/// SigningInfo stores validator signing info of corresponding address.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SignerInfo {
    pub address: ConsAddress,
    pub validator_signing_info: ValidatorSigningInfo,
}

impl TryFrom<SignerInfoRaw> for SignerInfo {
    type Error = CoreError;

    fn try_from(src: SignerInfoRaw) -> Result<Self, Self::Error> {
        Ok(SignerInfo {
            address: ConsAddress::try_from(src.address)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            validator_signing_info: ValidatorSigningInfo::decode_vec(&src.validator_signing_info)
                .map_err(|e| CoreError::DecodeProtobuf(e.to_string()))?,
        })
    }
}

impl Protobuf<SignerInfoRaw> for SignerInfo {}
