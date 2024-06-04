use super::*;
use crate::{
    DelegationResponse, QueryDelegationRequest, QueryDelegationResponse, QueryRedelegationRequest,
    QueryRedelegationResponse, QueryValidatorRequest, QueryValidatorResponse,
    RedelegationEntryResponse, RedelegationResponse,
};
use gears::context::query::QueryContext;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AccountKeeper<SK>,
        BK: BankKeeper<SK>,
        KH: KeeperHooks<SK>,
    > Keeper<SK, PSK, AK, BK, KH>
{
    pub fn query_validator<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: QueryValidatorRequest,
    ) -> Result<QueryValidatorResponse, AppError> {
        let validator = self
            .validator(ctx, &query.address)?
            .ok_or(AppError::Custom("Validator is not found".into()))?;
        Ok(QueryValidatorResponse { validator })
    }

    pub fn query_delegation<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: QueryDelegationRequest,
    ) -> Result<QueryDelegationResponse, AppError> {
        let delegation = self
            .delegation(ctx, &query.delegator_address, &query.validator_address)?
            .ok_or(AppError::Custom("Delegation doesn't exists".into()))?;
        let delegation_response = self.delegation_to_delegation_response(ctx, delegation)?;
        Ok(QueryDelegationResponse {
            delegation_response,
        })
    }

    pub fn delegation_to_delegation_response<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        delegation: Delegation,
    ) -> Result<DelegationResponse, AppError> {
        let validator = self
            .validator(ctx, &delegation.validator_address)?
            .ok_or(AppError::AccountNotFound)?;

        let params = self.staking_params_keeper.get(ctx);
        let tokens = validator
            .tokens_from_shares(delegation.shares)
            .map_err(|e| AppError::Coins(e.to_string()))?;
        let balance = Coin {
            denom: params.bond_denom,
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
    ) -> Result<QueryRedelegationResponse, AppError> {
        let redelegations = match &query {
            QueryRedelegationRequest {
                delegator_address: Some(a),
                src_validator_address: Some(v1),
                dst_validator_address: Some(v2),
                pagination: _,
            } => {
                let redelegation = self
                    .redelegation(ctx, a, v1, v2)?
                    .ok_or(AppError::Custom("no redelegation found".to_string()))?;
                vec![redelegation]
            }
            QueryRedelegationRequest {
                delegator_address: None,
                src_validator_address: Some(_v1),
                dst_validator_address: None,
                pagination: _,
            } => {
                //     redels = k.GetRedelegationsFromSrcValidator(ctx, params.SrcValidatorAddr)
                todo!()
            }
            _ => {
                //     redels = k.GetAllRedelegations(ctx, params.DelegatorAddr, params.SrcValidatorAddr, params.DstValidatorAddr)
                todo!()
            }
        };

        let redelegation_responses =
            self.redelegations_to_redelegations_response(ctx, redelegations)?;

        Ok(QueryRedelegationResponse {
            redelegation_responses,
            pagination: query.pagination,
        })
    }

    pub fn redelegations_to_redelegations_response<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        redelegations: Vec<Redelegation>,
    ) -> Result<Vec<RedelegationResponse>, AppError> {
        let mut resp = Vec::with_capacity(redelegations.len());
        for red in redelegations.into_iter() {
            let validator = self
                .validator(ctx, &red.validator_dst_address)?
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
}
