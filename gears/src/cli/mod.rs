use std::io::Write;

use anyhow::anyhow;
use clap::{ArgAction, Command, CommandFactory, Subcommand};
pub use clap_complete::Shell;
use clap_complete::{generate, Generator};
use human_panic::setup_panic;
use tracing::metadata::LevelFilter;

use crate::application::{
    command::{app::AppCommands, client::ClientCommands, ApplicationCommands},
    ApplicationInfo,
};

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

fn write_completions<G: Generator>(gen: G, cmd: &mut Command, buf: &mut dyn Write) {
    generate(gen, cmd, cmd.get_name().to_string(), buf);
}


#[derive(Debug, Clone, Default, strum::Display, clap::ValueEnum)]
pub enum LogLevel
{
    #[strum( to_string = "debug")]
    Debug,
    #[default]
    #[strum( to_string = "info")]
    Info,
    #[strum( to_string = "warn")]
    Warn,
    #[strum( to_string = "error")]
    Error,
    #[strum( to_string = "off")]
    Off,
}

impl From<LogLevel> for LevelFilter
{
    fn from(value: LogLevel) -> Self {
        match value
        {
            LogLevel::Debug => Self::DEBUG,
            LogLevel::Info => Self::INFO,
            LogLevel::Warn => Self::WARN,
            LogLevel::Error => Self::ERROR,
            LogLevel::Off => Self::OFF,
        }
    }
}

#[derive(Debug, Clone, ::clap::Parser)]
#[command(name = T::APP_NAME, version = T::APP_VERSION)]
pub struct CliApplicationArgs<T, CliClientAUX, CliAppAUX, CliTX, CliQue>
where
    T: ApplicationInfo,
    CliClientAUX: Subcommand,
    CliAppAUX: Subcommand,
    CliTX: Subcommand,
    CliQue: Subcommand,
{
    #[command(subcommand, value_parser = value_parser!(PhantomData))]
    pub command: CliCommands<T, CliClientAUX, CliAppAUX, CliTX, CliQue>,
    /// The logging level
    #[arg(long, global = true, action = ArgAction::Set, default_value_t = LogLevel::Info)]
    pub log_level : LogLevel,
}

impl<T, CliClientAUX, CliAppAUX, CliTX, CliQue>
    CliApplicationArgs<T, CliClientAUX, CliAppAUX, CliTX, CliQue>
where
    T: ApplicationInfo,
    CliClientAUX: Subcommand,
    CliAppAUX: Subcommand,
    CliTX: Subcommand,
    CliQue: Subcommand,
{
    pub fn execute_or_help(
        self,
        client_executor: impl FnOnce(
            CliClientCommands<T, CliClientAUX, CliTX, CliQue>,
        ) -> anyhow::Result<()>,
        executor: impl FnOnce(CliAppCommands<T, CliAppAUX>) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        setup_panic!();

        tracing_subscriber::fmt()
        .with_max_level(self.log_level)
        .without_time()
        .with_target(false)
        .try_init()
        .map_err( |e | anyhow!( "Failed to set logger: {}", e.to_string() ))?;

        match self.command {
            CliCommands::Cli(command) => match command {
                CliApplicationCommands::Client(command) => client_executor(command),
                CliApplicationCommands::App(command) => executor(command),
            },
            CliCommands::Completions(command) => {
                let mut cmd = <Self as CommandFactory>::command();
                write_completions(command.shell, &mut cmd, &mut std::io::stdout());

                Ok(())
            }
        }
    }

    pub fn write_completions(shell: Shell, buf: &mut dyn Write) {
        let mut cmd = <Self as CommandFactory>::command();
        write_completions(shell, &mut cmd, buf);
    }
}

impl<
        T: ApplicationInfo,
        CliClientAUX,
        ClientAUX,
        CliAppAUX,
        AppAUX,
        CliTX,
        TX,
        CliQue,
        QUE,
        ERR,
    > TryFrom<CliApplicationCommands<T, CliClientAUX, CliAppAUX, CliTX, CliQue>>
    for ApplicationCommands<ClientAUX, AppAUX, TX, QUE>
where
    CliClientAUX: Subcommand,
    ClientAUX: TryFrom<CliClientAUX, Error = ERR>,
    CliAppAUX: Subcommand,
    AppAUX: TryFrom<CliAppAUX, Error = ERR>,
    CliTX: Subcommand,
    TX: TryFrom<CliTX, Error = ERR>,
    CliQue: Subcommand,
    QUE: TryFrom<CliQue, Error = ERR>,
{
    type Error = ERR;

    fn try_from(
        value: CliApplicationCommands<T, CliClientAUX, CliAppAUX, CliTX, CliQue>,
    ) -> Result<Self, Self::Error> {
        let res = match value {
            CliApplicationCommands::Client(cmd) => Self::Client(cmd.try_into()?),
            CliApplicationCommands::App(cmd) => Self::App(cmd.try_into()?),
        };

        Ok(res)
    }
}

#[derive(Debug, Clone, ::clap::Subcommand)]
pub enum CliCommands<T, CliClientAUX, CliAppAUX, CliTX, CliQue>
where
    T: ApplicationInfo,
    CliClientAUX: Subcommand,
    CliAppAUX: Subcommand,
    CliTX: Subcommand,
    CliQue: Subcommand,
{
    #[command(flatten, value_parser = value_parser!(PhantomData))]
    Cli(CliApplicationCommands<T, CliClientAUX, CliAppAUX, CliTX, CliQue>),
    Completions(CliCompletionArgs),
}

/// If provided, outputs the completion file for given shell
#[derive(Debug, Clone, ::clap::Args)]
pub struct CliCompletionArgs {
    #[arg(required = true)]
    shell: Shell,
}

#[derive(Debug, Clone, ::clap::Subcommand)]
pub enum CliApplicationCommands<T, CliClientAUX, CliAppAUX, CliTX, CliQue>
where
    T: ApplicationInfo,
    CliClientAUX: Subcommand,
    CliAppAUX: Subcommand,
    CliTX: Subcommand,
    CliQue: Subcommand,
{
    #[command(flatten)]
    Client(CliClientCommands<T, CliClientAUX, CliTX, CliQue>),
    #[command(flatten)]
    App(CliAppCommands<T, CliAppAUX>),
}

#[derive(Debug, Clone, ::clap::Subcommand)]
pub enum CliClientCommands<T, CliAUX, CliTX, CliQue>
where
    T: ApplicationInfo,
    CliAUX: Subcommand,
    CliTX: Subcommand,
    CliQue: Subcommand,
{
    #[command(flatten)]
    Aux(CliAUX),
    Tx(CliTxCommand<T, CliTX>),
    Query(CliQueryCommand<CliQue>),
    #[command(subcommand)]
    Keys(CliKeyCommand<T>),
}

impl<T: ApplicationInfo, CliAUX, AUX, CliTX, TX, CliQue, QUE, ERR>
    TryFrom<CliClientCommands<T, CliAUX, CliTX, CliQue>> for ClientCommands<AUX, TX, QUE>
where
    CliAUX: Subcommand,
    AUX: TryFrom<CliAUX, Error = ERR>,
    CliTX: Subcommand,
    TX: TryFrom<CliTX, Error = ERR>,
    CliQue: Subcommand,
    QUE: TryFrom<CliQue, Error = ERR>,
{
    type Error = ERR;

    fn try_from(value: CliClientCommands<T, CliAUX, CliTX, CliQue>) -> Result<Self, Self::Error> {
        let res = match value {
            CliClientCommands::Aux(cmd) => Self::Aux(cmd.try_into()?),
            CliClientCommands::Tx(cmd) => Self::Tx(cmd.try_into()?),
            CliClientCommands::Query(cmd) => Self::Query(cmd.try_into()?),
            CliClientCommands::Keys(cmd) => Self::Keys(cmd.into()),
        };

        Ok(res)
    }
}

#[derive(Debug, Clone, ::clap::Subcommand)]
pub enum CliAppCommands<T: ApplicationInfo, CliAUX: Subcommand> {
    Init(CliInitCommand<T>),
    Run(CliRunCommand<T>),
    #[command(name = "add-genesis-account")]
    GenesisAdd(CliGenesisCommand<T>),
    #[command(flatten)]
    Aux(CliAUX),
}

impl<T, AUX, CliAUX, ERR> TryFrom<CliAppCommands<T, CliAUX>> for AppCommands<AUX>
where
    T: ApplicationInfo,
    CliAUX: Subcommand,
    AUX: TryFrom<CliAUX, Error = ERR>,
{
    type Error = ERR;

    fn try_from(value: CliAppCommands<T, CliAUX>) -> Result<Self, Self::Error> {
        let res = match value {
            CliAppCommands::Init(cmd) => Self::Init(cmd.into()),
            CliAppCommands::Run(cmd) => Self::Run(cmd.into()),
            CliAppCommands::GenesisAdd(cmd) => Self::GenesisAdd(cmd.into()),
            CliAppCommands::Aux(cmd) => Self::Aux(cmd.try_into()?),
        };

        Ok(res)
    }
}
