use gears::x::keepers::{gov::GovernanceBankKeeper, staking::GovStakingKeeper};

use crate::iter::{bounded::BoundedValidatorsIterator, delegation::DelegationIterator};

use super::*;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: GovernanceBankKeeper<SK, M>,
        KH: KeeperHooks<SK, AK, M>,
        M: Module,
    > GovStakingKeeper<SK, M> for Keeper<SK, PSK, AK, BK, KH, M>
{
    type Validator = Validator;
    type Delegation = Delegation;

    fn bonded_validators_by_power_iter<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<impl Iterator<Item = Result<Validator, GasStoreErrors>>, GasStoreErrors> {
        Ok(BoundedValidatorsIterator::new(
            ctx.kv_store(&self.store_key),
            self.staking_params_keeper.try_get(ctx)?.max_validators,
        ))
    }

    fn delegations_iter<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        voter: &AccAddress,
    ) -> impl Iterator<Item = Result<Self::Delegation, GasStoreErrors>> {
        DelegationIterator::new(ctx.kv_store(&self.store_key), voter)
            .map(|this| this.map(|(_, value)| value))
    }

    fn total_bonded_tokens<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<Coin, GasStoreErrors> {
        let account = self
            .auth_keeper
            .get_account(ctx, &self.bonded_module.get_address())?
            .unwrap(); // TODO: Unsure what to do in this case

        self.bank_keeper.balance(
            ctx,
            account.get_address(),
            &self.staking_params_keeper.try_get(ctx)?.bond_denom,
        )
    }
}
