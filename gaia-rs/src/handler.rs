use gears::{
    config::Config,
    types::context::{context::Context, query_context::QueryContext},
    x::params::Keeper as ParamsKeeper,
};
use proto_messages::cosmos::base::v1beta1::SendCoins;
use proto_types::AccAddress;
use tendermint_proto::abci::{RequestBeginBlock, RequestQuery};

use database::Database;
use gears::error::AppError;

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
        ctx: &mut Context<DB, GaiaStoreKey>,
        msg: &Message,
    ) -> Result<(), AppError> {
        match msg {
            Message::Bank(msg) => self.bank_handler.handle(ctx, msg),
        }
    }

    fn handle_init_genesis<DB: Database>(
        &self,
        ctx: &mut Context<DB, GaiaStoreKey>,
        genesis: GenesisState,
    ) {
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

    // TODO: move this into the SDK
    fn handle_add_genesis_account(
        &self,
        genesis_state: &mut GenesisState,
        address: AccAddress,
        coins: SendCoins,
    ) -> Result<(), AppError> {
        self.auth_handler
            .handle_add_genesis_account(&mut genesis_state.auth, address.clone())?;
        self.bank_handler
            .handle_add_genesis_account(&mut genesis_state.bank, address, coins);

        Ok(())
    }

    fn handle_begin_block<DB: Database>(
        &self,
        _ctx: &mut Context<DB, GaiaStoreKey>,
        _request: RequestBeginBlock,
    ) {
        // do nothing
    }
}
