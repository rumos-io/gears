use auth::Keeper as AuthKeeper;
use bank::Keeper as BankKeeper;
use gears::{baseapp::ante::BaseAnteHandler, config::Config, x::params::Keeper as ParamsKeeper};
use tendermint::proto::abci::RequestQuery;

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
pub struct ABCIHandler {
    bank_abci_handler: bank::ABCIHandler<GaiaStoreKey, GaiaParamsStoreKey>,
    auth_abci_handler: auth::ABCIHandler<GaiaStoreKey, GaiaParamsStoreKey>,
    ante_handler: BaseAnteHandler<
        BankKeeper<GaiaStoreKey, GaiaParamsStoreKey>,
        AuthKeeper<GaiaStoreKey, GaiaParamsStoreKey>,
        GaiaStoreKey,
    >,
}

impl ABCIHandler {
    pub fn new(_cfg: Config<AppConfig>) -> ABCIHandler {
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

        ABCIHandler {
            bank_abci_handler: bank::ABCIHandler::new(bank_keeper.clone()),
            auth_abci_handler: auth::ABCIHandler::new(auth_keeper.clone()),
            ante_handler: BaseAnteHandler::new(bank_keeper, auth_keeper),
        }
    }
}

impl gears::baseapp::ABCIHandler<Message, GaiaStoreKey, GenesisState> for ABCIHandler {
    fn tx<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, GaiaStoreKey>,
        msg: &Message,
    ) -> Result<(), AppError> {
        match msg {
            Message::Bank(msg) => self.bank_abci_handler.tx(ctx, msg),
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

    fn query<DB: Database>(
        &self,
        ctx: &QueryContext<'_, DB, GaiaStoreKey>,
        query: RequestQuery,
    ) -> Result<bytes::Bytes, AppError> {
        if query.path.starts_with("/cosmos.auth") {
            self.auth_abci_handler.query(ctx, query)
        } else if query.path.starts_with("/cosmos.bank") {
            self.bank_abci_handler.query(ctx, query)
        } else {
            Err(AppError::InvalidRequest("query path not found".into()))
        }
    }

    fn run_ante_checks<DB: Database>(
        &self,
        ctx: &mut gears::types::context::context::Context<'_, '_, DB, GaiaStoreKey>,
        tx: &proto_messages::cosmos::tx::v1beta1::tx_raw::TxWithRaw<Message>,
    ) -> Result<(), AppError> {
        self.ante_handler.run(ctx, tx)
    }
}
