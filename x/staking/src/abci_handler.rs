use crate::{
    error::StakingTxError, GenesisState, Keeper, Message, QueryDelegationRequest,
    QueryDelegationResponse, QueryParamsResponse, QueryRedelegationRequest,
    QueryRedelegationResponse, QueryUnbondingDelegationResponse, QueryValidatorRequest,
    QueryValidatorResponse, Redelegation, RedelegationEntryResponse, RedelegationResponse,
};
use gears::{
    application::handlers::node::{ModuleInfo, TxError},
    baseapp::{errors::QueryError, QueryResponse},
    context::{block::BlockContext, init::InitContext, query::QueryContext, tx::TxContext},
    core::Protobuf,
    ext::Pagination,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    tendermint::types::{
        proto::validator::ValidatorUpdate,
        request::{
            begin_block::RequestBeginBlock, end_block::RequestEndBlock, query::RequestQuery,
        },
    },
    types::{pagination::response::PaginationResponse, store::gas::ext::GasResultExt},
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
    MI: ModuleInfo,
> {
    keeper: Keeper<SK, PSK, AK, BK, KH, M>,
    phantom_data: std::marker::PhantomData<MI>,
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
        MI: ModuleInfo,
    > ABCIHandler<SK, PSK, AK, BK, KH, M, MI>
{
    pub fn new(keeper: Keeper<SK, PSK, AK, BK, KH, M>) -> Self {
        ABCIHandler {
            keeper,
            phantom_data: std::marker::PhantomData,
        }
    }

    pub fn msg<DB: Database + Sync + Send>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: &Message,
    ) -> Result<(), TxError> {
        let result = match msg {
            Message::CreateValidator(msg) => self.keeper.create_validator(ctx, msg),
            Message::EditValidator(msg) => self.keeper.edit_validator(ctx, msg),
            Message::Delegate(msg) => self.keeper.delegate_cmd_handler(ctx, msg),
            Message::Redelegate(msg) => self.keeper.redelegate_cmd_handler(ctx, msg),
            Message::Undelegate(msg) => self.keeper.undelegate_cmd_handler(ctx, msg),
        };

        result.map_err(|e| Into::<StakingTxError>::into(e).into::<MI>())
    }

    pub fn genesis<DB: Database>(&self, ctx: &mut InitContext<'_, DB, SK>, genesis: GenesisState) {
        self.keeper.init_genesis(ctx, genesis);
    }

    pub fn query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: RequestQuery,
    ) -> Result<prost::bytes::Bytes, QueryError> {
        match query.path.as_str() {
            "/cosmos.staking.v1beta1.Query/Validator" => {
                let req = QueryValidatorRequest::decode(query.data)?;

                Ok(self.keeper.query_validator(ctx, req).into_bytes().into())
            }
            "/cosmos.staking.v1beta1.Query/Delegation" => {
                let req = QueryDelegationRequest::decode(query.data)?;

                Ok(self.keeper.query_delegation(ctx, req).into_bytes().into())
            }
            "/cosmos.staking.v1beta1.Query/Redelegation" => {
                let req = QueryRedelegationRequest::decode(query.data)?;

                Ok(self.query_redelegations(ctx, req).into_bytes().into())
            }
            "/cosmos.staking.v1beta1.Query/UnbondingDelegation" => {
                let req = QueryDelegationRequest::decode(query.data)?;

                Ok(self
                    .keeper
                    .query_unbonding_delegation(ctx, req)
                    .into_bytes()
                    .into())
            }
            "/cosmos/staking/v1beta1/params" | "/cosmos.staking.v1beta1.Query/Params" => {
                Ok(self.keeper.query_params(ctx).into_bytes().into())
            }
            _ => Err(QueryError::PathNotFound),
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
                StakingNodeQueryResponse::Redelegation(self.query_redelegations(ctx, req))
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

    fn query_redelegations<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        QueryRedelegationRequest {
            delegator_address,
            src_validator_address,
            dst_validator_address,
            pagination,
        }: QueryRedelegationRequest,
    ) -> QueryRedelegationResponse {
        let (p_result, redelegations) = self.keeper.redelegations(
            ctx,
            &delegator_address,
            &src_validator_address,
            &dst_validator_address,
            pagination.map(Pagination::from),
        );

        let redelegation_responses = self
            .redelegations_to_redelegations_response(ctx, redelegations)
            .ok()
            .unwrap_or_default();

        QueryRedelegationResponse {
            redelegation_responses,
            pagination: p_result.map(PaginationResponse::from),
        }
    }

    fn redelegations_to_redelegations_response<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        redelegations: Vec<Redelegation>,
    ) -> Result<Vec<RedelegationResponse>, anyhow::Error> {
        let mut resp = Vec::with_capacity(redelegations.len());
        for red in redelegations.into_iter() {
            let validator = self
                .keeper
                .validator(ctx, &red.validator_dst_address)
                .unwrap_gas()
                .ok_or(anyhow::anyhow!("account not found"))?;

            let mut entries = Vec::with_capacity(red.entries.len());
            for entry in red.entries.clone().into_iter() {
                let balance = validator
                    .tokens_from_shares(entry.share_dst)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?
                    .to_uint_floor();
                entries.push(RedelegationEntryResponse {
                    redelegation_entry: entry,
                    balance,
                });
            }

            resp.push(RedelegationResponse {
                redelegation: red,
                entries,
            });
        }

        Ok(resp)
    }
}
