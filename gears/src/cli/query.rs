use crate::{
    application::ApplicationInfo, cli::config::client_config, commands::client::query::QueryCommand,
};
use clap::{ArgAction, Subcommand, ValueHint};
use std::marker::PhantomData;
use tendermint::types::proto::block::Height;

/// Querying subcommands
#[derive(Debug, Clone, ::clap::Args)]
pub struct CliQueryCommand<T: ApplicationInfo, C: Subcommand> {
    /// <host>:<port> to Tendermint RPC interface for this chain
    #[arg(long, global = true, action = ArgAction::Set, value_hint = ValueHint::Url, env = "GEARS_NODE", default_value_t = client_config(&T::home_dir()).node())]
    pub node: url::Url,
    /// TODO
    #[arg(long, global = true)]
    pub height: Option<Height>,

    #[command(subcommand)]
    pub command: C,

    #[arg(skip)]
    _marker: PhantomData<T>,
}

impl<T, C, AC, ERR> TryFrom<CliQueryCommand<T, C>> for QueryCommand<AC>
where
    T: ApplicationInfo,
    C: Subcommand,
    AC: TryFrom<C, Error = ERR>,
{
    type Error = ERR;

    fn try_from(value: CliQueryCommand<T, C>) -> Result<Self, Self::Error> {
        let CliQueryCommand {
            node,
            height,
            command,
            ..
        } = value;

        Ok(QueryCommand {
            node,
            height,
            inner: command.try_into()?,
        })
    }
}
