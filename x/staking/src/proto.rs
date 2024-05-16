use crate::consts::proto::*;
use gears::{
    core::Protobuf,
    error::AppError,
    tendermint::types::{proto::crypto::PublicKey, time::Timestamp},
    types::{
        address::{AccAddress, ValAddress},
        auth::fee::inner::Coin as CoinRaw,
        base::coin::Coin,
        decimal256::{CosmosDecimalProtoString, Decimal256},
        errors::StdError,
        uint::Uint256,
    },
};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// CommissionRates defines the initial commission rates to be used for creating
/// a validator.
#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct CommissionRatesRaw {
    #[prost(string)]
    pub rate: String,
    #[prost(string)]
    pub max_rate: String,
    #[prost(string)]
    pub max_change_rate: String,
}

impl From<CommissionRates> for CommissionRatesRaw {
    fn from(value: CommissionRates) -> Self {
        Self {
            rate: value.rate.to_cosmos_proto_string(),
            max_rate: value.max_rate.to_cosmos_proto_string(),
            max_change_rate: value.max_change_rate.to_cosmos_proto_string(),
        }
    }
}

/// CommissionRates defines the initial commission rates to be used for creating
/// a validator.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommissionRates {
    /// rate is the commission rate charged to delegators, as a fraction.
    pub rate: Decimal256,
    /// max_rate defines the maximum commission rate which validator can ever charge, as a fraction.
    pub max_rate: Decimal256,
    /// max_change_rate defines the maximum daily increase of the validator commission, as a fraction.
    pub max_change_rate: Decimal256,
}

impl TryFrom<CommissionRatesRaw> for CommissionRates {
    type Error = StdError;
    fn try_from(value: CommissionRatesRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            rate: Decimal256::from_cosmos_proto_string(&value.rate)?,
            max_rate: Decimal256::from_cosmos_proto_string(&value.max_rate)?,
            max_change_rate: Decimal256::from_cosmos_proto_string(&value.max_change_rate)?,
        })
    }
}

impl Protobuf<CommissionRatesRaw> for CommissionRates {}

/// Commission defines commission parameters for a given validator.
pub struct Commission {
    /// commission_rates defines the initial commission rates to be used for creating a validator.
    pub commission_rates: CommissionRates,
    /// update_time is the last time the commission rate was changed.
    pub update_time: Timestamp,
}

/// Description defines a validator description.
#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct Description {
    /// moniker defines a human-readable name for the validator.
    #[prost(string)]
    pub moniker: String,
    /// identity defines an optional identity signature (ex. UPort or Keybase).
    #[prost(string)]
    pub identity: String,
    /// website defines an optional website link.
    #[prost(string)]
    pub website: String,
    /// security_contact defines an optional email for security contact.
    #[prost(string)]
    pub security_contact: String,
    /// details define other optional details.
    #[prost(string)]
    pub details: String,
}

impl Protobuf<Description> for Description {}

impl Description {
    pub fn ensure_length(&self) -> Result<(), AppError> {
        if self.moniker.len() > MAX_MONIKER_LENGTH {
            return Err(self.form_ensure_length_err(
                "moniker",
                self.moniker.len(),
                MAX_MONIKER_LENGTH,
            ));
        }
        if self.identity.len() > MAX_IDENTITY_LENGTH {
            return Err(self.form_ensure_length_err(
                "identity",
                self.identity.len(),
                MAX_IDENTITY_LENGTH,
            ));
        }
        if self.website.len() > MAX_WEBSITE_LENGTH {
            return Err(self.form_ensure_length_err(
                "website",
                self.website.len(),
                MAX_WEBSITE_LENGTH,
            ));
        }
        if self.security_contact.len() > MAX_SECURITY_CONTACT_LENGTH {
            return Err(self.form_ensure_length_err(
                "security_contact",
                self.security_contact.len(),
                MAX_SECURITY_CONTACT_LENGTH,
            ));
        }
        if self.details.len() > MAX_DETAILS_LENGTH {
            return Err(self.form_ensure_length_err(
                "details",
                self.details.len(),
                MAX_DETAILS_LENGTH,
            ));
        }
        Ok(())
    }

    fn form_ensure_length_err(&self, name: &str, got: usize, max: usize) -> AppError {
        AppError::InvalidRequest(format!("invalid {name} length; got: {got}, max: {max}"))
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct CreateValidatorRaw {
    #[prost(message, optional)]
    pub description: Option<Description>,
    #[prost(message, optional)]
    pub commission: Option<CommissionRatesRaw>,
    #[prost(string)]
    pub min_self_delegation: String,
    #[prost(string)]
    pub delegator_address: String,
    #[prost(string)]
    pub validator_address: String,
    #[prost(bytes)]
    pub pub_key: Vec<u8>,
    #[prost(message, optional)]
    pub value: Option<CoinRaw>,
}

impl From<CreateValidator> for CreateValidatorRaw {
    fn from(src: CreateValidator) -> Self {
        Self {
            description: Some(src.description),
            commission: Some(src.commission.into()),
            min_self_delegation: src.min_self_delegation.to_string(),
            delegator_address: src.delegator_address.to_string(),
            validator_address: src.validator_address.to_string(),
            // TODO: Consider to implement Protobuf on PublicKey
            pub_key: serde_json::to_vec(&src.pub_key).expect("Expected valid public key that can be converted into vector of bytes using serde_json"),
            value: Some(src.value.into()),
        }
    }
}

/// CreateValidator defines a SDK message for creating a new validator.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateValidator {
    pub description: Description,
    pub commission: CommissionRates,
    pub min_self_delegation: Uint256,
    pub delegator_address: AccAddress,
    pub validator_address: ValAddress,
    pub pub_key: PublicKey,
    pub value: Coin,
}

impl TryFrom<CreateValidatorRaw> for CreateValidator {
    type Error = anyhow::Error;

    fn try_from(src: CreateValidatorRaw) -> Result<Self, Self::Error> {
        Ok(CreateValidator {
            description: src.description.ok_or(AppError::Custom(
                "Value should exists. It's the proto3 rule to have Option<T> instead of T".into(),
            ))?,
            commission: src
                .commission
                .ok_or(AppError::Custom(
                    "Value should exists. It's the proto3 rule to have Option<T> instead of T"
                        .into(),
                ))?
                .try_into()?,
            min_self_delegation: Uint256::from_str(&src.min_self_delegation)?,
            delegator_address: AccAddress::from_bech32(&src.delegator_address)?,
            validator_address: ValAddress::from_bech32(&src.validator_address)?,
            pub_key: serde_json::from_slice(&src.pub_key)?,
            value: src
                .value
                .ok_or(AppError::Custom(
                    "Value should exists. It's the proto3 rule to have Option<T> instead of T"
                        .into(),
                ))?
                .try_into()?,
        })
    }
}

impl Protobuf<CreateValidatorRaw> for CreateValidator {}
