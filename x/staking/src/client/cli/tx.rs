use crate::{
    encode_hex_str, CommissionRates, CreateValidator, Description, Message as StakingMessage,
};
use anyhow::Result;
use clap::{Args, Subcommand};
use gears::{
    core::address::{AccAddress, ValAddress},
    crypto::public::PublicKey,
    error::AppError,
    tendermint::types::proto::crypto::PublicKey as TendermintPublicKey,
    types::{base::coin::Coin, decimal256::Decimal256, tx::TxMessage, uint::Uint256},
};
use prost::Message;
use std::str::FromStr;

#[derive(Args, Debug, Clone)]
pub struct StakingTxCli {
    #[command(subcommand)]
    pub command: StakingCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum StakingCommands {
    /// Create new validator initialized with a self-delegation to it
    CreateValidator {
        /// The validator's Protobuf JSON encoded public key
        pubkey: String,
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
            let validator_address = ValAddress::try_from(encode_hex_str(&from_address.as_hex())?)?;
            let pub_key: PublicKey = TendermintPublicKey::decode(pubkey.as_bytes())?.try_into()?;
            let description = Description {
                moniker: moniker.to_string(),
                identity: identity.to_string(),
                website: website.to_string(),
                security_contact: security_contact.to_string(),
                details: details.to_string(),
            };
            let commission = CommissionRates {
                rate: Decimal256::from_str(commission_rate)?,
                max_rate: Decimal256::from_str(commission_max_rate)?,
                max_change_rate: Decimal256::from_str(commission_max_change_rate)?,
            };

            let msg = StakingMessage::CreateValidator(CreateValidator {
                description,
                commission,
                min_self_delegation: *min_self_delegation,
                delegator_address,
                validator_address,
                pub_key,
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
    }
}
