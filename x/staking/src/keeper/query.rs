use super::*;
use crate::{DelegationResponse, IbcV046Validator};
use gears::{
    baseapp::errors::QueryError,
    context::query::QueryContext,
    core::Protobuf,
    extensions::pagination::{IteratorPaginate, Pagination, PaginationResult},
    types::pagination::{request::PaginationRequest, response::PaginationResponse},
    x::types::delegation::StakingDelegation,
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
    pub fn validators<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        status: BondStatus,
        pagination: Option<PaginationRequest>,
    ) -> (Option<PaginationResponse>, Vec<IbcV046Validator>) {
        let store = ctx.kv_store(&self.store_key);
        let store = store.prefix_store(VALIDATORS_KEY);

        let iterator = store.into_range(..).filter_map(|(k, bytes)| {
            if let Ok(v) = Validator::decode_vec(&bytes) {
                Some((k, v))
            } else {
                None
            }
        });

        let pagination = pagination.map(gears::extensions::pagination::Pagination::from);
        let (validators, p_result) = if status == BondStatus::Unspecified {
            let (p_result, iterator) = iterator.maybe_paginate(pagination);
            (
                iterator.map(|(_k, v)| v).map(Into::into).collect(),
                p_result,
            )
        } else {
            let (p_result, iterator) = iterator
                .filter(|(_k, v)| v.status == status)
                .maybe_paginate(pagination);
            (
                iterator.map(|(_k, v)| v).map(Into::into).collect(),
                p_result,
            )
        };

        (p_result.map(PaginationResponse::from), validators)
    }

    pub fn delegator_delegations<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        delegator: &AccAddress,
        pagination: Option<PaginationRequest>,
    ) -> (Option<PaginationResponse>, Vec<DelegationResponse>) {
        let store = ctx.kv_store(&self.store_key);
        let key = [DELEGATION_KEY.as_slice(), &delegator.prefix_len_bytes()].concat();
        let store = store.prefix_store(key);
        let (p_result, iterator) = store
            .into_range(..)
            .maybe_paginate(pagination.map(gears::extensions::pagination::Pagination::from));
        let delegations = iterator
            .filter_map(|(_k, v)| Delegation::decode_vec(&v).ok())
            .filter_map(|del| self.delegation_to_delegation_response(ctx, del).ok())
            .collect();
        (p_result.map(PaginationResponse::from), delegations)
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

    pub fn validator_delegations<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        validator: &ValAddress,
        pagination: Option<PaginationRequest>,
    ) -> (Option<PaginationResponse>, Vec<DelegationResponse>) {
        let store = ctx.kv_store(&self.store_key);
        let store = store.prefix_store(DELEGATION_KEY);

        // TODO: more complex logic with iterator and pagination
        let delegations: Vec<_> = store
            .into_range(..)
            .filter_map(|(_k, bytes)| {
                if let Ok(del) = Delegation::decode_vec(&bytes) {
                    if del.validator() == validator {
                        Some(del)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .filter_map(|del| self.delegation_to_delegation_response(ctx, del).ok())
            .collect();
        let (p_res, iterator) = delegations
            .into_iter()
            .maybe_paginate(pagination.map(gears::extensions::pagination::Pagination::from));

        (p_res.map(PaginationResponse::from), iterator.collect())
    }

    pub fn unbonding_delegations<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        delegator: &AccAddress,
        pagination: Option<PaginationRequest>,
    ) -> Result<(Option<PaginationResponse>, Vec<UnbondingDelegation>), QueryError> {
        let store = ctx.kv_store(&self.store_key);
        let store = store.prefix_store(UNBONDING_DELEGATION_KEY);
        let key = delegator.prefix_len_bytes();

        let (p_result, iterator) = store
            .into_range(..)
            .filter(|(k, _v)| k.starts_with(&key))
            .maybe_paginate(pagination.map(gears::extensions::pagination::Pagination::from));

        let mut unbonding_responses = vec![];
        for (k, bytes) in iterator {
            if k.starts_with(&key) {
                unbonding_responses.push(
                    UnbondingDelegation::decode_vec(&bytes)
                        .map_err(|e| QueryError::Proto(e.to_string()))?,
                );
            }
        }

        Ok((p_result.map(PaginationResponse::from), unbonding_responses))
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

    pub fn delegator_validators<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        delegator: &AccAddress,
        pagination: Option<PaginationRequest>,
    ) -> (Option<PaginationResponse>, Vec<Validator>) {
        let store = ctx.kv_store(&self.store_key);
        let key = [DELEGATION_KEY.as_slice(), &delegator.prefix_len_bytes()].concat();
        let delegator_store = store.prefix_store(key);

        let (p_res, iter) = delegator_store
            .into_range(..)
            .maybe_paginate(pagination.map(gears::extensions::pagination::Pagination::from));
        let pagination = p_res.map(PaginationResponse::from);

        let mut validators = vec![];
        for (_k, v) in iter {
            let delegation = if let Ok(del) = Delegation::decode_vec(&v) {
                del
            } else {
                return (pagination, vec![]);
            };

            if let Some(v) = self
                .validator(ctx, &delegation.validator_address)
                .unwrap_gas()
            {
                validators.push(v);
            } else {
                return (pagination, vec![]);
            }
        }

        (pagination, validators)
    }
}
