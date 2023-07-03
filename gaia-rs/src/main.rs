use anyhow::Result;
use auth::cli::query::{get_auth_query_command, run_auth_query_command};
use auth::Keeper as AuthKeeper;
use bank::cli::query::{get_bank_query_command, run_bank_query_command};
use bank::cli::tx::get_bank_tx_command;
use bank::Keeper as BankKeeper;
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use clap_complete::{generate, Generator, Shell};
use client::query::query_command_handler;
use client::tx::tx_command_handler;
use gears::app::run;
use gears::baseapp::cli::get_run_command;
use gears::client::query::get_query_command_v2;
use gears::client::{init::get_init_command, query::get_query_command, tx::get_tx_command};
use gears::x::params::Keeper as ParamsKeeper;
use human_panic::setup_panic;

use gears::{
    baseapp::cli::run_run_command_micro,
    client::{
        init::run_init_command,
        keys::{get_keys_command, run_keys_command},
        query::run_query_command,
        tx::run_tx_command,
    },
};

use crate::genesis::GenesisState;
use crate::handler::Handler;
use crate::message::Message;
use crate::store_keys::{GaiaParamsStoreKey, GaiaStoreKey};

mod client;
mod genesis;
mod handler;
mod message;
mod store_keys;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("GIT_HASH");

fn main() -> Result<()> {
    let params_keeper = ParamsKeeper::new(GaiaStoreKey::Params);

    let auth_keeper = AuthKeeper::new(
        GaiaStoreKey::Auth,
        params_keeper.clone(),
        GaiaParamsStoreKey::Auth,
    );

    let bank_keeper = BankKeeper::new(
        GaiaStoreKey::Bank,
        params_keeper.clone(),
        GaiaParamsStoreKey::Bank,
        auth_keeper.clone(),
    );

    let query_commands = vec![get_bank_query_command(), get_auth_query_command()];
    let tx_commands = vec![get_bank_tx_command()];

    run::<
        GenesisState,
        GaiaStoreKey,
        GaiaParamsStoreKey,
        Message,
        BankKeeper<GaiaStoreKey, GaiaParamsStoreKey>,
        AuthKeeper<GaiaStoreKey, GaiaParamsStoreKey>,
        Handler,
        _,
        _,
    >(
        VERSION,
        GenesisState::default(),
        APP_NAME,
        bank_keeper,
        auth_keeper,
        params_keeper,
        GaiaParamsStoreKey::BaseApp,
        Handler::new(),
        query_commands,
        query_command_handler,
        tx_commands,
        tx_command_handler,
    )
}
