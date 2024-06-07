use super::*;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AccountKeeper<SK, M>,
        BK: BankKeeper<SK, M>,
        KH: KeeperHooks<SK, M>,
        M: Module,
    > Keeper<SK, PSK, AK, BK, KH, M>
{
    pub fn after_validator_created<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) {
        if let Some(ref hooks) = self.hooks_keeper {
            hooks.after_validator_created(ctx, validator.operator_address.clone());
        }
    }

    pub fn after_validator_bonded<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) {
        if let Some(ref hooks) = self.hooks_keeper {
            hooks.after_validator_bonded(
                ctx,
                validator.cons_addr(),
                validator.operator_address.clone(),
            );
        }
    }

    pub fn before_delegation_created<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        del_addr: &AccAddress,
        val_addr: &ValAddress,
    ) {
        if let Some(ref hooks) = self.hooks_keeper {
            hooks.before_delegation_created(ctx, del_addr.clone(), val_addr.clone());
        }
    }

    pub fn before_delegation_shares_modified<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        del_addr: &AccAddress,
        val_addr: &ValAddress,
    ) {
        if let Some(ref hooks) = self.hooks_keeper {
            hooks.before_delegation_shares_modified::<DB, AK, CTX>(
                ctx,
                del_addr.clone(),
                val_addr.clone(),
            );
        }
    }

    pub fn after_delegation_modified<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        del_addr: &AccAddress,
        val_addr: &ValAddress,
    ) {
        if let Some(ref hooks) = self.hooks_keeper {
            hooks.after_delegation_modified(ctx, del_addr.clone(), val_addr.clone());
        }
    }
}
