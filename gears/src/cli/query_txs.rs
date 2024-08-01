use crate::{commands::client::query::QueryCommand, config::DEFAULT_TENDERMINT_RPC_ADDRESS};
use clap::{ArgAction, Args, ValueHint};
use tendermint::types::proto::block::Height;

#[derive(Debug, Clone, strum::EnumString, strum::Display)]
pub enum TxQueryType {
    #[strum(serialize = "hash")]
    Hash,
    #[strum(serialize = "acc_seq")]
    AccSeq,
    #[strum(serialize = "signature")]
    Signature,
}

#[derive(Debug, Clone, Args)]
pub struct TxQueryCli {
    pub hash: String,
    #[arg(long, default_value_t = TxQueryType::Hash)]
    pub query_type: TxQueryType,
}

#[derive(Debug, Clone, Args)]
pub struct TxsQueryCli {
    #[arg(long)]
    pub events: String,
    #[arg(long, default_value_t = 1)]
    pub page: u32,
    #[arg(long, default_value_t = 30)]
    pub limit: u32,
}

/// Query for a transaction by hash, "<addr>/<seq>" combination or comma-separated signatures
/// in a committed block
#[derive(Debug, Clone, ::clap::Args)]
pub struct CliQueryTxCommand {
    /// <host>:<port> to Tendermint RPC interface for this chain
    #[arg(long, global = true, action = ArgAction::Set, value_hint = ValueHint::Url, default_value_t = DEFAULT_TENDERMINT_RPC_ADDRESS.parse().expect( "const should be valid"))]
    pub node: url::Url,
    /// TODO
    #[arg(long, global = true)]
    pub height: Option<Height>,

    #[command(flatten)]
    pub command: TxQueryCli,
}

impl From<CliQueryTxCommand> for QueryCommand<TxQueryCli> {
    fn from(value: CliQueryTxCommand) -> Self {
        let CliQueryTxCommand {
            node,
            height,
            command,
        } = value;

        QueryCommand {
            node,
            height,
            inner: command,
        }
    }
}

/// Query for paginated transactions that match a set of events
#[derive(Debug, Clone, ::clap::Args)]
pub struct CliQueryTxsCommand {
    /// <host>:<port> to Tendermint RPC interface for this chain
    #[arg(long, global = true, action = ArgAction::Set, value_hint = ValueHint::Url, default_value_t = DEFAULT_TENDERMINT_RPC_ADDRESS.parse().expect( "const should be valid"))]
    pub node: url::Url,
    /// TODO
    #[arg(long, global = true)]
    pub height: Option<Height>,

    #[command(flatten)]
    pub command: TxsQueryCli,
}

impl From<CliQueryTxsCommand> for QueryCommand<TxsQueryCli> {
    fn from(value: CliQueryTxsCommand) -> Self {
        let CliQueryTxsCommand {
            node,
            height,
            command,
        } = value;

        QueryCommand {
            node,
            height,
            inner: command,
        }
    }
}
