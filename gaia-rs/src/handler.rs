use gears::x::params::Keeper as ParamsKeeper;
use std::str::FromStr;
use tendermint_proto::abci::RequestQuery;
use tracing::error;

use database::Database;
use gears::{
    error::AppError,
    types::context_v2::{Context, QueryContext},
};

use crate::{
    genesis::GenesisState,
    message::Message,
    store_keys::{GaiaParamsStoreKey, GaiaStoreKey},
};

#[derive(Debug, Clone)]
pub struct Handler {
    bank_handler: bank::Handler<GaiaStoreKey, GaiaParamsStoreKey>,
    auth_handler: auth::Handler<GaiaStoreKey, GaiaParamsStoreKey>,
}

impl Handler {
    pub fn new() -> Handler {
        let params_keeper = ParamsKeeper::new(GaiaStoreKey::Params);

        let auth_keeper = auth::Keeper::new(
            GaiaStoreKey::Auth,
            params_keeper.clone(),
            GaiaParamsStoreKey::Auth,
        );

        let bank_keeper = bank::Keeper::new(
            GaiaStoreKey::Bank,
            params_keeper,
            GaiaParamsStoreKey::Bank,
            auth_keeper.clone(),
        );

        Handler {
            bank_handler: bank::Handler::new(bank_keeper),
            auth_handler: auth::Handler::new(auth_keeper),
        }
    }
}

impl gears::baseapp::Handler<Message, GaiaStoreKey> for Handler {
    fn handle_tx<DB: Database>(
        &self,
        ctx: &mut Context<DB, GaiaStoreKey>,
        msg: &Message,
    ) -> Result<(), AppError> {
        match msg {
            Message::Bank(msg) => self.bank_handler.handle(ctx, msg),
        }
    }

    //TODO: make BaseApp generic over the genesis struct then pass a struct here rather than raw bytes?
    fn handle_init_genesis<DB: Database>(
        &self,
        ctx: &mut Context<DB, GaiaStoreKey>,
        raw: bytes::Bytes,
    ) {
        let genesis = String::from_utf8(raw.into())
            .map_err(|e| AppError::Genesis(e.to_string()))
            .and_then(|f| GenesisState::from_str(&f))
            .unwrap_or_else(|e| {
                error!(
                    "Invalid genesis provided by Tendermint.\n{}\nTerminating process",
                    e.to_string()
                );
                std::process::exit(1)
            });

        self.bank_handler.init_genesis(ctx, genesis.bank);
        self.auth_handler.init_genesis(ctx, genesis.auth);
    }

    fn handle_query<DB: Database>(
        &self,
        ctx: &QueryContext<DB, GaiaStoreKey>,
        query: RequestQuery,
    ) -> Result<bytes::Bytes, AppError> {
        if query.path.starts_with("/cosmos.auth") {
            self.auth_handler.handle_query(ctx, query)
        } else if query.path.starts_with("/cosmos.bank") {
            self.bank_handler.handle_query(ctx, query)
        } else {
            Err(AppError::InvalidRequest("query path not found".into()))
        }
    }
}
