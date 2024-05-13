use core_types::address::AccAddress;
use database::Database;
use store_crate::StoreKey;

use crate::{
    types::{
        account::Account,
        context::{QueryableContext, TransactionalContext},
    },
    x::module::Module,
};

pub trait AuthParams {
    fn max_memo_characters(&self) -> u64;
    fn sig_verify_cost_secp256k1(&self) -> u64;
    fn tx_cost_per_byte(&self) -> u64;
}

pub trait AuthKeeper<SK: StoreKey>: Clone + Send + Sync + 'static {
    type Params: AuthParams;

    fn get_auth_params<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Self::Params;

    fn has_account<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        addr: &AccAddress,
    ) -> bool;

    fn get_account<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        addr: &AccAddress,
    ) -> Option<Account>;

    fn set_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        acct: Account,
    );

    /// Overwrites existing account
    fn create_new_base_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        addr: &AccAddress,
    );

    /// Creates a new module account if it doesn't already exist
    fn check_create_new_module_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        module: &Module,
    );
}
