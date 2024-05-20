use crate::{AccountKeeper, BankKeeper, GenesisState, Keeper, KeeperHooks, Message};
use gears::{
    application::handlers::node::ABCIHandler as NodeABCIHandler,
    error::AppError,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    tendermint::types::{
        proto::validator::ValidatorUpdate, request::end_block::RequestEndBlock,
        request::query::RequestQuery,
    },
    types::context::{block::BlockContext, init::InitContext, query::QueryContext, tx::TxContext},
    types::tx::raw::TxWithRaw,
};

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
        self.keeper.init_genesis(ctx, genesis);
    }

    fn query<DB: Database + Send + Sync>(
        &self,
        _ctx: &QueryContext<DB, SK>,
        _query: RequestQuery,
    ) -> Result<prost::bytes::Bytes, AppError> {
        todo!()
    }

    fn end_block<DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, SK>,
        _request: RequestEndBlock,
    ) -> Vec<ValidatorUpdate> {
        self.keeper.block_validator_updates(ctx)
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
