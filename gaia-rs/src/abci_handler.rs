use crate::{
    config::AppConfig,
    genesis::GenesisState,
    message::Message,
    modules::{modules_map, GaiaModuleKey},
    store_keys::{GaiaParamsStoreKey, GaiaStoreKey},
    GaiaNodeQueryRequest, GaiaNodeQueryResponse,
};
use gears::config::Config;
use gears::context::init::InitContext;
use gears::context::query::QueryContext;
use gears::store::database::Database;
use gears::tendermint::types::request::query::RequestQuery;
use gears::types::tx::raw::TxWithRaw;
use gears::{application::handlers::node::ABCIHandler, x::ante::BaseAnteHandler};
use gears::{context::tx::TxContext, error::AppError, x::ante::DefaultSignGasConsumer};

#[derive(Debug, Clone)]
pub struct GaiaABCIHandler {
    bank_abci_handler: bank::ABCIHandler<
        GaiaStoreKey,
        GaiaParamsStoreKey,
        auth::Keeper<GaiaStoreKey, GaiaParamsStoreKey, GaiaModuleKey>,
        GaiaModuleKey,
    >,
    auth_abci_handler: auth::ABCIHandler<GaiaStoreKey, GaiaParamsStoreKey, GaiaModuleKey>,
    ibc_abci_handler: ibc_rs::ABCIHandler<GaiaStoreKey, GaiaParamsStoreKey>,
    ante_handler: BaseAnteHandler<
        bank::Keeper<
            GaiaStoreKey,
            GaiaParamsStoreKey,
            auth::Keeper<GaiaStoreKey, GaiaParamsStoreKey, GaiaModuleKey>,
            GaiaModuleKey,
        >,
        auth::Keeper<GaiaStoreKey, GaiaParamsStoreKey, GaiaModuleKey>,
        GaiaStoreKey,
        DefaultSignGasConsumer,
        GaiaModuleKey,
    >,
}

impl GaiaABCIHandler {
    pub fn new(_cfg: Config<AppConfig>) -> GaiaABCIHandler {
        let modules_map = modules_map();
        let auth_keeper = auth::Keeper::new(
            GaiaStoreKey::Auth,
            GaiaParamsStoreKey::Auth,
            modules_map,
            GaiaModuleKey::FeeCollector,
        );

        let bank_keeper = bank::Keeper::new(
            GaiaStoreKey::Bank,
            GaiaParamsStoreKey::Bank,
            auth_keeper.clone(),
        );

        let ibc_keeper = ibc_rs::keeper::Keeper::new(GaiaStoreKey::IBC, GaiaParamsStoreKey::IBC);

        GaiaABCIHandler {
            bank_abci_handler: bank::ABCIHandler::new(bank_keeper.clone()),
            auth_abci_handler: auth::ABCIHandler::new(auth_keeper.clone()),
            ibc_abci_handler: ibc_rs::ABCIHandler::new(ibc_keeper.clone()),
            ante_handler: BaseAnteHandler::new(
                auth_keeper,
                bank_keeper,
                DefaultSignGasConsumer,
                GaiaModuleKey::FeeCollector,
            ),
        }
    }
}

impl ABCIHandler for GaiaABCIHandler {
    type Message = Message;
    type Genesis = GenesisState;
    type StoreKey = GaiaStoreKey;

    type QReq = GaiaNodeQueryRequest;
    type QRes = GaiaNodeQueryResponse;

    fn tx<DB: Database + Sync + Send>(
        &self,
        ctx: &mut TxContext<'_, DB, GaiaStoreKey>,
        msg: &Message,
    ) -> Result<(), AppError> {
        match msg {
            Message::Bank(msg) => self.bank_abci_handler.tx(ctx, msg),
            Message::IBC(msg) => self.ibc_abci_handler.tx(ctx, msg.clone()),
        }
    }

    fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, GaiaStoreKey>,
        genesis: GenesisState,
    ) {
        self.bank_abci_handler.genesis(ctx, genesis.bank);
        self.auth_abci_handler.genesis(ctx, genesis.auth);
        self.ibc_abci_handler.genesis(ctx, genesis.ibc);
    }

    fn query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<DB, GaiaStoreKey>,
        query: RequestQuery,
    ) -> Result<bytes::Bytes, AppError> {
        if query.path.starts_with("/cosmos.auth") {
            self.auth_abci_handler.query(ctx, query)
        } else if query.path.starts_with("/cosmos.bank") {
            self.bank_abci_handler.query(ctx, query)
        } else if query.path.starts_with("/ibc.core.client") {
            self.ibc_abci_handler.query(ctx, query)
        } else {
            Err(AppError::InvalidRequest("query path not found".into()))
        }
    }

    fn run_ante_checks<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, GaiaStoreKey>,
        tx: &TxWithRaw<Message>,
    ) -> Result<(), AppError> {
        self.ante_handler.run(ctx, tx)
    }

    fn typed_query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<DB, GaiaStoreKey>,
        query: GaiaNodeQueryRequest,
    ) -> GaiaNodeQueryResponse {
        match query {
            GaiaNodeQueryRequest::Bank(req) => {
                GaiaNodeQueryResponse::Bank(self.bank_abci_handler.typed_query(ctx, req))
            }
            GaiaNodeQueryRequest::Auth(req) => {
                GaiaNodeQueryResponse::Auth(self.auth_abci_handler.typed_query(ctx, req))
            }
        }
    }
}
