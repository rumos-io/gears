use std::marker::PhantomData;

use gears::{
    context::TransactionalContext,
    store::{database::Database, StoreKey},
    types::{
        address::{AccAddress, ConsAddress, ValAddress},
        decimal256::Decimal256,
    },
    x::{
        keepers::{auth::AuthKeeper, staking::KeeperHooks},
        module::Module,
    },
};

/// An implementor of KeeperHooks trait. Do nothing.
#[derive(Debug, Clone, Default)]
pub struct MockHookKeeper<SK: StoreKey, AK: AuthKeeper<SK, M>, M: Module> {
    _handlers: PhantomData<(SK, AK, M)>,
}

impl<SK: StoreKey, AK: AuthKeeper<SK, M>, M: Module> MockHookKeeper<SK, AK, M> {
    pub fn new() -> MockHookKeeper<SK, AK, M> {
        MockHookKeeper {
            _handlers: PhantomData,
        }
    }
}

impl<SK: StoreKey, AK: AuthKeeper<SK, M> + Send + Sync + 'static, M: Module> KeeperHooks<SK, AK, M>
    for MockHookKeeper<SK, AK, M>
{
    fn after_validator_created<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        _ctx: &mut CTX,
        _val_addr: ValAddress,
    ) {
    }

    fn before_validator_modified<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        _ctx: &mut CTX,
        _val_addr: ValAddress,
    ) {
    }

    fn after_validator_removed<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        _ctx: &mut CTX,
        _cons_addr: ConsAddress,
        _val_addr: ValAddress,
    ) {
    }

    fn after_validator_bonded<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        _ctx: &mut CTX,
        _cons_addr: ConsAddress,
        _val_addr: ValAddress,
    ) {
    }

    fn after_validator_begin_unbonding<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        _ctx: &mut CTX,
        _cons_addr: ConsAddress,
        _val_addr: ValAddress,
    ) {
    }

    fn before_delegation_created<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        _ctx: &mut CTX,
        _del_addr: AccAddress,
        _val_addr: ValAddress,
    ) {
    }

    fn before_delegation_shares_modified<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        _ctx: &mut CTX,
        _del_addr: AccAddress,
        _val_addr: ValAddress,
    ) {
    }

    fn before_delegation_removed<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        _ctx: &mut CTX,
        _del_addr: AccAddress,
        _val_addr: ValAddress,
    ) {
    }

    fn after_delegation_modified<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        _ctx: &mut CTX,
        _del_addr: AccAddress,
        _val_addr: ValAddress,
    ) {
    }

    fn before_validator_slashed<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        _ctx: &mut CTX,
        _val_addr: ValAddress,
        _fraction: Decimal256,
    ) {
    }
}
