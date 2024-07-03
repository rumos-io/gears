use super::*;
use crate::{
    DelegationResponse, QueryDelegationRequest, QueryDelegationResponse, QueryParamsResponse,
    QueryRedelegationRequest, QueryRedelegationResponse, QueryUnbondingDelegationResponse,
    QueryValidatorRequest, QueryValidatorResponse, RedelegationEntryResponse, RedelegationResponse,
};
use gears::context::query::QueryContext;

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
    ) -> Result<DelegationResponse, AppError> {
        let validator = self
            .validator(ctx, &delegation.validator_address)
            .unwrap_gas()
            .ok_or(AppError::AccountNotFound)?;

        let params = self.staking_params_keeper.get(ctx);
        let tokens = validator
            .tokens_from_shares(delegation.shares)
            .map_err(|e| AppError::Coins(e.to_string()))?;
        let balance = Coin {
            denom: params.bond_denom().clone(),
            amount: tokens.to_uint_floor(),
        };
        Ok(DelegationResponse {
            delegation,
            balance,
        })
    }

    pub fn query_redelegations<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: QueryRedelegationRequest,
    ) -> QueryRedelegationResponse {
        let redelegations = match &query {
            QueryRedelegationRequest {
                delegator_address: Some(a),
                src_validator_address: Some(v1),
                dst_validator_address: Some(v2),
                pagination: _,
            } => self
                .redelegation(ctx, a, v1, v2)
                .unwrap_gas()
                .map(|red| vec![red])
                .unwrap_or_default(),
            /* */
            QueryRedelegationRequest {
                delegator_address: None,
                src_validator_address: Some(_v1),
                dst_validator_address: None,
                pagination: _,
            } => {
                // TODO: add logic for a query with only src validator
                //     redels = k.GetRedelegationsFromSrcValidator(ctx, params.SrcValidatorAddr)
                todo!()
            }
            /* */
            _ => {
                // TODO: add logic for a query to get all redelegations
                //     redels = k.GetAllRedelegations(ctx, params.DelegatorAddr, params.SrcValidatorAddr, params.DstValidatorAddr)
                todo!()
            }
        };

        let redelegation_responses = self
            .redelegations_to_redelegations_response(ctx, redelegations)
            .ok()
            .unwrap_or_default();

        QueryRedelegationResponse {
            redelegation_responses,
            pagination: query.pagination,
        }
    }

    pub fn redelegations_to_redelegations_response<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        redelegations: Vec<Redelegation>,
    ) -> Result<Vec<RedelegationResponse>, AppError> {
        let mut resp = Vec::with_capacity(redelegations.len());
        for red in redelegations.into_iter() {
            let validator = self
                .validator(ctx, &red.validator_dst_address)
                .unwrap_gas()
                .ok_or(AppError::AccountNotFound)?;

            let mut entries = Vec::with_capacity(red.entries.len());
            for entry in red.entries.clone().into_iter() {
                let balance = validator
                    .tokens_from_shares(entry.share_dst)
                    .map_err(|e| AppError::Custom(e.to_string()))?
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
        QueryParamsResponse { params }
    }
}
