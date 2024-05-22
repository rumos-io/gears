use crate::{
    config::AppConfig,
    genesis::GenesisState,
    message::Message,
    store_keys::{GaiaParamsStoreKey, GaiaStoreKey},
};
use gears::store::database::Database;
use gears::tendermint::types::request::query::RequestQuery;
use gears::types::context::init::InitContext;
use gears::types::context::query::QueryContext;
use gears::types::tx::raw::TxWithRaw;
use gears::{application::handlers::node::ABCIHandler, x::ante::BaseAnteHandler};
use gears::{config::Config, params::keeper::ParamsKeeper};
use gears::{error::AppError, types::context::tx::TxContext, x::ante::DefaultSignGasConsumer};

#[derive(Debug, Clone)]
pub struct GaiaABCIHandler {
    bank_abci_handler: bank::ABCIHandler<
        GaiaStoreKey,
        GaiaParamsStoreKey,
        auth::Keeper<GaiaStoreKey, GaiaParamsStoreKey>,
    >,
    auth_abci_handler: auth::ABCIHandler<GaiaStoreKey, GaiaParamsStoreKey>,
    ibc_abci_handler: ibc_rs::ABCIHandler<GaiaStoreKey, GaiaParamsStoreKey>,
    ante_handler: BaseAnteHandler<
        bank::Keeper<
            GaiaStoreKey,
            GaiaParamsStoreKey,
            auth::Keeper<GaiaStoreKey, GaiaParamsStoreKey>,
        >,
        auth::Keeper<GaiaStoreKey, GaiaParamsStoreKey>,
        GaiaStoreKey,
        DefaultSignGasConsumer,
    >,
}

impl GaiaABCIHandler {
    pub fn new(_cfg: Config<AppConfig>) -> GaiaABCIHandler {
        let params_keeper = ParamsKeeper::new(GaiaStoreKey::Params);

        let auth_keeper = auth::Keeper::new(
            GaiaStoreKey::Auth,
            params_keeper.clone(),
            GaiaParamsStoreKey::Auth,
        );

        let bank_keeper = bank::Keeper::new(
            GaiaStoreKey::Bank,
            params_keeper.clone(),
            GaiaParamsStoreKey::Bank,
            auth_keeper.clone(),
        );

        let ibc_keeper = ibc_rs::keeper::Keeper::new(
            GaiaStoreKey::IBC,
            params_keeper.clone(),
            GaiaParamsStoreKey::IBC,
        );

        GaiaABCIHandler {
            bank_abci_handler: bank::ABCIHandler::new(bank_keeper.clone()),
            auth_abci_handler: auth::ABCIHandler::new(auth_keeper.clone()),
            ibc_abci_handler: ibc_rs::ABCIHandler::new(ibc_keeper.clone()),
            ante_handler: BaseAnteHandler::new(auth_keeper, bank_keeper, DefaultSignGasConsumer),
        }
    }
}

impl ABCIHandler<Message, GaiaStoreKey, GenesisState> for GaiaABCIHandler {
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
}
