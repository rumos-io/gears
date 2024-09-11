use gears::{
    core::{errors::CoreError, Protobuf},
    types::address::{AccAddress, ValAddress},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DvvTriplet {
    pub del_addr: AccAddress,
    pub val_src_addr: ValAddress,
    pub val_dst_addr: ValAddress,
}
impl DvvTriplet {
    pub fn new(del_addr: AccAddress, val_src_addr: ValAddress, val_dst_addr: ValAddress) -> Self {
        Self {
            del_addr,
            val_src_addr,
            val_dst_addr,
        }
    }
}

impl From<DvvTriplet> for inner::DvvTriplet {
    fn from(value: DvvTriplet) -> Self {
        Self {
            delegator_address: value.del_addr.into(),
            validator_src_address: value.val_src_addr.into(),
            validator_dst_address: value.val_dst_addr.into(),
        }
    }
}

impl TryFrom<inner::DvvTriplet> for DvvTriplet {
    type Error = CoreError;

    fn try_from(value: inner::DvvTriplet) -> Result<Self, Self::Error> {
        Ok(Self {
            del_addr: AccAddress::from_bech32(&value.delegator_address)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            val_src_addr: ValAddress::from_bech32(&value.validator_src_address)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            val_dst_addr: ValAddress::from_bech32(&value.validator_dst_address)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DvvTriplets {
    pub triplets: Vec<DvvTriplet>,
}

impl From<Vec<DvvTriplet>> for DvvTriplets {
    fn from(triplets: Vec<DvvTriplet>) -> Self {
        Self { triplets }
    }
}

impl From<DvvTriplets> for inner::DvvTriplets {
    fn from(value: DvvTriplets) -> Self {
        Self {
            triplets: value.triplets.into_iter().map(|x| x.into()).collect(),
        }
    }
}

impl TryFrom<inner::DvvTriplets> for DvvTriplets {
    type Error = CoreError;

    fn try_from(value: inner::DvvTriplets) -> Result<Self, Self::Error> {
        Ok(Self {
            triplets: value
                .triplets
                .into_iter()
                .map(DvvTriplet::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl Protobuf<inner::DvvTriplets> for DvvTriplets {}

mod inner {
    pub use ibc_proto::cosmos::staking::v1beta1::DvPair;
    pub use ibc_proto::cosmos::staking::v1beta1::DvPairs;
    pub use ibc_proto::cosmos::staking::v1beta1::DvvTriplet;
    pub use ibc_proto::cosmos::staking::v1beta1::DvvTriplets;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DvPair {
    pub val_addr: ValAddress,
    pub del_addr: AccAddress,
}
impl DvPair {
    pub fn new(val_addr: ValAddress, del_addr: AccAddress) -> Self {
        Self { val_addr, del_addr }
    }
}

impl From<DvPair> for inner::DvPair {
    fn from(value: DvPair) -> Self {
        Self {
            validator_address: value.val_addr.into(),
            delegator_address: value.del_addr.into(),
        }
    }
}

impl TryFrom<inner::DvPair> for DvPair {
    type Error = CoreError;

    fn try_from(value: inner::DvPair) -> Result<Self, Self::Error> {
        Ok(Self {
            val_addr: ValAddress::from_bech32(&value.validator_address)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            del_addr: AccAddress::from_bech32(&value.delegator_address)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DvPairs {
    pub pairs: Vec<DvPair>,
}

impl From<DvPairs> for inner::DvPairs {
    fn from(value: DvPairs) -> Self {
        Self {
            pairs: value.pairs.into_iter().map(|x| x.into()).collect(),
        }
    }
}

impl TryFrom<inner::DvPairs> for DvPairs {
    type Error = CoreError;

    fn try_from(value: inner::DvPairs) -> Result<Self, Self::Error> {
        Ok(Self {
            pairs: value
                .pairs
                .into_iter()
                .map(DvPair::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl Protobuf<inner::DvPairs> for DvPairs {}
