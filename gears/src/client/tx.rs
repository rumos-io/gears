use std::path::PathBuf;

use clap::{arg, value_parser, ArgAction, Command, Subcommand};

use crate::utils::get_default_home_dir;

pub fn get_tx_command<TxSubcommand: Subcommand>(app_name: &str) -> Command {
    let cli = Command::new("tx")
        .about("Transaction subcommands")
        .subcommand_required(true)
        .arg(
            arg!(--node)
                .help("<host>:<port> to Tendermint RPC interface for this chain")
                .default_value("http://localhost:26657")
                .action(ArgAction::Set)
                .global(true),
        )
        .arg(
            arg!(--home)
                .help(format!(
                    "Directory for config and data [default: {}]",
                    get_default_home_dir(app_name)
                        .unwrap_or_default()
                        .display()
                        .to_string()
                ))
                .action(ArgAction::Set)
                .value_parser(value_parser!(PathBuf)),
        );

    TxSubcommand::augment_subcommands(cli)
}
