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
use gears::{config::Config, params::Keeper as ParamsKeeper};
use gears::{error::AppError, types::context::tx::TxContext, x::ante::DefaultSignGasConsumer};

#[derive(Debug, Clone)]
pub struct GaiaABCIHandler {
    bank_abci_handler: bank::ABCIHandler<
        GaiaStoreKey,
        GaiaParamsStoreKey,
        auth::Keeper<GaiaStoreKey, GaiaParamsStoreKey>,
    >,
    auth_abci_handler: auth::ABCIHandler<GaiaStoreKey, GaiaParamsStoreKey>,
    // ibc_handler: ibc::handler::Handler<GaiaStoreKey, GaiaParamsStoreKey>,
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

        // let ibc_tx_keeper = ibc::keeper::tx::TxKeeper::new(
        //     GaiaStoreKey::Bank,
        //     params_keeper.clone(),
        //     GaiaParamsStoreKey::Bank,
        // );

        // let ibc_query_keeper = ibc::keeper::query::QueryKeeper::new(
        //     GaiaStoreKey::Bank,
        //     params_keeper,
        //     GaiaParamsStoreKey::Bank,
        // );

        GaiaABCIHandler {
            bank_abci_handler: bank::ABCIHandler::new(bank_keeper.clone()),
            auth_abci_handler: auth::ABCIHandler::new(auth_keeper.clone()),
            // ibc_handler: ibc::handler::Handler::new(ibc_tx_keeper, ibc_query_keeper),
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
            // Message::Ibc(msg) => self
            //     .ibc_handler
            //     .tx(ctx, msg.clone())
            //     .map_err(|e| AppError::IBC(e.to_string())),
        }
    }

    fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, GaiaStoreKey>,
        genesis: GenesisState,
    ) {
        self.bank_abci_handler.genesis(ctx, genesis.bank);
        self.auth_abci_handler.genesis(ctx, genesis.auth);
    }

    fn query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<'_, DB, GaiaStoreKey>,
        query: RequestQuery,
    ) -> Result<bytes::Bytes, AppError> {
        if query.path.starts_with("/cosmos.auth") {
            self.auth_abci_handler.query(ctx, query)
        } else if query.path.starts_with("/cosmos.bank") {
            self.bank_abci_handler.query(ctx, query)
        }
        // else if query.path.starts_with("/ibc.core.client") {
        //     self.ibc_handler
        //         .query(ctx, query)
        //         .map_err(|e| AppError::Query(e.to_string()))
        //}
        else {
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
