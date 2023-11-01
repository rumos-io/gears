use std::{
    fs::{self},
    path::PathBuf,
};

use anyhow::Result;
use clap::{arg, Arg, ArgAction, ArgMatches, Command};

use proto_messages::cosmos::{base::v1beta1::SendCoins, tx::v1beta1::message::Message};
use proto_types::AccAddress;
use store_crate::StoreKey;
use tendermint_informal::Genesis;

use crate::{
    baseapp::{Genesis as SDKGenesis, Handler},
    config::{ApplicationConfig, Config},
    utils::{get_config_file_from_home_dir, get_default_home_dir, get_genesis_file_from_home_dir},
};

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
            arg!(--home)
                .help(format!(
                    "Directory for config and data [default: {}]",
                    get_default_home_dir(app_name).unwrap_or_default().display()
                ))
                .action(ArgAction::Set)
                .value_parser(clap::value_parser!(PathBuf)),
        )
}

pub fn run_add_genesis_account_command<
    G: SDKGenesis,
    H: Handler<M, SK, G>,
    HandlerBuilder,
    SK: StoreKey,
    M: Message,
    AC: ApplicationConfig,
>(
    sub_matches: &ArgMatches,
    app_name: &str,
    handler_builder: HandlerBuilder,
) -> Result<()>
where
    HandlerBuilder: FnOnce(Config<AC>) -> H,
{
    let default_home_directory = get_default_home_dir(app_name);

    let home = sub_matches
        .get_one::<PathBuf>("home")
        .or(default_home_directory.as_ref())
        .unwrap_or_else(|| {
            println!("Home argument not provided and OS does not provide a default home directory");
            std::process::exit(1)
        });

    let address = sub_matches
        .get_one::<AccAddress>("address")
        .expect("address argument is required preventing `None`")
        .to_owned();

    let coins = sub_matches
        .get_one::<SendCoins>("coin")
        .expect("coin argument is required preventing `None`")
        .to_owned();

    let mut cfg_file_path = home.clone();
    get_config_file_from_home_dir(&mut cfg_file_path);

    let config: Config<AC> = Config::from_file(cfg_file_path).unwrap_or_else(|err| {
        tracing::error!("Error reading config file: {:?}", err);
        std::process::exit(1)
    });

    let handler = handler_builder(config);

    let mut genesis_file_path = home.clone();
    get_genesis_file_from_home_dir(&mut genesis_file_path);

    let raw_genesis = fs::read_to_string(genesis_file_path.clone())?;
    let mut genesis: Genesis<G> = serde_json::from_str(&raw_genesis)?;
    handler.handle_add_genesis_account(&mut genesis.app_state, address, coins)?;
    std::fs::write(genesis_file_path, serde_json::to_string_pretty(&genesis)?)?;
    Ok(())
}
