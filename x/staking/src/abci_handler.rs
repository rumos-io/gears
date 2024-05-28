use crate::{AccountKeeper, BankKeeper, GenesisState, Keeper, KeeperHooks, Message};
use gears::{
    error::AppError,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    tendermint::types::{
        proto::validator::ValidatorUpdate, request::end_block::RequestEndBlock,
        request::query::RequestQuery,
    },
    types::context::{block::BlockContext, init::InitContext, query::QueryContext, tx::TxContext},
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

    pub fn tx<DB: Database + Sync + Send>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: &Message,
    ) -> Result<(), AppError> {
        match msg {
            Message::CreateValidator(msg) => self.keeper.create_validator(ctx, msg),
        }
    }

    pub fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        genesis: GenesisState,
    ) {
        self.keeper.init_genesis(ctx, genesis);
    }

    pub fn query<DB: Database + Send + Sync>(
        &self,
        _ctx: &QueryContext<DB, SK>,
        _query: RequestQuery,
    ) -> Result<prost::bytes::Bytes, AppError> {
        todo!()
    }

    pub fn end_block<DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, SK>,
        _request: RequestEndBlock,
    ) -> Vec<ValidatorUpdate> {
        self.keeper.block_validator_updates(ctx)
        // TODO
        // defer telemetry.ModuleMeasureSince(types.ModuleName, time.Now(), telemetry.MetricKeyEndBlocker)
    }
}
