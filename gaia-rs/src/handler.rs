use gears::{config::Config, x::params::Keeper as ParamsKeeper};
use tendermint_proto::abci::{RequestBeginBlock, RequestQuery};

use database::Database;
use gears::error::AppError;
use gears::types::context::init_context::InitContext;
use gears::types::context::query_context::QueryContext;
use gears::types::context::tx_context::TxContext;

use crate::{
    config::AppConfig,
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
    pub fn new(_cfg: Config<AppConfig>) -> Handler {
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

impl gears::baseapp::Handler<Message, GaiaStoreKey, GenesisState> for Handler {
    fn handle_tx<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, GaiaStoreKey>,
        msg: &Message,
    ) -> Result<(), AppError> {
        match msg {
            Message::Bank(msg) => self.bank_handler.handle(ctx, msg),
        }
    }

    fn handle_init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, GaiaStoreKey>,
        genesis: GenesisState,
    ) {
        self.bank_handler.init_genesis(ctx, genesis.bank);
        self.auth_handler.init_genesis(ctx, genesis.auth);
    }

    fn handle_query<DB: Database>(
        &self,
        ctx: &QueryContext<'_, DB, GaiaStoreKey>,
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

    fn handle_begin_block<DB: Database>(
        &self,
        _ctx: &mut TxContext<'_, DB, GaiaStoreKey>,
        _request: RequestBeginBlock,
    ) {
        // do nothing
    }
}
