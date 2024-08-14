use std::fmt::Debug;

use kv_store::StoreKey;

use crate::{
    context::{QueryableContext, TransactionalContext},
    types::{account::Account, store::gas::errors::GasStoreErrors},
    x::{
        keepers::auth::{AuthKeeper, AuthParams},
        module::Module,
    },
};

#[derive(Debug, Clone)]
pub struct MockAuthParams {
    pub max_memo_characters: u64,
    pub sig_verify_cost_secp256k1: u64,
    pub tx_cost_per_byte: u64,
}

impl Default for MockAuthParams {
    fn default() -> Self {
        Self {
            max_memo_characters: 256,
            tx_cost_per_byte: 10,
            sig_verify_cost_secp256k1: 1000,
        }
    }
}

impl AuthParams for MockAuthParams {
    fn max_memo_characters(&self) -> u64 {
        self.max_memo_characters
    }

    fn sig_verify_cost_secp256k1(&self) -> u64 {
        self.sig_verify_cost_secp256k1
    }

    fn tx_cost_per_byte(&self) -> u64 {
        self.tx_cost_per_byte
    }
}

#[derive(former::Former)]
pub struct MockAuthKeeper {
    pub get_auth_params: Option<Box<dyn Fn() -> Result<MockAuthParams, GasStoreErrors>>>,
    pub has_account: Option<Box<dyn Fn() -> Result<bool, GasStoreErrors>>>,
    pub get_account: Option<Box<dyn Fn() -> Result<Option<Account>, GasStoreErrors>>>,
    pub set_account: Option<Box<dyn Fn() -> Result<(), GasStoreErrors>>>,
    pub create_new_base_account: Option<Box<dyn Fn() -> Result<(), GasStoreErrors>>>,
    pub check_create_new_module_account: Option<Box<dyn Fn() -> Result<(), GasStoreErrors>>>,
}

impl Debug for MockAuthKeeper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockAuthKeeper")
            .field("get_auth_params: is_some", &self.get_auth_params.is_some())
            .finish()
    }
}

impl Clone for MockAuthKeeper {
    fn clone(&self) -> Self {
        Self {
            get_auth_params: None,
            has_account: None,
            get_account: None,
            set_account: None,
            create_new_base_account: None,
            check_create_new_module_account: None,
        }
    }
}

impl<SK: StoreKey, M: Module> AuthKeeper<SK, M> for MockAuthKeeper {
    type Params = MockAuthParams;

    fn get_auth_params<DB, CTX: QueryableContext<DB, SK>>(
        &self,
        _: &CTX,
    ) -> Result<Self::Params, GasStoreErrors> {
        self.get_auth_params
            .as_ref()
            .expect("get_auth_params mock not set")()
    }

    fn has_account<DB: database::Database, CTX: QueryableContext<DB, SK>>(
        &self,
        _: &CTX,
        _: &address::AccAddress,
    ) -> Result<bool, GasStoreErrors> {
        self.has_account.as_ref().expect("has_account mock not set")()
    }

    fn get_account<DB: database::Database, CTX: QueryableContext<DB, SK>>(
        &self,
        _: &CTX,
        _: &address::AccAddress,
    ) -> Result<Option<Account>, GasStoreErrors> {
        self.get_account.as_ref().expect("get_account mock not set")()
    }

    fn set_account<DB: database::Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        _: &mut CTX,
        _: crate::types::account::Account,
    ) -> Result<(), GasStoreErrors> {
        self.set_account.as_ref().expect("set_account mock not set")()
    }

    fn create_new_base_account<DB: database::Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        _: &mut CTX,
        _: &address::AccAddress,
    ) -> Result<(), GasStoreErrors> {
        self.create_new_base_account
            .as_ref()
            .expect("create_new_base_account mock not set")()
    }

    fn check_create_new_module_account<
        DB: database::Database,
        CTX: TransactionalContext<DB, SK>,
    >(
        &self,
        _: &mut CTX,
        _: &M,
    ) -> Result<(), GasStoreErrors> {
        self.check_create_new_module_account
            .as_ref()
            .expect("check_create_new_module_account mock not set")()
    }
}
