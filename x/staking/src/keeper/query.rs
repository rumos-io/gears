use super::*;
use crate::{
    DelegationResponse, QueryDelegationRequest, QueryDelegationResponse,
    QueryDelegatorDelegationsRequest, QueryDelegatorDelegationsResponse,
    QueryDelegatorUnbondingDelegationsRequest, QueryDelegatorUnbondingDelegationsResponse,
    QueryDelegatorValidatorsRequest, QueryDelegatorValidatorsResponse, QueryHistoricalInfoRequest,
    QueryHistoricalInfoResponse, QueryParamsResponse, QueryPoolResponse,
    QueryUnbondingDelegationRequest, QueryUnbondingDelegationResponse, QueryValidatorRequest,
    QueryValidatorResponse, QueryValidatorsRequest, QueryValidatorsResponse,
};
use gears::{
    baseapp::errors::QueryError,
    context::query::QueryContext,
    core::Protobuf,
    extensions::pagination::{IteratorPaginate, Pagination, PaginationResult},
    types::pagination::response::PaginationResponse,
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
        let validator = self
            .validator(ctx, &query.validator_addr)
            .unwrap_gas()
            .map(Into::into);
        QueryValidatorResponse { validator }
    }

    pub fn query_validators<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: QueryValidatorsRequest,
    ) -> QueryValidatorsResponse {
        let store = ctx.kv_store(&self.store_key);
        let store = store.prefix_store(VALIDATORS_KEY);

        let iterator = store.into_range(..).filter_map(|(k, bytes)| {
            if let Ok(v) = Validator::decode_vec(&bytes) {
                Some((k, v))
            } else {
                None
            }
        });

        let pagination = query
            .pagination
            .map(gears::extensions::pagination::Pagination::from);
        let (validators, p_result) = if query.status == BondStatus::Unspecified {
            let (p_result, iterator) = iterator.maybe_paginate(pagination);
            (
                iterator.map(|(_k, v)| v).map(Into::into).collect(),
                p_result,
            )
        } else {
            let (p_result, iterator) = iterator
                .filter(|(_k, v)| v.status == query.status)
                .maybe_paginate(pagination);
            (
                iterator.map(|(_k, v)| v).map(Into::into).collect(),
                p_result,
            )
        };

        QueryValidatorsResponse {
            validators,
            pagination: p_result.map(PaginationResponse::from),
        }
    }

    pub fn query_delegation<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: QueryDelegationRequest,
    ) -> QueryDelegationResponse {
        if let Some(delegation) = self
            .delegation(ctx, &query.delegator_addr, &query.validator_addr)
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
            delegation: Some(delegation),
            balance: Some(balance),
        })
    }

    pub fn query_delegator_delegations<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: QueryDelegatorDelegationsRequest,
    ) -> QueryDelegatorDelegationsResponse {
        let store = ctx.kv_store(&self.store_key);
        let store = store.prefix_store(DELEGATION_KEY);
        let key = query.delegator_addr.prefix_len_bytes();
        let (p_result, iterator) = store.into_range(..).maybe_paginate(
            query
                .pagination
                .map(gears::extensions::pagination::Pagination::from),
        );

        let delegation_responses = iterator
            .filter_map(|(k, bytes)| {
                if k.starts_with(&key) {
                    Delegation::decode_vec(&bytes).ok()
                } else {
                    None
                }
            })
            .filter_map(|del| self.delegation_to_delegation_response(ctx, del).ok())
            .collect();

        QueryDelegatorDelegationsResponse {
            delegation_responses,
            pagination: p_result.map(PaginationResponse::from),
        }
    }

    pub fn query_unbonding_delegation<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: QueryUnbondingDelegationRequest,
    ) -> QueryUnbondingDelegationResponse {
        QueryUnbondingDelegationResponse {
            unbond: self
                .unbonding_delegation(ctx, &query.delegator_addr, &query.validator_addr)
                .unwrap_gas(),
        }
    }

    pub fn query_unbonding_delegations<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: QueryDelegatorUnbondingDelegationsRequest,
    ) -> Result<QueryDelegatorUnbondingDelegationsResponse, QueryError> {
        let store = ctx.kv_store(&self.store_key);
        let store = store.prefix_store(UNBONDING_DELEGATION_KEY);
        let key = query.delegator_addr.prefix_len_bytes();
        let (p_result, iterator) = store.into_range(..).maybe_paginate(
            query
                .pagination
                .map(gears::extensions::pagination::Pagination::from),
        );

        let mut unbonding_responses = vec![];
        for (k, bytes) in iterator {
            if k.starts_with(&key) {
                unbonding_responses.push(
                    UnbondingDelegation::decode_vec(&bytes)
                        .map_err(|e| QueryError::Proto(e.to_string()))?,
                );
            }
        }

        Ok(QueryDelegatorUnbondingDelegationsResponse {
            unbonding_responses,
            pagination: p_result.map(PaginationResponse::from),
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

    /// query_delegator_validators queries all validators info for given delegator address
    pub fn query_delegator_validators<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: QueryDelegatorValidatorsRequest,
    ) -> QueryDelegatorValidatorsResponse {
        let delegator_addr = if let Ok(addr) = AccAddress::try_from(query.delegator_addr) {
            addr
        } else {
            return QueryDelegatorValidatorsResponse::default();
        };

        let store = ctx.kv_store(&self.store_key);
        let key = [
            DELEGATION_KEY.as_slice(),
            &delegator_addr.prefix_len_bytes(),
        ]
        .concat();
        let delegator_store = store.prefix_store(key);

        let pagination = query
            .pagination
            .map(gears::extensions::pagination::Pagination::from);
        let (p_res, iter) = delegator_store.into_range(..).maybe_paginate(pagination);

        let mut validators = vec![];
        for (_k, v) in iter {
            let delegation = if let Ok(del) = Delegation::decode_vec(&v) {
                del
            } else {
                return QueryDelegatorValidatorsResponse::default();
            };

            if let Some(v) = self
                .validator(ctx, &delegation.validator_address)
                .unwrap_gas()
            {
                validators.push(v);
            } else {
                return QueryDelegatorValidatorsResponse::default();
            }
        }

        QueryDelegatorValidatorsResponse {
            validators,
            pagination: p_res.map(PaginationResponse::from),
        }
    }

    pub fn query_historical_info<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        QueryHistoricalInfoRequest { height }: QueryHistoricalInfoRequest,
    ) -> QueryHistoricalInfoResponse {
        let historical_info = self.historical_info(ctx, height as u32).unwrap_gas();
        QueryHistoricalInfoResponse {
            hist: historical_info,
        }
    }

    pub fn query_pool<DB: Database>(&self, ctx: &QueryContext<DB, SK>) -> QueryPoolResponse {
        let pool = self.pool(ctx).unwrap_gas();
        QueryPoolResponse { pool: Some(pool) }
    }

    pub fn query_params<DB: Database>(&self, ctx: &QueryContext<DB, SK>) -> QueryParamsResponse {
        let params = self.staking_params_keeper.get(ctx);
        QueryParamsResponse {
            params: Some(params),
        }
    }
}
