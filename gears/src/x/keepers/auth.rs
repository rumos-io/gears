use database::Database;
use kv_store::StoreKey;

use crate::{
    context::{ImmutableGasContext, MutableGasContext},
    types::{account::Account, address::AccAddress, store::errors::StoreErrors},
    x::module::Module,
};

pub trait AuthParams {
    fn max_memo_characters(&self) -> u64;
    fn sig_verify_cost_secp256k1(&self) -> u64;
    fn tx_cost_per_byte(&self) -> u64;
}

pub trait AuthKeeper<SK: StoreKey>: Clone + Send + Sync + 'static {
    type Params: AuthParams;

    fn get_auth_params<DB: Database, CTX: ImmutableGasContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<Self::Params, StoreErrors>;

    fn has_account<DB: Database, CTX: ImmutableGasContext<DB, SK>>(
        &self,
        ctx: &CTX,
        addr: &AccAddress,
    ) -> Result<bool, StoreErrors>;

    fn get_account<DB: Database, CTX: ImmutableGasContext<DB, SK>>(
        &self,
        ctx: &CTX,
        addr: &AccAddress,
    ) -> Result<Option<Account>, StoreErrors>;

    fn set_account<DB: Database, CTX: MutableGasContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        acct: Account,
    ) -> Result<(), StoreErrors>;

    /// Overwrites existing account
    fn create_new_base_account<DB: Database, CTX: MutableGasContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        addr: &AccAddress,
    ) -> Result<(), StoreErrors>;

    /// Creates a new module account if it doesn't already exist
    fn check_create_new_module_account<DB: Database, CTX: MutableGasContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        module: &Module,
    ) -> Result<(), StoreErrors>;
}
