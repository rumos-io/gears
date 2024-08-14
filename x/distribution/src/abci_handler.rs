use crate::{
    errors::DistributionTxError, GenesisState, Keeper, Message, QueryCommunityPoolRequest,
    QueryCommunityPoolResponse, QueryDelegationRewardsRequest, QueryParamsRequest,
    QueryParamsResponse, QueryValidatorCommissionRequest, QueryValidatorCommissionResponse,
    QueryValidatorOutstandingRewardsRequest, QueryValidatorOutstandingRewardsResponse,
    QueryValidatorSlashesRequest, QueryValidatorSlashesResponse, QueryWithdrawAllRewardsRequest,
};
use gears::{
    baseapp::errors::QueryError,
    context::{
        block::BlockContext, init::InitContext, query::QueryContext, tx::TxContext,
        QueryableContext,
    },
    core::Protobuf,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    tendermint::types::request::{begin_block::RequestBeginBlock, query::RequestQuery},
    types::address::ConsAddress,
    x::{
        keepers::{
            auth::AuthKeeper, staking::DistributionStakingKeeper,
            staking::StakingBankKeeper as BankKeeper,
        },
        module::Module,
    },
};

#[derive(Debug, Clone)]
pub enum DistributionNodeQueryRequest {
    ValidatorOutstandingRewards(QueryValidatorOutstandingRewardsRequest),
    ValidatorCommission(QueryValidatorCommissionRequest),
    ValidatorSlashes(QueryValidatorSlashesRequest),
    CommunityPool(QueryCommunityPoolRequest),
    Params(QueryParamsRequest),
}
#[derive(Debug, Clone)]
pub enum DistributionNodeQueryResponse {
    ValidatorOutstandingRewards(QueryValidatorOutstandingRewardsResponse),
    ValidatorCommission(QueryValidatorCommissionResponse),
    ValidatorSlashes(QueryValidatorSlashesResponse),
    CommunityPool(QueryCommunityPoolResponse),
    Params(QueryParamsResponse),
}

#[derive(Debug, Clone)]
pub struct ABCIHandler<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    AK: AuthKeeper<SK, M>,
    BK: BankKeeper<SK, M>,
    DSK: DistributionStakingKeeper<SK, M>,
    M: Module,
> {
    keeper: Keeper<SK, PSK, AK, BK, DSK, M>,
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: BankKeeper<SK, M>,
        DSK: DistributionStakingKeeper<SK, M>,
        M: Module,
    > ABCIHandler<SK, PSK, AK, BK, DSK, M>
{
    pub fn new(keeper: Keeper<SK, PSK, AK, BK, DSK, M>) -> Self {
        ABCIHandler { keeper }
    }

    pub fn genesis<DB: Database>(&self, ctx: &mut InitContext<'_, DB, SK>, genesis: GenesisState) {
        if let Err(e) = self.keeper.init_genesis(ctx, genesis) {
            panic!("Initialization of genesis failed with error:\n{e}")
        }
    }

    pub fn tx<DB: Database + Sync + Send>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: &Message,
    ) -> Result<(), DistributionTxError> {
        match msg {
            Message::WithdrawRewards(msg) => Ok(self
                .keeper
                .withdraw_delegator_reward_and_commission(ctx, msg)?),
            Message::SetWithdrawAddr(msg) => Ok(self.keeper.set_withdraw_address(ctx, msg)?),
            Message::FundCommunityPool(msg) => Ok(self.keeper.fund_community_pool_cmd(ctx, msg)?),
        }
    }

    pub fn query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: RequestQuery,
    ) -> Result<prost::bytes::Bytes, QueryError> {
        match query.path.as_str() {
            "/cosmos.distribution.v1beta1.Query/ValidatorOutstandingRewards" => {
                let req = QueryValidatorOutstandingRewardsRequest::decode(query.data)?;

                Ok(self
                    .keeper
                    .query_validator_outstanding_rewards(ctx, req)
                    .encode_vec()
                    .into())
            }
            "/cosmos.distribution.v1beta1.Query/ValidatorCommission" => {
                let req = QueryValidatorCommissionRequest::decode(query.data)?;

                Ok(self
                    .keeper
                    .query_validator_commission(ctx, req)
                    .encode_vec()
                    .into())
            }
            "/cosmos.distribution.v1beta1.Query/ValidatorSlashes" => {
                let req = QueryValidatorSlashesRequest::decode(query.data)?;

                Ok(self
                    .keeper
                    .query_validator_slashes(ctx, req)
                    .encode_vec()
                    .into())
            }
            "/cosmos.distribution.v1beta1.Query/DelegationRewards" => {
                let req = QueryDelegationRewardsRequest::decode(query.data)?;

                Ok(self
                    .keeper
                    .query_delegation_rewards(ctx, req)?
                    .encode_vec()
                    .into())
            }
            "/cosmos.distribution.v1beta1.Query/DelegatorValidators" => {
                let req = QueryWithdrawAllRewardsRequest::decode(query.data)?;

                Ok(self
                    .keeper
                    .query_delegator_validators(ctx, req)
                    .encode_vec()
                    .into())
            }
            "/cosmos.distribution.v1beta1.Query/CommunityPool" => {
                let req = QueryCommunityPoolRequest::decode(query.data)?;

                Ok(self
                    .keeper
                    .query_community_pool(ctx, req)
                    .encode_vec()
                    .into())
            }
            "/cosmos.distribution.v1beta1.Query/Params" => {
                let req = QueryParamsRequest::decode(query.data)?;

                Ok(self.keeper.query_params(ctx, req).encode_vec().into())
            }
            _ => Err(QueryError::PathNotFound),
        }
    }

    pub fn typed_query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: DistributionNodeQueryRequest,
    ) -> DistributionNodeQueryResponse {
        match query {
            DistributionNodeQueryRequest::ValidatorOutstandingRewards(req) => {
                DistributionNodeQueryResponse::ValidatorOutstandingRewards(
                    self.keeper.query_validator_outstanding_rewards(ctx, req),
                )
            }
            DistributionNodeQueryRequest::ValidatorCommission(req) => {
                DistributionNodeQueryResponse::ValidatorCommission(
                    self.keeper.query_validator_commission(ctx, req),
                )
            }
            DistributionNodeQueryRequest::ValidatorSlashes(req) => {
                DistributionNodeQueryResponse::ValidatorSlashes(
                    self.keeper.query_validator_slashes(ctx, req),
                )
            }
            DistributionNodeQueryRequest::CommunityPool(req) => {
                DistributionNodeQueryResponse::CommunityPool(
                    self.keeper.query_community_pool(ctx, req),
                )
            }
            DistributionNodeQueryRequest::Params(req) => {
                DistributionNodeQueryResponse::Params(self.keeper.query_params(ctx, req))
            }
        }
    }

    /// begin_block sets the proposer for determining distribution during end_block
    /// and distribute rewards for the previous block
    pub fn begin_block<DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, SK>,
        request: RequestBeginBlock,
    ) {
        // determine the total power signing the block
        let mut sum_previous_precommit_power: u64 = 0;
        let previous_total_power = request.last_commit_info.votes.iter().fold(0, |acc, vote| {
            let power = u64::from(vote.validator.power);
            if vote.signed_last_block {
                sum_previous_precommit_power += power;
            }
            acc + power
        });

        // TODO this is Tendermint-dependent
        // ref https://github.com/cosmos/cosmos-sdk/issues/3095

        if ctx.height() > 1 {
            if let Some(previous_proposer) = self.keeper.previous_proposer_cons_addr(ctx) {
                if let Err(e) = self.keeper.allocate_tokens(
                    ctx,
                    sum_previous_precommit_power,
                    previous_total_power,
                    &previous_proposer,
                    &request.last_commit_info.votes,
                ) {
                    panic!("Error thrown in begin_block method: \n{e}");
                }
            } else {
                panic!("previous proposer not set");
            }
        }

        // record the proposer for when we payout on the next block
        // TODO: consider to change request header structure to have ConsAddress
        let cons_addr = match ConsAddress::try_from(request.header.proposer_address) {
            Ok(addr) => addr,
            Err(e) => panic!("{e}"),
        };
        self.keeper.set_previous_proposer_cons_addr(ctx, &cons_addr);
    }
}
