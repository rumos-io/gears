use crate::{
    config::AppConfig,
    genesis::GenesisState,
    message::Message,
    modules::GaiaModules,
    store_keys::{GaiaParamsStoreKey, GaiaStoreKey},
    GaiaNodeQueryRequest, GaiaNodeQueryResponse,
};
use anyhow::anyhow;
use gears::config::Config;
use gears::context::init::InitContext;
use gears::context::query::QueryContext;
use gears::store::database::Database;
use gears::tendermint::types::request::query::RequestQuery;
use gears::types::tx::raw::TxWithRaw;
use gears::{application::handlers::node::ABCIHandler, x::ante::BaseAnteHandler};
use gears::{context::tx::TxContext, x::ante::DefaultSignGasConsumer};

#[derive(Debug, Clone)]
pub struct GaiaABCIHandler {
    bank_abci_handler: bank::ABCIHandler<
        GaiaStoreKey,
        GaiaParamsStoreKey,
        auth::Keeper<GaiaStoreKey, GaiaParamsStoreKey, GaiaModules>,
        GaiaModules,
    >,
    auth_abci_handler: auth::ABCIHandler<GaiaStoreKey, GaiaParamsStoreKey, GaiaModules>,
    staking_abci_handler: staking::ABCIHandler<
        GaiaStoreKey,
        GaiaParamsStoreKey,
        auth::Keeper<GaiaStoreKey, GaiaParamsStoreKey, GaiaModules>,
        bank::Keeper<
            GaiaStoreKey,
            GaiaParamsStoreKey,
            auth::Keeper<GaiaStoreKey, GaiaParamsStoreKey, GaiaModules>,
            GaiaModules,
        >,
        staking::MockHookKeeper<
            GaiaStoreKey,
            auth::Keeper<GaiaStoreKey, GaiaParamsStoreKey, GaiaModules>,
            GaiaModules,
        >,
        GaiaModules,
    >,
    ibc_abci_handler: ibc_rs::ABCIHandler<GaiaStoreKey, GaiaParamsStoreKey>,
    ante_handler: BaseAnteHandler<
        bank::Keeper<
            GaiaStoreKey,
            GaiaParamsStoreKey,
            auth::Keeper<GaiaStoreKey, GaiaParamsStoreKey, GaiaModules>,
            GaiaModules,
        >,
        auth::Keeper<GaiaStoreKey, GaiaParamsStoreKey, GaiaModules>,
        GaiaStoreKey,
        DefaultSignGasConsumer,
        GaiaModules,
    >,
}

impl GaiaABCIHandler {
    pub fn new(_cfg: Config<AppConfig>) -> GaiaABCIHandler {
        let auth_keeper = auth::Keeper::new(
            GaiaStoreKey::Auth,
            GaiaParamsStoreKey::Auth,
            GaiaModules::FeeCollector,
        );

        let bank_keeper = bank::Keeper::new(
            GaiaStoreKey::Bank,
            GaiaParamsStoreKey::Bank,
            auth_keeper.clone(),
        );

        let staking_keeper = staking::Keeper::new(
            GaiaStoreKey::Staking,
            GaiaParamsStoreKey::Staking,
            auth_keeper.clone(),
            bank_keeper.clone(),
            // NOTE: The variant with instance should have less performance.
            // Some(staking::MockHookKeeper::new()),
            // The compiler require type for option `None`
            None::<
                staking::MockHookKeeper<
                    GaiaStoreKey,
                    auth::Keeper<GaiaStoreKey, GaiaParamsStoreKey, GaiaModules>,
                    GaiaModules,
                >,
            >,
            GaiaModules::BondedPool,
            GaiaModules::NotBondedPool,
        );

        let ibc_keeper = ibc_rs::keeper::Keeper::new(GaiaStoreKey::IBC, GaiaParamsStoreKey::IBC);

        GaiaABCIHandler {
            bank_abci_handler: bank::ABCIHandler::new(bank_keeper.clone()),
            auth_abci_handler: auth::ABCIHandler::new(auth_keeper.clone()),
            staking_abci_handler: staking::ABCIHandler::new(staking_keeper),
            ibc_abci_handler: ibc_rs::ABCIHandler::new(ibc_keeper.clone()),
            ante_handler: BaseAnteHandler::new(
                auth_keeper,
                bank_keeper,
                DefaultSignGasConsumer,
                GaiaModules::FeeCollector,
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
    ) -> Result<(), anyhow::Error> {
        match msg {
            Message::Bank(msg) => self.bank_abci_handler.tx(ctx, msg),
            Message::Staking(msg) => self.staking_abci_handler.tx(ctx, msg),
            Message::IBC(msg) => self.ibc_abci_handler.tx(ctx, msg.clone()),
        }
    }

    fn begin_block<'a, DB: Database>(
        &self,
        _ctx: &mut gears::context::block::BlockContext<'_, DB, Self::StoreKey>,
        _request: gears::tendermint::types::request::begin_block::RequestBeginBlock,
    ) {
        //self.staking_abci_handler.begin_block(ctx, request);
    }

    fn end_block<'a, DB: Database>(
        &self,
        _ctx: &mut gears::context::block::BlockContext<'_, DB, Self::StoreKey>,
        _request: gears::tendermint::types::request::end_block::RequestEndBlock,
    ) -> Vec<gears::tendermint::types::proto::validator::ValidatorUpdate> {
        //self.staking_abci_handler.end_block(ctx, request)
        vec![]
    }

    fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, GaiaStoreKey>,
        genesis: GenesisState,
    ) {
        self.bank_abci_handler.genesis(ctx, genesis.bank);
        self.staking_abci_handler.genesis(ctx, genesis.staking);
        self.ibc_abci_handler.genesis(ctx, genesis.ibc);
        self.auth_abci_handler.genesis(ctx, genesis.auth);
    }

    fn query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<DB, GaiaStoreKey>,
        query: RequestQuery,
    ) -> Result<bytes::Bytes, anyhow::Error> {
        if query.path.starts_with("/cosmos.auth") {
            Ok(self.auth_abci_handler.query(ctx, query)?)
        } else if query.path.starts_with("/cosmos.bank") {
            Ok(self.bank_abci_handler.query(ctx, query)?)
        } else if query.path.starts_with("/cosmos.staking") {
            Ok(self.staking_abci_handler.query(ctx, query)?)
        } else if query.path.starts_with("/ibc.core.client") {
            Ok(self.ibc_abci_handler.query(ctx, query)?)
        } else {
            Err(anyhow!("query path not found"))
        }
    }

    fn run_ante_checks<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, GaiaStoreKey>,
        tx: &TxWithRaw<Message>,
    ) -> Result<(), anyhow::Error> {
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
            GaiaNodeQueryRequest::Staking(req) => {
                GaiaNodeQueryResponse::Staking(self.staking_abci_handler.typed_query(ctx, req))
            }
        }
    }
}
