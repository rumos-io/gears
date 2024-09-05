use gears::{
    core::Protobuf,
    derive::AppMessage,
    types::address::{AccAddress, AddressError, ValAddress},
};
use prost::Message;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct MsgUnjailRaw {
    #[prost(bytes)]
    pub validator_address: Vec<u8>,
    #[prost(bytes)]
    pub from_address: Vec<u8>,
}

impl From<MsgUnjail> for MsgUnjailRaw {
    fn from(value: MsgUnjail) -> Self {
        Self {
            validator_address: value.validator_address.into(),
            from_address: value.from_address.into(),
        }
    }
}

/// MsgUnjail creates a new MsgUnjail instance
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, AppMessage)]
#[msg(url = "/cosmos.slashing.v1beta1.Unjail")]
pub struct MsgUnjail {
    pub validator_address: ValAddress,
    #[msg(signer)]
    pub from_address: AccAddress,
}

impl TryFrom<MsgUnjailRaw> for MsgUnjail {
    type Error = AddressError;

    fn try_from(value: MsgUnjailRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            validator_address: ValAddress::try_from(value.validator_address)?,
            from_address: AccAddress::try_from(value.from_address)?,
        })
    }
}

impl Protobuf<MsgUnjailRaw> for MsgUnjail {}
