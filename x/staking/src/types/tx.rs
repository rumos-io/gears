use crate::consts::proto::*;
use gears::{
    core::{errors::CoreError, Protobuf},
    derive::{AppMessage, Protobuf},
    signing::renderer::value_renderer::ValueRenderer,
    tendermint::types::{proto::crypto::PublicKey, time::timestamp::Timestamp},
    types::{
        address::{AccAddress, ValAddress},
        auth::fee::inner::Coin as CoinRaw,
        base::coin::UnsignedCoin,
        decimal256::{CosmosDecimalProtoString, Decimal256, ONE_DEC},
        errors::StdError,
        uint::Uint256,
    },
};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

mod inner {
    pub use ibc_proto::cosmos::staking::v1beta1::Commission;
    pub use ibc_proto::cosmos::staking::v1beta1::CommissionRates;
    pub use ibc_proto::cosmos::staking::v1beta1::Description;
    pub use ibc_proto::cosmos::staking::v1beta1::MsgCreateValidator;
}

// constant used in flags to indicate that description field should not be updated
pub const DO_NOT_MODIFY_DESCRIPTION: &str = "[do-not-modify]";

/// CommissionRates defines the initial commission rates to be used for creating
/// a validator.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommissionRates {
    /// rate is the commission rate charged to delegators, as a fraction.
    rate: Decimal256,
    /// max_rate defines the maximum commission rate which validator can ever charge, as a fraction.
    max_rate: Decimal256,
    /// max_change_rate defines the maximum daily increase of the validator commission, as a fraction.
    max_change_rate: Decimal256,
}

impl CommissionRates {
    pub fn new(
        rate: Decimal256,
        max_rate: Decimal256,
        max_change_rate: Decimal256,
    ) -> Result<CommissionRates, anyhow::Error> {
        CommissionRates::validate(rate, max_rate, max_change_rate)?;
        Ok(CommissionRates {
            rate,
            max_rate,
            max_change_rate,
        })
    }

    pub fn rate(&self) -> Decimal256 {
        self.rate
    }

    pub fn max_rate(&self) -> Decimal256 {
        self.max_rate
    }

    pub fn max_change_rate(&self) -> Decimal256 {
        self.max_change_rate
    }

    fn validate(
        rate: Decimal256,
        max_rate: Decimal256,
        max_change_rate: Decimal256,
    ) -> Result<(), anyhow::Error> {
        if max_rate > ONE_DEC {
            // max rate cannot be greater than 1
            return Err(anyhow::anyhow!("max_rate too big"));
        }
        if rate > max_rate {
            // rate cannot be greater than the max rate
            return Err(anyhow::anyhow!("rate is bigger than max_rate"));
        }
        if max_change_rate > max_rate {
            // change rate cannot be greater than the max rate
            return Err(anyhow::anyhow!("max_change_rate is bigger than max_rate"));
        }
        Ok(())
    }
}

impl From<CommissionRates> for inner::CommissionRates {
    fn from(
        CommissionRates {
            rate,
            max_rate,
            max_change_rate,
        }: CommissionRates,
    ) -> Self {
        Self {
            rate: rate.to_cosmos_proto_string(),
            max_rate: max_rate.to_cosmos_proto_string(),
            max_change_rate: max_change_rate.to_cosmos_proto_string(),
        }
    }
}

impl TryFrom<inner::CommissionRates> for CommissionRates {
    type Error = StdError;

    fn try_from(
        inner::CommissionRates {
            rate,
            max_change_rate,
            max_rate,
        }: inner::CommissionRates,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            rate: Decimal256::from_cosmos_proto_string(&rate)?,
            max_rate: Decimal256::from_cosmos_proto_string(&max_rate)?,
            max_change_rate: Decimal256::from_cosmos_proto_string(&max_change_rate)?,
        })
    }
}

impl Protobuf<inner::CommissionRates> for CommissionRates {}

/// Commission defines commission parameters for a given validator.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Protobuf)]
#[proto(raw = "inner::Commission")]
pub struct Commission {
    /// commission_rates defines the initial commission rates to be used for creating a validator.
    #[proto(optional)]
    commission_rates: CommissionRates,
    /// update_time is the last time the commission rate was changed.
    #[proto(optional)]
    update_time: Timestamp,
}

impl Commission {
    pub fn new(commission_rates: CommissionRates, update_time: Timestamp) -> Commission {
        Commission {
            commission_rates,
            update_time,
        }
    }

    pub fn new_checked(
        &self,
        commission_rates: CommissionRates,
        update_time: Timestamp,
    ) -> Result<Commission, anyhow::Error> {
        let diff = update_time.checked_sub(&self.update_time).unwrap();
        if i64::from(diff.duration_hours()) < 24 {
            return Err(anyhow::anyhow!(
                "new rate cannot be changed more than once within 24 hours"
            ));
        }
        Ok(Commission {
            commission_rates,
            update_time,
        })
    }

    pub fn commission_rates(&self) -> &CommissionRates {
        &self.commission_rates
    }

    pub fn update_time(&self) -> &Timestamp {
        &self.update_time
    }
}

/// Description defines a validator description.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditDescription {
    /// moniker defines a human-readable name for the validator.
    pub moniker: Option<String>,
    /// identity defines an optional identity signature (ex. UPort or Keybase).
    pub identity: Option<String>,
    /// website defines an optional website link.
    pub website: Option<String>,
    /// security_contact defines an optional email for security contact.
    pub security_contact: Option<String>,
    /// details define other optional details.
    pub details: Option<String>,
}

impl From<Description> for EditDescription {
    fn from(value: Description) -> Self {
        let moniker = if value.moniker == DO_NOT_MODIFY_DESCRIPTION {
            None
        } else {
            Some(value.moniker)
        };
        let identity = if value.identity == DO_NOT_MODIFY_DESCRIPTION {
            None
        } else {
            Some(value.identity)
        };
        let website = if value.website == DO_NOT_MODIFY_DESCRIPTION {
            None
        } else {
            Some(value.website)
        };
        let security_contact = if value.security_contact == DO_NOT_MODIFY_DESCRIPTION {
            None
        } else {
            Some(value.security_contact)
        };
        let details = if value.details == DO_NOT_MODIFY_DESCRIPTION {
            None
        } else {
            Some(value.details)
        };

        Self {
            moniker,
            identity,
            website,
            security_contact,
            details,
        }
    }
}

impl From<EditDescription> for Description {
    fn from(value: EditDescription) -> Self {
        let moniker = value
            .moniker
            .unwrap_or(DO_NOT_MODIFY_DESCRIPTION.to_string());
        let identity = value
            .identity
            .unwrap_or(DO_NOT_MODIFY_DESCRIPTION.to_string());
        let website = value
            .website
            .unwrap_or(DO_NOT_MODIFY_DESCRIPTION.to_string());
        let security_contact = value
            .security_contact
            .unwrap_or(DO_NOT_MODIFY_DESCRIPTION.to_string());
        let details = value
            .details
            .unwrap_or(DO_NOT_MODIFY_DESCRIPTION.to_string());

        Self {
            moniker,
            identity,
            website,
            security_contact,
            details,
        }
    }
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
    pub fn try_new<T: Into<String>>(
        moniker: T,
        identity: T,
        website: T,
        security_contact: T,
        details: T,
    ) -> Result<Self, anyhow::Error> {
        let desc = Self {
            moniker: moniker.into(),
            identity: identity.into(),
            website: website.into(),
            security_contact: security_contact.into(),
            details: details.into(),
        };

        desc.ensure_length()?;

        Ok(desc)
    }

    /// create_updated_description creates a description with the base of current description
    /// supplemented by values from a given description. An error is
    /// returned if the resulting description contains an invalid length.
    pub fn create_updated_description(
        &self,
        other: &EditDescription,
    ) -> Result<Description, anyhow::Error> {
        let mut description = self.clone();
        if let Some(moniker) = &other.moniker {
            description.moniker.clone_from(moniker);
        }
        if let Some(identity) = &other.identity {
            description.identity.clone_from(identity);
        }
        if let Some(website) = &other.website {
            description.website.clone_from(website);
        }
        if let Some(security_contact) = &other.security_contact {
            description.security_contact.clone_from(security_contact);
        }
        if let Some(details) = &other.details {
            description.details.clone_from(details);
        }
        description.ensure_length()?;
        Ok(description)
    }

    // TODO: these constraints should be part of the `Description` type definition
    pub fn ensure_length(&self) -> Result<(), anyhow::Error> {
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

    fn form_ensure_length_err(&self, name: &str, got: usize, max: usize) -> anyhow::Error {
        anyhow::anyhow!("invalid {name} length; got: {got}, max: {max}")
    }
}

impl From<CreateValidator> for inner::MsgCreateValidator {
    fn from(msg: CreateValidator) -> Self {
        Self {
            description: Some(inner::Description {
                moniker: msg.description.moniker,
                identity: msg.description.identity,
                website: msg.description.website,
                security_contact: msg.description.security_contact,
                details: msg.description.details,
            }),
            commission: Some(inner::CommissionRates {
                rate: msg.commission.rate.to_cosmos_proto_string(),
                max_rate: msg.commission.max_rate.to_cosmos_proto_string(),
                max_change_rate: msg.commission.max_change_rate.to_cosmos_proto_string(),
            }),
            min_self_delegation: msg.min_self_delegation.to_string(),
            delegator_address: msg.delegator_address.to_string(),
            validator_address: msg.validator_address.to_string(),
            pubkey: Some(gears::crypto::public::PublicKey::from(msg.pubkey).into()),
            value: Some(msg.value.into()),
        }
    }
}

impl TryFrom<inner::MsgCreateValidator> for CreateValidator {
    type Error = CoreError;

    fn try_from(val: inner::MsgCreateValidator) -> Result<Self, Self::Error> {
        // TODO: there are missing checks here, like that validator address and delegator address should be the same
        // see https://github.com/cosmos/cosmos-sdk/blob/2582f0aab7b2cbf66ade066fe570a4622cf0b098/x/staking/types/msg.go#L89-L144
        let description = val
            .description
            .ok_or(CoreError::MissingField("description".into()))?;
        let commission = val
            .commission
            .ok_or(CoreError::MissingField("commission".into()))?;
        let pubkey = val.pubkey.ok_or(CoreError::MissingField("pubkey".into()))?;
        let pubkey = gears::crypto::public::PublicKey::try_from(pubkey)
            .map_err(|e| CoreError::DecodeGeneral(e.to_string()))?;

        let delegator_address = AccAddress::from_bech32(&val.delegator_address)
            .map_err(|e| CoreError::DecodeAddress(e.to_string()))?;
        let validator_address = ValAddress::from_bech32(&val.validator_address)
            .map_err(|e| CoreError::DecodeAddress(e.to_string()))?;

        if delegator_address != validator_address.into() {
            return Err(CoreError::DecodeGeneral(
                "delegator address and validator address must be derived from the same public key"
                    .into(),
            ));
        }

        Ok(CreateValidator {
            description: Description {
                moniker: description.moniker,
                identity: description.identity,
                website: description.website,
                security_contact: description.security_contact,
                details: description.details,
            },
            commission: CommissionRates {
                rate: Decimal256::from_cosmos_proto_string(&commission.rate)
                    .map_err(|e| CoreError::DecodeGeneral(e.to_string()))?,
                max_rate: Decimal256::from_cosmos_proto_string(&commission.max_rate)
                    .map_err(|e| CoreError::DecodeGeneral(e.to_string()))?,
                max_change_rate: Decimal256::from_cosmos_proto_string(&commission.max_change_rate)
                    .map_err(|e| CoreError::DecodeGeneral(e.to_string()))?,
            },
            min_self_delegation: Uint256::from_str(&val.min_self_delegation)
                .map_err(|e| CoreError::DecodeGeneral(e.to_string()))?,
            delegator_address: AccAddress::from_bech32(&val.delegator_address)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            validator_address: ValAddress::from_bech32(&val.validator_address)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            pubkey: pubkey.into(),
            value: val
                .value
                .ok_or(CoreError::MissingField("value".into()))?
                .try_into()
                .map_err(|e| CoreError::Coin(format!("{e}")))?,
        })
    }
}

/// CreateValidator defines a SDK message for creating a new validator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AppMessage)]
#[serde(tag = "@type")]
#[serde(rename = "/cosmos.staking.v1beta1.MsgCreateValidator")]
#[msg(url = "/cosmos.staking.v1beta1.MsgCreateValidator")]
pub struct CreateValidator {
    pub description: Description,
    pub commission: CommissionRates,
    pub min_self_delegation: Uint256,
    #[msg(signer)]
    pub delegator_address: AccAddress,
    pub validator_address: ValAddress,
    pub pubkey: PublicKey,
    pub value: UnsignedCoin,
}

impl Protobuf<inner::MsgCreateValidator> for CreateValidator {}

impl ValueRenderer for CreateValidator {
    fn format<MG: gears::signing::handler::MetadataGetter>(
        &self,
        _get_metadata: &MG,
    ) -> Result<
        Vec<gears::types::rendering::screen::Screen>,
        gears::signing::renderer::value_renderer::RenderError,
    > {
        Err(gears::signing::renderer::value_renderer::RenderError::NotImplemented)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AppMessage)]
#[msg(url = "/cosmos.staking.v1beta1.MsgEditValidator")]
pub struct EditValidator {
    pub description: EditDescription,
    pub commission_rate: Option<Decimal256>, // TODO: add a CommissionRate type to capture the =< 1 constraint currently this is checked here https://github.com/rumos-io/gears/blob/672d6cf7e4376076c218b46121e197ac1f1029a7/x/staking/src/keeper/validator.rs#L67
    pub min_self_delegation: Option<Uint256>,
    pub validator_address: ValAddress,
    // for method `get_signers`. The sdk converts validator_address
    #[msg(signer)]
    from_address: AccAddress,
}

impl EditValidator {
    pub fn new(
        description: EditDescription,
        commission_rate: Option<Decimal256>,
        min_self_delegation: Option<Uint256>,
        validator_address: ValAddress,
    ) -> EditValidator {
        EditValidator {
            description,
            commission_rate,
            min_self_delegation,
            validator_address: validator_address.clone(),
            from_address: validator_address.into(),
        }
    }

    pub fn get_signers(&self) -> Vec<&AccAddress> {
        vec![&self.from_address]
    }
}
#[derive(Clone, PartialEq, Message)]
pub struct EditValidatorRaw {
    #[prost(message, optional)]
    pub description: Option<Description>,
    #[prost(string)]
    pub validator_address: String,
    #[prost(string, optional)]
    pub commission_rate: Option<String>,
    #[prost(string, optional)]
    pub min_self_delegation: Option<String>,
}

impl From<EditValidator> for EditValidatorRaw {
    fn from(src: EditValidator) -> Self {
        Self {
            description: Some(src.description.into()),
            commission_rate: src
                .commission_rate
                .map(|com_rate| com_rate.to_cosmos_proto_string()),
            min_self_delegation: src.min_self_delegation.map(|msd| msd.to_string()),
            validator_address: src.validator_address.to_string(),
        }
    }
}

impl TryFrom<EditValidatorRaw> for EditValidator {
    type Error = CoreError;

    fn try_from(src: EditValidatorRaw) -> Result<Self, Self::Error> {
        // TODO: there are missing checks here, like that at least one of the description fields should be non empty (yes that's right as long as one is set all is good!)
        // see https://github.com/cosmos/cosmos-sdk/blob/2582f0aab7b2cbf66ade066fe570a4622cf0b098/x/staking/types/msg.go#L185-L210
        // we should capture this restriction in the description type
        let commission_rate = if let Some(rate) = src.commission_rate {
            Some(
                Decimal256::from_cosmos_proto_string(&rate)
                    .map_err(|e| CoreError::DecodeGeneral(e.to_string()))?,
            )
        } else {
            None
        };
        let min_self_delegation = if let Some(min_self_delegation) = src.min_self_delegation {
            Some(
                Uint256::from_str(&min_self_delegation)
                    .map_err(|e| CoreError::DecodeGeneral(e.to_string()))?,
            )
        } else {
            None
        };

        let validator_address = ValAddress::from_bech32(&src.validator_address)
            .map_err(|e| CoreError::DecodeAddress(e.to_string()))?;

        Ok(EditValidator {
            description: src
                .description
                .ok_or(CoreError::MissingField(
                    "Missing field 'description'.".into(),
                ))?
                .into(),
            commission_rate,
            min_self_delegation,
            validator_address: validator_address.clone(),
            from_address: validator_address.into(),
        })
    }
}

impl Protobuf<EditValidatorRaw> for EditValidator {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct DelegateMsgRaw {
    #[prost(string)]
    pub delegator_address: String,
    #[prost(string)]
    pub validator_address: String,
    #[prost(message, optional)]
    pub amount: Option<CoinRaw>,
}

impl From<DelegateMsg> for DelegateMsgRaw {
    fn from(src: DelegateMsg) -> Self {
        Self {
            delegator_address: src.delegator_address.to_string(),
            validator_address: src.validator_address.to_string(),
            amount: Some(src.amount.into()),
        }
    }
}

/// Creates a new DelegateMsg transaction message instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AppMessage)]
#[msg(
    url = "/cosmos.staking.v1beta1.MsgDelegate",
    amino_url = "cosmos-sdk/MsgDelegate"
)]
pub struct DelegateMsg {
    #[msg(signer)]
    pub delegator_address: AccAddress,
    pub validator_address: ValAddress,
    pub amount: UnsignedCoin,
}

impl TryFrom<DelegateMsgRaw> for DelegateMsg {
    type Error = CoreError;

    fn try_from(src: DelegateMsgRaw) -> Result<Self, Self::Error> {
        Ok(DelegateMsg {
            delegator_address: AccAddress::from_bech32(&src.delegator_address)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            validator_address: ValAddress::from_bech32(&src.validator_address)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            amount: src
                .amount
                .ok_or(CoreError::MissingField("Missing field 'amount'.".into()))?
                .try_into()
                .map_err(|e| CoreError::Coin(format!("{e}")))?,
        })
    }
}

impl Protobuf<DelegateMsgRaw> for DelegateMsg {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct RedelegateMsgRaw {
    #[prost(string)]
    pub delegator_address: String,
    #[prost(string)]
    pub src_validator_address: String,
    #[prost(string)]
    pub dst_validator_address: String,
    #[prost(message, optional)]
    pub amount: Option<CoinRaw>,
}

impl From<RedelegateMsg> for RedelegateMsgRaw {
    fn from(src: RedelegateMsg) -> Self {
        Self {
            delegator_address: src.delegator_address.to_string(),
            src_validator_address: src.src_validator_address.to_string(),
            dst_validator_address: src.dst_validator_address.to_string(),
            amount: Some(src.amount.into()),
        }
    }
}

/// Creates a new RedelegateMsg transaction message instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AppMessage)]
#[msg(url = "/cosmos.staking.v1beta1.MsgBeginRedelegate")]
pub struct RedelegateMsg {
    #[msg(signer)]
    pub delegator_address: AccAddress,
    pub src_validator_address: ValAddress,
    pub dst_validator_address: ValAddress,
    pub amount: UnsignedCoin,
}

impl TryFrom<RedelegateMsgRaw> for RedelegateMsg {
    type Error = CoreError;

    fn try_from(src: RedelegateMsgRaw) -> Result<Self, Self::Error> {
        Ok(RedelegateMsg {
            delegator_address: AccAddress::from_bech32(&src.delegator_address)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            src_validator_address: ValAddress::from_bech32(&src.src_validator_address)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            dst_validator_address: ValAddress::from_bech32(&src.dst_validator_address)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            amount: src
                .amount
                .ok_or(CoreError::MissingField("Missing field 'amount'.".into()))?
                .try_into()
                .map_err(|e| CoreError::Coin(format!("{e}")))?,
        })
    }
}

impl Protobuf<RedelegateMsgRaw> for RedelegateMsg {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct UndelegateMsgRaw {
    #[prost(string)]
    pub delegator_address: String,
    #[prost(string)]
    pub validator_address: String,
    #[prost(message, optional)]
    pub amount: Option<CoinRaw>,
}

impl From<UndelegateMsg> for UndelegateMsgRaw {
    fn from(src: UndelegateMsg) -> Self {
        Self {
            delegator_address: src.delegator_address.to_string(),
            validator_address: src.validator_address.to_string(),
            amount: Some(src.amount.into()),
        }
    }
}

/// Creates a new UndelegateMsg transaction message instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AppMessage)]
#[msg(url = "/cosmos.staking.v1beta1.MsgUndelegate")]
pub struct UndelegateMsg {
    #[msg(signer)]
    pub delegator_address: AccAddress,
    pub validator_address: ValAddress,
    pub amount: UnsignedCoin,
}

impl TryFrom<UndelegateMsgRaw> for UndelegateMsg {
    type Error = CoreError;

    fn try_from(src: UndelegateMsgRaw) -> Result<Self, Self::Error> {
        Ok(UndelegateMsg {
            delegator_address: AccAddress::from_bech32(&src.delegator_address)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            validator_address: ValAddress::from_bech32(&src.validator_address)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            amount: src
                .amount
                .ok_or(CoreError::MissingField("Missing field 'amount'.".into()))?
                .try_into()
                .map_err(|e| CoreError::Coin(format!("{e}")))?,
        })
    }
}

impl Protobuf<UndelegateMsgRaw> for UndelegateMsg {}
