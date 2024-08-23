use std::{net::SocketAddr, path::PathBuf};

use clap::Args;
use gears::tendermint::types::proto::crypto::PublicKey as TendermintPublicKey;
use gears::{
    socket_addr,
    types::{base::coin::UnsignedCoin, decimal256::Decimal256, uint::Uint256},
};

use crate::gentx::GentxCmd;

const DEFAULT_NODE_IP: SocketAddr = socket_addr!(192, 168, 1, 106, 26656);

#[derive(Args, Debug, Clone)]
pub struct GentxCli {
    /// The validator's Protobuf JSON encoded public key
    pub pubkey: Option<TendermintPublicKey>,
    /// Amount of coins to bond
    pub amount: UnsignedCoin,
    /// The validator's name
    pub moniker: String,
    /// The optional identity signature (ex. UPort or Keybase)
    #[arg(long)]
    pub identity: String,
    /// The validator's (optional) website
    #[arg(long)]
    pub website: String,
    /// The validator's (optional) security contact email
    #[arg(long)]
    pub security_contact: String,
    /// The validator's (optional) details
    #[arg(long)]
    pub details: String,
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
    #[arg(long, default_value_t = Uint256::one())]
    pub min_self_delegation: Uint256,

    /// Output dir to place a new tx file
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// The node's public IP
    #[arg(long, default_value_t = DEFAULT_NODE_IP)]
    pub ip: SocketAddr,
    /// The node's NodeID
    #[arg(long)]
    pub node_id: String,
}

impl TryFrom<GentxCli> for GentxCmd {
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
        }: GentxCli,
    ) -> Result<Self, Self::Error> {
        let node_id = match node_id.is_empty() {
            true => None,
            false => Some(node_id),
        };

        Ok(Self {
            output,
            node_id,
            ip,
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
        })
    }
}
