use crate::{
    application::ApplicationInfo, cli::config::client_config, commands::client::query::QueryCommand,
};
use clap::{ArgAction, Args, ValueHint};
use std::marker::PhantomData;
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
pub struct CliQueryTxCommand<T: ApplicationInfo> {
    /// <host>:<port> to Tendermint RPC interface for this chain
    #[arg(long, global = true, action = ArgAction::Set, value_hint = ValueHint::Url, default_value_t = std::env::var("GEARS_NODE").map(|v| v.parse().expect("GEARS_NODE should be a valid http/https url")).unwrap_or(client_config(&T::home_dir()).node()))]
    pub node: url::Url,
    /// TODO
    #[arg(long, global = true)]
    pub height: Option<Height>,

    #[command(flatten)]
    pub command: TxQueryCli,

    #[arg(skip)]
    _marker: PhantomData<T>,
}

impl<T: ApplicationInfo> From<CliQueryTxCommand<T>> for QueryCommand<TxQueryCli> {
    fn from(value: CliQueryTxCommand<T>) -> Self {
        let CliQueryTxCommand {
            node,
            height,
            command,
            ..
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
pub struct CliQueryTxsCommand<T: ApplicationInfo> {
    /// <host>:<port> to Tendermint RPC interface for this chain
    #[arg(long, global = true, action = ArgAction::Set, value_hint = ValueHint::Url, default_value_t = std::env::var("GEARS_NODE").map(|v| v.parse().expect("GEARS_NODE should be a valid http/https url")).unwrap_or(client_config(&T::home_dir()).node()))]
    pub node: url::Url,
    /// TODO
    #[arg(long, global = true)]
    pub height: Option<Height>,

    #[command(flatten)]
    pub command: TxsQueryCli,

    #[arg(skip)]
    _marker: PhantomData<T>,
}

impl<T: ApplicationInfo> From<CliQueryTxsCommand<T>> for QueryCommand<TxsQueryCli> {
    fn from(value: CliQueryTxsCommand<T>) -> Self {
        let CliQueryTxsCommand {
            node,
            height,
            command,
            ..
        } = value;

        QueryCommand {
            node,
            height,
            inner: command,
        }
    }
}
