use crate::{
    GenesisState, Keeper, Message, QueryDelegationRequest, QueryDelegationResponse,
    QueryParamsResponse, QueryRedelegationRequest, QueryRedelegationResponse,
    QueryUnbondingDelegationResponse, QueryValidatorRequest, QueryValidatorResponse,
};
use gears::{
    context::{block::BlockContext, init::InitContext, query::QueryContext, tx::TxContext},
    core::{errors::CoreError, Protobuf},
    error::AppError,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    tendermint::types::{
        proto::validator::ValidatorUpdate,
        request::{
            begin_block::RequestBeginBlock, end_block::RequestEndBlock, query::RequestQuery,
        },
    },
    x::{
        keepers::{auth::AuthKeeper, bank::StakingBankKeeper, staking::KeeperHooks},
        module::Module,
    },
};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct ABCIHandler<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    AK: AuthKeeper<SK, M>,
    BK: StakingBankKeeper<SK, M>,
    KH: KeeperHooks<SK, AK, M>,
    M: Module,
> {
    keeper: Keeper<SK, PSK, AK, BK, KH, M>,
}

#[derive(Clone)]
pub enum StakingNodeQueryRequest {
    Validator(QueryValidatorRequest),
    Delegation(QueryDelegationRequest),
    Redelegation(QueryRedelegationRequest),
    UnbondingDelegation(QueryDelegationRequest),
    Params,
}

#[derive(Clone, Serialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum StakingNodeQueryResponse {
    Validator(QueryValidatorResponse),
    Delegation(QueryDelegationResponse),
    Redelegation(QueryRedelegationResponse),
    UnbondingDelegation(QueryUnbondingDelegationResponse),
    Params(QueryParamsResponse),
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: StakingBankKeeper<SK, M>,
        KH: KeeperHooks<SK, AK, M>,
        M: Module,
    > ABCIHandler<SK, PSK, AK, BK, KH, M>
{
    pub fn new(keeper: Keeper<SK, PSK, AK, BK, KH, M>) -> Self {
        ABCIHandler { keeper }
    }

    pub fn tx<DB: Database + Sync + Send>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: &Message,
    ) -> Result<(), AppError> {
        match msg {
            Message::CreateValidator(msg) => self.keeper.create_validator(ctx, msg),
            Message::EditValidator(msg) => self.keeper.edit_validator(ctx, msg),
            Message::Delegate(msg) => self.keeper.delegate_cmd_handler(ctx, msg),
            Message::Redelegate(msg) => self.keeper.redelegate_cmd_handler(ctx, msg),
            Message::Undelegate(_msg) => self.keeper.undelegate_cmd_handler(ctx, msg),
        }
    }

    pub fn genesis<DB: Database>(&self, ctx: &mut InitContext<'_, DB, SK>, genesis: GenesisState) {
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
                    .map_err(|e| CoreError::DecodeProtobuf(e.to_string()))?;

                Ok(self.keeper.query_validator(ctx, req).encode_vec().into())
            }
            "/cosmos.staking.v1beta1.Query/Delegation" => {
                let req = QueryDelegationRequest::decode(query.data)
                    .map_err(|e| CoreError::DecodeProtobuf(e.to_string()))?;

                Ok(self.keeper.query_delegation(ctx, req).encode_vec().into())
            }
            "/cosmos.staking.v1beta1.Query/Redelegation" => {
                let req = QueryRedelegationRequest::decode(query.data)
                    .map_err(|e| CoreError::DecodeProtobuf(e.to_string()))?;

                Ok(self
                    .keeper
                    .query_redelegations(ctx, req)
                    .encode_vec()
                    .into())
            }
            "/cosmos.staking.v1beta1.Query/UnbondingDelegation" => {
                let req = QueryDelegationRequest::decode(query.data)
                    .map_err(|e| CoreError::DecodeProtobuf(e.to_string()))?;

                Ok(self
                    .keeper
                    .query_unbonding_delegation(ctx, req)
                    .encode_vec()
                    .into())
            }
            "/cosmos/staking/v1beta1/params" | "/cosmos.staking.v1beta1.Query/Params" => {
                Ok(self.keeper.query_params(ctx).encode_vec().into())
            }
            _ => Err(AppError::InvalidRequest("query path not found".into())),
        }
    }

    pub fn typed_query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: StakingNodeQueryRequest,
    ) -> StakingNodeQueryResponse {
        match query {
            StakingNodeQueryRequest::Validator(req) => {
                StakingNodeQueryResponse::Validator(self.keeper.query_validator(ctx, req))
            }
            StakingNodeQueryRequest::Delegation(req) => {
                StakingNodeQueryResponse::Delegation(self.keeper.query_delegation(ctx, req))
            }
            StakingNodeQueryRequest::Redelegation(req) => {
                StakingNodeQueryResponse::Redelegation(self.keeper.query_redelegations(ctx, req))
            }
            StakingNodeQueryRequest::UnbondingDelegation(req) => {
                StakingNodeQueryResponse::UnbondingDelegation(
                    self.keeper.query_unbonding_delegation(ctx, req),
                )
            }
            StakingNodeQueryRequest::Params => {
                StakingNodeQueryResponse::Params(self.keeper.query_params(ctx))
            }
        }
    }

    pub fn begin_block<DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, SK>,
        _request: RequestBeginBlock,
    ) {
        self.keeper.track_historical_info(ctx);
    }

    pub fn end_block<DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, SK>,
        _request: RequestEndBlock,
    ) -> Vec<ValidatorUpdate> {
        self.keeper.block_validator_updates(ctx)
    }
}
