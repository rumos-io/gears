use super::*;
use crate::{
    DelegationResponse, QueryDelegationRequest, QueryDelegationResponse, QueryParamsResponse,
    QueryUnbondingDelegationResponse, QueryValidatorRequest, QueryValidatorResponse,
};
use gears::{
    context::query::QueryContext,
    ext::{IteratorPaginate, Pagination, PaginationResult},
};

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: StakingBankKeeper<SK, M>,
        KH: KeeperHooks<SK, AK, M>,
        M: Module,
    > Keeper<SK, PSK, AK, BK, KH, M>
{
    pub fn query_validator<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: QueryValidatorRequest,
    ) -> QueryValidatorResponse {
        let validator = self.validator(ctx, &query.address).unwrap_gas();
        QueryValidatorResponse { validator }
    }

    pub fn query_delegation<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: QueryDelegationRequest,
    ) -> QueryDelegationResponse {
        if let Some(delegation) = self
            .delegation(ctx, &query.delegator_address, &query.validator_address)
            .unwrap_gas()
        {
            let delegation_response = self.delegation_to_delegation_response(ctx, delegation).ok();
            QueryDelegationResponse {
                delegation_response,
            }
        } else {
            QueryDelegationResponse {
                delegation_response: None,
            }
        }
    }

    pub fn delegation_to_delegation_response<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        delegation: Delegation,
    ) -> Result<DelegationResponse, anyhow::Error> {
        let validator = self
            .validator(ctx, &delegation.validator_address)
            .unwrap_gas()
            .ok_or(anyhow::anyhow!("account not found"))?;

        let params = self.staking_params_keeper.get(ctx);
        let tokens = validator.tokens_from_shares(delegation.shares)?;
        let balance = UnsignedCoin {
            denom: params.bond_denom().clone(),
            amount: tokens.to_uint_floor(),
        };
        Ok(DelegationResponse {
            delegation,
            balance,
        })
    }

    pub fn redelegations<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        delegator_address: &Option<AccAddress>,
        src_validator_address: &Option<ValAddress>,
        dst_validator_address: &Option<ValAddress>,
        pagination: Option<Pagination>,
    ) -> (Option<PaginationResult>, Vec<Redelegation>) {
        let redelegations = match (
            delegator_address,
            src_validator_address,
            dst_validator_address,
        ) {
            (Some(a), Some(v1), Some(v2)) => self
                .redelegation(ctx, a, v1, v2)
                .unwrap_gas()
                .map(|red| vec![red])
                .unwrap_or_default(),
            (None, Some(_v1), None) => {
                /*
                  TODO: add logic for a query with only src validator
                    redels = k.GetRedelegationsFromSrcValidator(ctx, params.SrcValidatorAddr)
                */
                todo!()
            }
            _ => {
                // TODO: add logic for a query to get all redelegations
                //     redels = k.GetAllRedelegations(ctx, params.DelegatorAddr, params.SrcValidatorAddr, params.DstValidatorAddr)
                todo!()
            }
        };

        let (p_result, iter) = redelegations.into_iter().maybe_paginate(pagination);

        (p_result, iter.collect())
    }

    pub fn query_unbonding_delegation<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: QueryDelegationRequest,
    ) -> QueryUnbondingDelegationResponse {
        QueryUnbondingDelegationResponse {
            unbond: self
                .unbonding_delegation(ctx, &query.delegator_address, &query.validator_address)
                .unwrap_gas(),
        }
    }

    pub fn query_params<DB: Database>(&self, ctx: &QueryContext<DB, SK>) -> QueryParamsResponse {
        let params = self.staking_params_keeper.get(ctx);
        QueryParamsResponse {
            params: Some(params),
        }
    }
}
