use crate::{
    commands::client::{query::QueryCommand, ExtendedQueryCommand},
    config::DEFAULT_TENDERMINT_RPC_ADDRESS,
};
use clap::{ArgAction, Subcommand, ValueHint};
use tendermint::types::proto::block::Height;

/// Querying subcommands
#[derive(Debug, Clone, ::clap::Args)]
pub struct CliQueryCommand<C: Subcommand> {
    /// <host>:<port> to Tendermint RPC interface for this chain
    #[arg(long, global = true, action = ArgAction::Set, value_hint = ValueHint::Url, default_value_t = DEFAULT_TENDERMINT_RPC_ADDRESS.parse().expect( "const should be valid"))]
    pub node: url::Url,
    /// TODO
    #[arg(long, global = true)]
    pub height: Option<Height>,

    #[command(subcommand)]
    pub command: QueryCommands<C>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum QueryCommands<S: Subcommand> {
    #[command(flatten)]
    QueryCmd(S),
    Tx(TxQueryCli),
    Txs(TxsQueryCli),
}

/// Query for a transaction by hash, "<addr>/<seq>" combination or comma-separated signatures
/// in a committed block
#[derive(Debug, Clone, ::clap::Args)]
pub struct TxQueryCli {
    pub hash: String,
    #[arg(long, default_value_t = TxQueryType::Hash)]
    pub query_type: TxQueryType,
}

/// Query for paginated transactions that match a set of events
#[derive(Debug, Clone, ::clap::Args)]
pub struct TxsQueryCli {
    #[arg(long)]
    pub events: String,
    #[arg(long, default_value_t = 1)]
    pub page: u32,
    #[arg(long, default_value_t = 30)]
    pub limit: u32,
}

#[derive(Debug, Clone, strum::EnumString, strum::Display)]
pub enum TxQueryType {
    #[strum(serialize = "hash")]
    Hash,
    #[strum(serialize = "acc_seq")]
    AccSeq,
    #[strum(serialize = "signature")]
    Signature,
}

impl<C, AC, ERR> TryFrom<CliQueryCommand<C>> for ExtendedQueryCommand<AC, TxQueryCli, TxsQueryCli>
where
    C: Subcommand,
    AC: TryFrom<C, Error = ERR>,
{
    type Error = ERR;

    fn try_from(value: CliQueryCommand<C>) -> Result<Self, Self::Error> {
        let CliQueryCommand {
            node,
            height,
            command,
        } = value;

        let query = match command {
            QueryCommands::QueryCmd(c) => ExtendedQueryCommand::QueryCmd(QueryCommand {
                node,
                height,
                inner: c.try_into()?,
            }),
            QueryCommands::Tx(c) => ExtendedQueryCommand::Tx(QueryCommand {
                node,
                height,
                inner: c,
            }),
            QueryCommands::Txs(c) => ExtendedQueryCommand::Txs(QueryCommand {
                node,
                height,
                inner: c,
            }),
        };
        Ok(query)
    }
}
