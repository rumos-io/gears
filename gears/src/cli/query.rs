use clap::{ArgAction, Subcommand, ValueHint};
use tendermint::types::proto::block::Height;

use crate::{commands::client::query::QueryCommand, config::DEFAULT_TENDERMINT_RPC_ADDRESS};

/// Querying subcommands
#[derive(Debug, Clone, ::clap::Args)]
pub struct CliQueryCommand<C: Subcommand> {
    /// <host>:<port> to Tendermint RPC interface for this chain
    #[arg(long, global = true, action = ArgAction::Set, value_hint = ValueHint::Url, default_value_t = DEFAULT_TENDERMINT_RPC_ADDRESS.parse().expect( "const should be valid"))]
    pub node: url::Url,
    ///  Use a specific height to query state at (this can error if the node is pruning state)
    #[arg(long, global = true)]
    pub height: Option<Height>,

    #[command(subcommand)]
    pub command: C,
}

impl<C, AC, ERR> TryFrom<CliQueryCommand<C>> for QueryCommand<AC>
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

        Ok(Self {
            node,
            height,
            inner: command.try_into()?,
        })
    }
}
