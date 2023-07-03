use anyhow::Result;
use auth::cli::query::get_auth_query_command;
use auth::Keeper as AuthKeeper;
use bank::cli::query::get_bank_query_command;
use bank::cli::tx::get_bank_tx_command;
use bank::Keeper as BankKeeper;
use client::query_command_handler;
use client::tx_command_handler;
use gears::app::run;
use gears::x::params::Keeper as ParamsKeeper;

use crate::genesis::GenesisState;
use crate::handler::Handler;
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

    run(
        APP_NAME,
        VERSION,
        GenesisState::default(),
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
