pub use super::*;
use crate::{
    DelegationResponse, QueryDelegationRequest, QueryDelegationResponse, QueryValidatorRequest,
    QueryValidatorResponse,
};
use gears::types::context::query::QueryContext;

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
            .validator(ctx, &query.address)
            .ok_or(AppError::Custom("Validator is not found".into()))?;
        Ok(QueryValidatorResponse { validator })
    }

    pub fn query_delegation<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: QueryDelegationRequest,
    ) -> Result<QueryDelegationResponse, AppError> {
        let delegation = self
            .delegation(ctx, &query.delegator_address, &query.validator_address)
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
            .validator(ctx, &delegation.validator_address)
            .ok_or(AppError::AccountNotFound)?;

        let params = self.staking_params_keeper.get(&ctx.multi_store());
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
}
