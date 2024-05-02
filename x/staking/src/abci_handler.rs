use crate::{AccountKeeper, BankKeeper, GenesisState, Keeper, KeeperHooks, Message};
use gears::application::handlers::node::ABCIHandler as NodeABCIHandler;
use gears::store::database::Database;
use gears::store::StoreKey;
use gears::tendermint::types::proto::validator::ValidatorUpdate;
use gears::tendermint::types::request::end_block::RequestEndBlock;
use gears::tendermint::types::request::query::RequestQuery;
use gears::types::context::init::InitContext;
use gears::types::context::query::QueryContext;
use gears::types::context::tx::TxContext;
use gears::types::context::TransactionalContext;
use gears::types::tx::raw::TxWithRaw;
use gears::{error::AppError, params::ParamsSubspaceKey};

#[derive(Debug, Clone)]
pub struct ABCIHandler<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    AK: AccountKeeper<SK>,
    BK: BankKeeper<SK>,
    KH: KeeperHooks<SK>,
> {
    keeper: Keeper<SK, PSK, AK, BK, KH>,
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AccountKeeper<SK>,
        BK: BankKeeper<SK>,
        KH: KeeperHooks<SK>,
    > ABCIHandler<SK, PSK, AK, BK, KH>
{
    pub fn new(keeper: Keeper<SK, PSK, AK, BK, KH>) -> Self {
        ABCIHandler { keeper }
    }
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AccountKeeper<SK>,
        BK: BankKeeper<SK>,
        KH: KeeperHooks<SK>,
    > NodeABCIHandler<Message, SK, GenesisState> for ABCIHandler<SK, PSK, AK, BK, KH>
{
    fn tx<DB: Database + Sync + Send>(
        &self,
        _ctx: &mut TxContext<'_, DB, SK>,
        _msg: &Message,
    ) -> Result<(), AppError> {
        todo!()
    }

    fn init_genesis<DB: Database>(&self, ctx: &mut InitContext<'_, DB, SK>, genesis: GenesisState) {
        self.keeper
            .init_genesis(ctx, genesis)
            .expect("Cannot perform genesis actions.")
    }

    fn query<DB: Database + Send + Sync>(
        &self,
        _ctx: &QueryContext<'_, DB, SK>,
        _query: RequestQuery,
    ) -> Result<prost::bytes::Bytes, AppError> {
        todo!()
    }

    fn end_block<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        _request: RequestEndBlock,
    ) -> Vec<ValidatorUpdate> {
        self.keeper
            .block_validator_updates(ctx)
            .expect("Unknown error")
        // TODO
        // defer telemetry.ModuleMeasureSince(types.ModuleName, time.Now(), telemetry.MetricKeyEndBlocker)
    }

    fn run_ante_checks<DB: Database>(
        &self,
        _ctx: &mut TxContext<'_, DB, SK>,
        _tx: &TxWithRaw<Message>,
    ) -> Result<(), AppError> {
        unreachable!()
    }
}
