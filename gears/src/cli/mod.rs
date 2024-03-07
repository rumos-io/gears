use clap::{Args, Subcommand};
use clap_complete::Shell;

use crate::{ApplicationCommands, ApplicationInfo};

use self::{
    genesis::CliGenesisCommand, init::CliInitCommand, key::CliKeyCommand, query::CliQueryCommand,
    run::CliRunCommand, tx::CliTxCommand,
};

pub mod aux;
pub mod genesis;
pub mod init;
pub mod key;
pub mod query;
pub mod run;
pub mod tx;
mod utils;

#[derive(Debug, Clone, ::clap::Subcommand)]
pub enum CliApplicationCommands<T, CliAUX, CliTX, CliQue>
where
    T: ApplicationInfo,
    CliAUX: Args,
    CliTX: Subcommand,
    CliQue: Subcommand,
{
    Init(CliInitCommand<T>),
    Run(CliRunCommand<T>),
    #[command(subcommand)]
    Keys(CliKeyCommand<T>),
    GenesisAdd(CliGenesisCommand<T>),
    Aux(CliAUX),
    Tx(CliTxCommand<T, CliTX>),
    Query(CliQueryCommand<CliQue>),
}

#[derive(Debug, Clone, ::clap::Parser)]
#[command(name = T::APP_NAME)]
pub struct CliApplicationArgs<T, CliAUX, CliTX, CliQue>
where
    T: ApplicationInfo,
    CliAUX: Args,
    CliTX: Subcommand,
    CliQue: Subcommand,
{
    #[command(subcommand, value_parser = value_parser!(PhantomData))]
    pub command: Option<CliApplicationCommands<T, CliAUX, CliTX, CliQue>>,
    /// If provided, outputs the completion file for given shell
    #[clap(long = "completion")]
    pub completion: Option<Shell>,
}

impl<T: ApplicationInfo, CliAUX, AUX, CliTX, TX, CliQue, QUE, ERR>
    TryFrom<CliApplicationCommands<T, CliAUX, CliTX, CliQue>> for ApplicationCommands<AUX, TX, QUE>
where
    CliAUX: Args,
    AUX: TryFrom<CliAUX, Error = ERR>,
    CliTX: Subcommand,
    TX: TryFrom<CliTX, Error = ERR>,
    CliQue: Subcommand,
    QUE: TryFrom<CliQue, Error = ERR>,
{
    type Error = ERR;

    fn try_from(
        value: CliApplicationCommands<T, CliAUX, CliTX, CliQue>,
    ) -> Result<Self, Self::Error> {
        let res = match value {
            CliApplicationCommands::Init(cmd) => Self::Init(cmd.into()),
            CliApplicationCommands::Run(cmd) => Self::Run(cmd.into()),
            CliApplicationCommands::Keys(cmd) => Self::Keys(cmd.into()),
            CliApplicationCommands::GenesisAdd(cmd) => Self::GenesisAdd(cmd.into()),
            CliApplicationCommands::Aux(cmd) => Self::Aux(cmd.try_into()?),
            CliApplicationCommands::Tx(cmd) => Self::Tx(cmd.try_into()?),
            CliApplicationCommands::Query(cmd) => Self::Query(cmd.try_into()?),
        };

        Ok(res)
    }
}
