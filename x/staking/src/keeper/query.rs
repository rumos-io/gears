use gears::types::context::query::QueryContext;

use crate::{DelegationResponse, QueryDelegationRequest, QueryDelegationResponse};

pub use super::*;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AccountKeeper<SK>,
        BK: BankKeeper<SK>,
        KH: KeeperHooks<SK>,
    > Keeper<SK, PSK, AK, BK, KH>
{
    pub fn query_delegation<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: QueryDelegationRequest,
    ) -> Result<QueryDelegationResponse, AppError> {
        let delegator_address = query.delegator_address;
        let validator_address = query.validator_address;
        let delegation = if let Some(delegation) =
            self.delegation(ctx, &delegator_address, &validator_address)
        {
            delegation
        } else {
            return Err(AppError::Custom("Delegation doesn't exists".into()));
        };

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
        let validator = if let Some(validator) = self.validator(ctx, &delegation.validator_address)
        {
            validator
        } else {
            return Err(AppError::AccountNotFound);
        };

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
