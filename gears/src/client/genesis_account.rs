use std::path::PathBuf;

use anyhow::Result;
use clap::{arg, Arg, ArgAction, ArgMatches, Command};

use proto_messages::cosmos::base::v1beta1::SendCoins;
use proto_types::AccAddress;
use tendermint::informal::Genesis;

use crate::{
    baseapp::Genesis as SDKGenesis,
    error::AppError,
    utils::{self, get_default_home_dir},
};

pub fn get_add_genesis_account_command(app_name: &str) -> Command {
    Command::new("add-genesis-account")
        .about(
            "Add a genesis account to genesis.json. The provided account must specify the
account address and a list of initial coins. The list of initial tokens must
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

#[derive(Debug, Clone)]
pub struct GenesisOptions {
    home: PathBuf,
    address: AccAddress,
    coins: SendCoins,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct GenesisOptionParseError(pub String);

impl TryFrom<&ArgMatches> for GenesisOptions {
    type Error = GenesisOptionParseError;

    fn try_from(value: &ArgMatches) -> Result<Self, Self::Error> {
        let address = value
            .get_one::<AccAddress>("address")
            .ok_or(GenesisOptionParseError(
                "address argument is required preventing `None`".to_owned(),
            ))?
            .to_owned();

        let coins = value
            .get_one::<SendCoins>("coin")
            .ok_or(GenesisOptionParseError(
                "coin argument is required preventing `None`".to_owned(),
            ))?
            .to_owned();

        let home = value
            .get_one::<PathBuf>("home")
            .cloned()
            .or(utils::default_home())
            .ok_or(GenesisOptionParseError(
                "Home argument not provided and OS does not provide a default home directory"
                    .to_owned(),
            ))?
            .to_owned();

        Ok(Self {
            home,
            address,
            coins,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GenesisError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Serde(#[from] serde_json::Error),
    #[error("{0}")]
    AppError(#[from] AppError),
}

pub fn genesis_account_add<G: SDKGenesis>(opt: GenesisOptions) -> Result<(), GenesisError> {
    let GenesisOptions {
        home,
        address,
        coins,
    } = opt;

    let mut genesis_file_path = home.clone();
    crate::utils::get_genesis_file_from_home_dir(&mut genesis_file_path);

    let raw_genesis = std::fs::read_to_string(genesis_file_path.clone())?;
    let mut genesis: Genesis<G> = serde_json::from_str(&raw_genesis)?;
    genesis.app_state.add_genesis_account(address, coins)?;
    std::fs::write(genesis_file_path, &serde_json::to_string_pretty(&genesis)?)?;

    Ok(())
}
