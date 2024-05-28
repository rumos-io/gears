use crate::{
    AccountKeeper, BankKeeper, GenesisState, Keeper, KeeperHooks, Message, QueryDelegationRequest,
    QueryValidatorRequest,
};
use gears::{
    core::errors::Error,
    error::{AppError, IBC_ENCODE_UNWRAP},
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    tendermint::types::{
        proto::{validator::ValidatorUpdate, Protobuf},
        request::{end_block::RequestEndBlock, query::RequestQuery},
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
            Message::Delegate(msg) => self.keeper.delegate_cmd_handler(ctx, msg),
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
        ctx: &QueryContext<DB, SK>,
        query: RequestQuery,
    ) -> Result<prost::bytes::Bytes, AppError> {
        match query.path.as_str() {
            "/cosmos.staking.v1beta1.Query/Validator" => {
                let req = QueryValidatorRequest::decode(query.data)
                    .map_err(|e| Error::DecodeProtobuf(e.to_string()))?;

                Ok(self
                    .keeper
                    .query_validator(ctx, req)?
                    .encode_vec()
                    .expect(IBC_ENCODE_UNWRAP)
                    .into()) // TODO:IBC
            }
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
