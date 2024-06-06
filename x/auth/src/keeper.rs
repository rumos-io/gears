use std::collections::HashMap;

use crate::{AuthParamsKeeper, AuthsParams, GenesisState};
use bytes::Bytes;
use gears::context::init::InitContext;
use gears::context::query::QueryContext;
use gears::context::{QueryableContext, TransactionalContext};
use gears::error::IBC_ENCODE_UNWRAP;
use gears::params::ParamsSubspaceKey;
use gears::store::database::{ext::UnwrapCorrupt, Database};
use gears::store::StoreKey;
use gears::tendermint::types::proto::Protobuf as _;
use gears::types::address::AccAddress;
use gears::types::query::account::QueryAccountRequest;
use gears::types::store::gas::errors::GasStoreErrors;
use gears::types::{
    account::{Account, BaseAccount, ModuleAccount},
    query::account::QueryAccountResponse,
};
use gears::x::keepers::auth::AuthKeeper;
use gears::x::module::{Module, ModuleKey};
use prost::Message;

const ACCOUNT_STORE_PREFIX: [u8; 1] = [1];
const GLOBAL_ACCOUNT_NUMBER_KEY: [u8; 19] = [
    103, 108, 111, 098, 097, 108, 065, 099, 099, 111, 117, 110, 116, 078, 117, 109, 098, 101, 114,
]; // "globalAccountNumber"

#[derive(Debug, Clone)]
pub struct Keeper<SK: StoreKey, PSK: ParamsSubspaceKey, MK: ModuleKey> {
    store_key: SK,
    auth_params_keeper: AuthParamsKeeper<PSK>,
    /// Static map of modules. Module keys are declared on app level and can't be changed. The module instances can
    /// be updated using proper keys.
    mod_storage: HashMap<MK, Module>,
    /// Fee collector access key.
    fee_collector_key: MK,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, MK: ModuleKey> AuthKeeper<SK, MK>
    for Keeper<SK, PSK, MK>
{
    type Params = AuthsParams;

    fn get_auth_params<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<Self::Params, GasStoreErrors> {
        self.auth_params_keeper.try_get(ctx)
    }

    fn has_account<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        addr: &AccAddress,
    ) -> Result<bool, GasStoreErrors> {
        let auth_store = ctx.kv_store(&self.store_key);
        let key = create_auth_store_key(addr.to_owned());
        Ok(auth_store.get(&key)?.is_some())
    }

    fn get_account<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        addr: &AccAddress,
    ) -> Result<Option<Account>, GasStoreErrors> {
        let auth_store = ctx.kv_store(&self.store_key);
        let key = create_auth_store_key(addr.to_owned());
        let account = auth_store.get(&key)?;

        if let Some(buf) = account {
            let account = Account::decode::<Bytes>(buf.to_owned().into())
                .ok()
                .unwrap_or_corrupt();

            Ok(Some(account))
        } else {
            Ok(None)
        }
    }

    fn set_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        acct: Account,
    ) -> Result<(), GasStoreErrors> {
        let mut auth_store = ctx.kv_store_mut(&self.store_key);
        let key = create_auth_store_key(acct.get_address().to_owned());

        auth_store.set(key, acct.encode_vec().expect(IBC_ENCODE_UNWRAP))?; // TODO:IBC

        Ok(())
    }

    fn create_new_base_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        addr: &AccAddress,
    ) -> Result<(), GasStoreErrors> {
        let acct = BaseAccount {
            address: addr.clone(),
            pub_key: None,
            account_number: self.get_next_account_number(ctx)?,
            sequence: 0,
        };

        self.set_account(ctx, Account::Base(acct))?;

        Ok(())
    }

    fn check_create_new_module_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        module_key: &MK,
    ) -> Result<(), GasStoreErrors> {
        if let Some(module) = self.mod_storage.get(module_key) {
            let addr = module.get_address();

            if !self.has_account(ctx, &addr)? {
                let account = ModuleAccount {
                    base_account: BaseAccount {
                        address: addr.clone(),
                        pub_key: None,
                        account_number: self.get_next_account_number(ctx)?,
                        sequence: 0,
                    },
                    name: module.get_name().to_string(),
                    permissions: module.get_permissions().clone(),
                };

                self.set_account(ctx, Account::Module(account))?
            }
            Ok(())
        } else {
            panic!(
                "Module with key '{}' doesn't exist. Please, build application with proper modules",
                module_key.key()
            );
        }
    }

    fn get_module_account<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        _ctx: &CTX,
        module_key: &MK,
    ) -> Result<Option<Module>, GasStoreErrors> {
        Ok(self.mod_storage.get(module_key).cloned())
    }
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, MK: ModuleKey> Keeper<SK, PSK, MK> {
    pub fn new(
        store_key: SK,
        params_subspace_key: PSK,
        mod_storage: HashMap<MK, Module>,
        fee_collector_key: MK,
    ) -> Self {
        let auth_params_keeper = AuthParamsKeeper {
            params_subspace_key,
        };

        Keeper {
            store_key,
            auth_params_keeper,
            mod_storage,
            fee_collector_key,
        }
    }

    pub fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        genesis: GenesisState,
    ) {
        //TODO: sdk sanitizes accounts
        self.auth_params_keeper.set(ctx, genesis.params);

        for mut acct in genesis.accounts {
            acct.account_number = self
                .get_next_account_number(ctx)
                .expect("Init context doesn't have any gas");
            self.set_account(ctx, Account::Base(acct))
                .expect("Init context doesn't have any gas");
        }

        // Create the fee collector account
        self.check_create_new_module_account(ctx, &self.fee_collector_key)
            .expect("Init context doesn't have any gas");
    }

    pub fn query_account<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        req: QueryAccountRequest,
    ) -> QueryAccountResponse {
        let auth_store = ctx.kv_store(&self.store_key);
        let key = create_auth_store_key(req.address);
        let account = auth_store.get(&key);

        if let Some(buf) = account {
            let account = Some(
                Account::decode::<Bytes>(buf.to_owned().into())
                    .ok()
                    .unwrap_or_corrupt(),
            );

            QueryAccountResponse { account }
        } else {
            QueryAccountResponse { account: None }
        }
    }

    fn get_next_account_number<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
    ) -> Result<u64, GasStoreErrors> {
        let mut auth_store = ctx.kv_store_mut(&self.store_key);

        // NOTE: The next available account number is what's stored in the KV store
        let acct_num = auth_store.get(&GLOBAL_ACCOUNT_NUMBER_KEY)?;

        let acct_num: u64 = match acct_num {
            None => 0, //initialize account numbers
            Some(num) => u64::decode::<Bytes>(num.to_owned().into())
                .ok()
                .unwrap_or_corrupt(),
        };

        let next_acct_num = acct_num + 1;
        auth_store.set(GLOBAL_ACCOUNT_NUMBER_KEY, next_acct_num.encode_to_vec())?;

        Ok(acct_num)
    }

    pub fn set_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        acct: Account,
    ) -> Result<(), GasStoreErrors> {
        let mut auth_store = ctx.kv_store_mut(&self.store_key);
        let key = create_auth_store_key(acct.get_address().to_owned());

        auth_store.set(key, acct.encode_vec().expect(IBC_ENCODE_UNWRAP))?; // TODO:IBC

        Ok(())
    }
}

fn create_auth_store_key(address: AccAddress) -> Vec<u8> {
    let mut auth_store_key: Vec<u8> = address.into();
    let mut prefix = Vec::new();
    prefix.extend(ACCOUNT_STORE_PREFIX);
    prefix.append(&mut auth_store_key);

    prefix
}
