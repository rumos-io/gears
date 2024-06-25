use crate::{
    consts::proto::DO_NOT_MODIFY_STRING, CommissionRates, CreateValidator, DelegateMsg,
    Description, EditValidator, Message as StakingMessage, RedelegateMsg,
};
use anyhow::Result;
use clap::{Args, Subcommand};
use gears::{
    error::AppError,
    tendermint::types::proto::crypto::PublicKey as TendermintPublicKey,
    types::{
        address::{AccAddress, ValAddress},
        base::coin::Coin,
        decimal256::Decimal256,
        tx::TxMessage,
        uint::Uint256,
    },
};
use std::str::FromStr;

#[derive(Args, Debug, Clone)]
pub struct StakingTxCli {
    #[command(subcommand)]
    pub command: StakingCommands,
}

#[derive(Subcommand, Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum StakingCommands {
    /// Create new validator initialized with a self-delegation to it
    CreateValidator {
        /// The validator's Protobuf JSON encoded public key
        pubkey: TendermintPublicKey,
        /// Amount of coins to bond
        amount: Coin,
        /// The validator's name
        moniker: String,
        /// The optional identity signature (ex. UPort or Keybase)
        #[arg(long)]
        identity: String,
        /// The validator's (optional) website
        #[arg(long)]
        website: String,
        /// The validator's (optional) security contact email
        #[arg(long)]
        security_contact: String,
        /// The validator's (optional) details
        #[arg(long)]
        details: String,
        /// The initial commission rate percentage
        #[arg(long, default_value_t = 0.1.to_string())]
        commission_rate: String,
        /// The maximum commission rate percentage
        #[arg(long, default_value_t = 0.2.to_string())]
        commission_max_rate: String,
        /// The maximum commission change rate percentage (per day)
        #[arg(long, default_value_t = 0.01.to_string())]
        commission_max_change_rate: String,
        /// The minimum self delegation required on the validator
        #[arg(long, default_value_t = Uint256::one())]
        min_self_delegation: Uint256,
    },
    /// Edit an existing validator account
    EditValidator {
        /// The validator's name
        #[arg(default_value = DO_NOT_MODIFY_STRING)]
        moniker: String,
        /// The optional identity signature (ex. UPort or Keybase)
        #[arg(long, default_value = DO_NOT_MODIFY_STRING)]
        identity: String,
        /// The validator's (optional) website
        #[arg(long, default_value = DO_NOT_MODIFY_STRING)]
        website: String,
        /// The validator's (optional) security contact email
        #[arg(long, default_value = DO_NOT_MODIFY_STRING)]
        security_contact: String,
        /// The validator's (optional) details
        #[arg(long, default_value = DO_NOT_MODIFY_STRING)]
        details: String,
        /// The initial commission rate percentage
        #[arg(long)]
        commission_rate: Option<String>,
        /// The minimum self delegation required on the validator
        #[arg(long)]
        min_self_delegation: Option<Uint256>,
    },
    /// Delegate liquid tokens to a validator
    Delegate {
        /// The validator account address
        validator_address: ValAddress,
        /// Amount of coins to bond
        amount: Coin,
    },
    /// Redelegate illiquid tokens from one validator to another
    Redelegate {
        /// The validator account address from which sends coins
        src_validator_address: ValAddress,
        /// The validator account address that receives coins
        dst_validator_address: ValAddress,
        /// Amount of coins to redelegate
        amount: Coin,
    },
}

pub fn run_staking_tx_command(
    args: StakingTxCli,
    from_address: AccAddress,
) -> Result<StakingMessage> {
    match &args.command {
        StakingCommands::CreateValidator {
            pubkey,
            amount,
            moniker,
            identity,
            website,
            security_contact,
            details,
            commission_rate,
            commission_max_rate,
            commission_max_change_rate,
            min_self_delegation,
        } => {
            let delegator_address = from_address.clone();
            let validator_address = ValAddress::try_from(Vec::from(from_address))?;
            let description = Description {
                moniker: moniker.to_string(),
                identity: identity.to_string(),
                website: website.to_string(),
                security_contact: security_contact.to_string(),
                details: details.to_string(),
            };
            let commission = CommissionRates::new(
                Decimal256::from_str(commission_rate)?,
                Decimal256::from_str(commission_max_rate)?,
                Decimal256::from_str(commission_max_change_rate)?,
            )?;

            let msg = StakingMessage::CreateValidator(CreateValidator {
                description,
                commission,
                min_self_delegation: *min_self_delegation,
                delegator_address,
                validator_address,
                pub_key: pubkey.clone(),
                value: amount.clone(),
            });
            msg.validate_basic().map_err(AppError::TxValidation)?;
            Ok(msg)

            // genOnly, _ := fs.GetBool(flags.FlagGenerateOnly)
            // if genOnly {
            //     ip, _ := fs.GetString(FlagIP)
            //     nodeID, _ := fs.GetString(FlagNodeID)
            //
            //     if nodeID != "" && ip != "" {
            //         txf = txf.WithMemo(fmt.Sprintf("%s@%s:26656", nodeID, ip))
            //     }
            // }
            //
            // return txf, msg, nil
        }
        StakingCommands::EditValidator {
            moniker,
            identity,
            website,
            security_contact,
            details,
            commission_rate,
            min_self_delegation,
        } => {
            let delegator_address = from_address.clone();
            let validator_address = ValAddress::try_from(Vec::from(from_address))?;
            let description = Description {
                moniker: moniker.to_string(),
                identity: identity.to_string(),
                website: website.to_string(),
                security_contact: security_contact.to_string(),
                details: details.to_string(),
            };
            let commission_rate = if let Some(rate) = commission_rate {
                Some(Decimal256::from_str(rate)?)
            } else {
                None
            };
            let msg = StakingMessage::EditValidator(EditValidator {
                description,
                commission_rate,
                min_self_delegation: *min_self_delegation,
                validator_address,
                from_address: delegator_address,
            });
            msg.validate_basic().map_err(AppError::TxValidation)?;
            Ok(msg)
        }
        StakingCommands::Delegate {
            validator_address,
            amount,
        } => Ok(StakingMessage::Delegate(DelegateMsg {
            delegator_address: from_address.clone(),
            validator_address: validator_address.clone(),
            amount: amount.clone(),
        })),
        StakingCommands::Redelegate {
            src_validator_address,
            dst_validator_address,
            amount,
        } => Ok(StakingMessage::Redelegate(RedelegateMsg {
            delegator_address: from_address.clone(),
            src_validator_address: src_validator_address.clone(),
            dst_validator_address: dst_validator_address.clone(),
            amount: amount.clone(),
        })),
    }
}
