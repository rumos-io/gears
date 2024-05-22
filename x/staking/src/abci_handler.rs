use crate::{
    AccountKeeper, BankKeeper, GenesisState, Keeper, KeeperHooks, Message, QueryDelegationRequest,
};
use gears::{
    application::handlers::node::ABCIHandler as NodeABCIHandler,
    core::errors::Error,
    error::{AppError, IBC_ENCODE_UNWRAP},
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    tendermint::types::{
        proto::{validator::ValidatorUpdate, Protobuf},
        request::{end_block::RequestEndBlock, query::RequestQuery},
    },
    types::{
        context::{block::BlockContext, init::InitContext, query::QueryContext, tx::TxContext},
        tx::raw::TxWithRaw,
    },
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
        ctx: &mut TxContext<'_, DB, SK>,
        msg: &Message,
    ) -> Result<(), AppError> {
        match msg {
            Message::CreateValidator(msg) => self.keeper.create_validator(ctx, msg),
            Message::Delegate(msg) => self.keeper.delegate_cmd_handler(ctx, msg),
        }
    }

    fn init_genesis<DB: Database>(&self, ctx: &mut InitContext<'_, DB, SK>, genesis: GenesisState) {
        self.keeper.init_genesis(ctx, genesis);
    }

    fn query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: RequestQuery,
    ) -> Result<prost::bytes::Bytes, AppError> {
        match query.path.as_str() {
            "/cosmos.staking.v1beta1.Query/Delegation" => {
                let req = QueryDelegationRequest::decode(query.data)
                    .map_err(|e| Error::DecodeProtobuf(e.to_string()))?;

                Ok(self
                    .keeper
                    .query_delegation(ctx, req)?
                    .encode_vec()
                    .expect(IBC_ENCODE_UNWRAP)
                    .into()) // TODO:IBC
            }
            _ => Err(AppError::InvalidRequest("query path not found".into())),
        }
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
