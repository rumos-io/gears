use gears::x::keepers::staking::StakingKeeper;

use crate::iter::{bounded::BoundedValidatorsIterator, delegation::DelegationIterator};

use super::*;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: BankKeeper<SK, M>,
        KH: KeeperHooks<SK, AK, M>,
        M: Module,
    > StakingKeeper<SK, M> for Keeper<SK, PSK, AK, BK, KH, M>
{
    type Validator = Validator;
    type Delegation = Delegation;

    fn bonded_validators_by_power<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<impl Iterator<Item = Result<Validator, GasStoreErrors>>, GasStoreErrors> {
        Ok(BoundedValidatorsIterator::new(
            ctx.kv_store(&self.store_key),
            self.staking_params_keeper.try_get(ctx)?.max_validators,
        ))
    }

    fn delegations<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        voter: &AccAddress,
    ) -> impl Iterator<Item = Result<Self::Delegation, GasStoreErrors>> {
        DelegationIterator::new(ctx.kv_store(&self.store_key), voter)
            .map(|this| this.map(|(_, value)| value))
    }
}
