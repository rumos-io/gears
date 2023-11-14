#![warn(rust_2018_idioms)]

use anyhow::Result;
use auth::Keeper as AuthKeeper;
use bank::Keeper as BankKeeper;
use client::query_command_handler;
use client::tx_command_handler;
use gears::x::params::Keeper as ParamsKeeper;
use gears::Application;
use rest::get_router;

use crate::handler::Handler;
use crate::store_keys::{GaiaParamsStoreKey, GaiaStoreKey};

mod client;
mod config;
mod genesis;
mod handler;
mod message;
mod rest;
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

    let app: Application<_, _, _, _, _, _, _, _, _, _, _, _, _> = Application::new(
        APP_NAME,
        VERSION,
        bank_keeper,
        auth_keeper,
        params_keeper,
        GaiaParamsStoreKey::BaseApp,
        Handler::new,
        query_command_handler,
        tx_command_handler,
        get_router(),
    );
    app.run_command()
}
