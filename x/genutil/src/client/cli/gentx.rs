use std::marker::PhantomData;
use std::{net::SocketAddr, path::PathBuf};

use clap::{ArgAction, Args, ValueHint};
use gears::application::ApplicationInfo;
use gears::tendermint::types::proto::crypto::PublicKey as TendermintPublicKey;
use gears::{
    socket_addr,
    types::{base::coin::UnsignedCoin, decimal256::Decimal256, uint::Uint256},
};

use crate::gentx::GentxCmd;

const DEFAULT_NODE_IP: SocketAddr = socket_addr!(192, 168, 1, 106, 26656);

#[derive(Args, Debug, Clone)]
pub struct GentxCli<AI: ApplicationInfo> {
    /// The validator's Protobuf JSON encoded public key
    #[arg(long)]
    pub pubkey: Option<TendermintPublicKey>,
    /// Amount of coins to bond
    pub amount: UnsignedCoin,
    /// The validator's name
    pub moniker: String,
    /// The optional identity signature (ex. UPort or Keybase)
    #[arg(long)]
    pub identity: Option<String>,
    /// The validator's (optional) website
    #[arg(long)]
    pub website: Option<String>,
    /// The validator's (optional) security contact email
    #[arg(long)]
    pub security_contact: Option<String>,
    /// The validator's (optional) details
    #[arg(long)]
    pub details: Option<String>,
    /// The initial commission rate percentage
    /* 0.1 */
    #[arg(long, default_value_t = Decimal256::from_atomics(1u64, 1).unwrap())]
    pub commission_rate: Decimal256,
    /// The maximum commission rate percentage
    /* 0.2 */
    #[arg(long, default_value_t = Decimal256::from_atomics(2u64, 1).unwrap())]
    pub commission_max_rate: Decimal256,
    /// The maximum commission change rate percentage (per day)
    /* 0.01 */
    #[arg(long, default_value_t = Decimal256::from_atomics(1u64, 2).unwrap())]
    pub commission_max_change_rate: Decimal256,
    /// The minimum self delegation required on the validator
    #[arg(long, default_value_t = Uint256::zero())]
    pub min_self_delegation: Uint256,

    /// Build an transaction and write it to STDOUT
    #[arg(long)]
    pub generate_only: bool,
    /// Output dir to place a new tx file
    #[arg(long,   action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = AI::home_dir().join("config/gentx") )]
    pub output: PathBuf,
    /// The node's public IP
    #[arg(long, default_value_t = DEFAULT_NODE_IP)]
    pub ip: SocketAddr,
    /// The node's NodeID
    #[arg(long)]
    pub node_id: Option<String>,

    #[arg(skip)]
    _marker: PhantomData<AI>,
}

impl<AI: ApplicationInfo> TryFrom<GentxCli<AI>> for GentxCmd {
    type Error = anyhow::Error;

    fn try_from(
        GentxCli {
            output,
            ip,
            node_id,
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
            generate_only,
            _marker,
        }: GentxCli<AI>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            output: match generate_only {
                true => None,
                false => Some(output),
            },
            node_id,
            ip,
            pubkey,
            amount,
            moniker,
            identity: identity.unwrap_or_default(),
            website: website.unwrap_or_default(),
            security_contact: security_contact.unwrap_or_default(),
            details: details.unwrap_or_default(),
            commission_rate,
            commission_max_rate,
            commission_max_change_rate,
            min_self_delegation,
        })
    }
}
