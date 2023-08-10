use std::{
    fs::{self},
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use clap::{arg, Arg, ArgAction, ArgMatches, Command};

use proto_messages::cosmos::{base::v1beta1::SendCoins, tx::v1beta1::Message};
use proto_types::AccAddress;
use serde::{de::DeserializeOwned, Serialize};
use store_crate::StoreKey;
use tendermint_informal::Genesis;

use crate::{baseapp::Handler, utils::get_default_genesis_file};

pub fn get_add_genesis_account_command(app_name: &str) -> Command {
    Command::new("add-genesis-account")
        .about(
            "Add a genesis account to genesis.json. The provided account must specify
        the account address and a list of initial coins. The list of initial tokens must
        contain valid denominations.",
        )
        .arg(
            Arg::new("address")
                .required(true)
                .value_parser(clap::value_parser!(AccAddress)),
        )
        .arg(
            Arg::new("coin")
                .required(true)
                .value_parser(clap::value_parser!(SendCoins)),
        )
        .arg(
            arg!(--file)
                .help(format!(
                    "Path to genesis file [default: {}]",
                    get_default_genesis_file(app_name)
                        .unwrap_or_default()
                        .display()
                        .to_string()
                ))
                .action(ArgAction::Set)
                .value_parser(clap::value_parser!(PathBuf)),
        )
}

pub fn run_add_genesis_account_command<
    G: DeserializeOwned + Serialize,
    H: Handler<M, SK, G>,
    SK: StoreKey,
    M: Message,
>(
    sub_matches: &ArgMatches,
    app_name: &str,
    handler: H,
) -> Result<()> {
    let address = sub_matches
        .get_one::<AccAddress>("address")
        .expect("address argument is required preventing `None`")
        .to_owned();

    let coins = sub_matches
        .get_one::<SendCoins>("coin")
        .expect("coin argument is required preventing `None`")
        .to_owned();

    let default_genesis_file = get_default_genesis_file(app_name);

    let genesis_file_path = sub_matches
        .get_one::<PathBuf>("file")
        .or(default_genesis_file.as_ref())
        .ok_or(anyhow!(
            "Genesis file path not provided and OS does not provide a default home directory"
        ))?;

    let raw_genesis = fs::read_to_string(genesis_file_path)?;
    let mut genesis: Genesis<G> = serde_json::from_str(&raw_genesis)?;
    handler.handle_add_genesis_account(&mut genesis.app_state, address, coins)?;
    std::fs::write(genesis_file_path, &serde_json::to_string_pretty(&genesis)?)?;
    Ok(())
}
