use crate::{AuthParamsKeeper, AuthsParams, GenesisState};
use bytes::Bytes;
use gears::error::IBC_ENCODE_UNWRAP;
use gears::params::keeper::ParamsKeeper;
use gears::params::ParamsSubspaceKey;
use gears::store::database::{ext::UnwrapCorrupt, Database};
use gears::store::{QueryableKVStore, StoreKey, TransactionalKVStore};
use gears::tendermint::types::proto::Protobuf as _;
use gears::types::address::AccAddress;
use gears::types::context::init::InitContext;
use gears::types::context::query::QueryContext;
use gears::types::context::QueryableContext;
use gears::types::query::account::QueryAccountRequest;
use gears::x::keepers::auth::AuthKeeper;
use gears::x::module::Module;
use gears::{
    error::AppError,
    types::{
        account::{Account, BaseAccount, ModuleAccount},
        context::TransactionalContext,
        query::account::QueryAccountResponse,
    },
};
use prost::Message;

const ACCOUNT_STORE_PREFIX: [u8; 1] = [1];
const GLOBAL_ACCOUNT_NUMBER_KEY: [u8; 19] = [
    103, 108, 111, 098, 097, 108, 065, 099, 099, 111, 117, 110, 116, 078, 117, 109, 098, 101, 114,
]; // "globalAccountNumber"

#[derive(Debug, Clone)]
pub struct Keeper<SK: StoreKey, PSK: ParamsSubspaceKey> {
    store_key: SK,
    auth_params_keeper: AuthParamsKeeper<SK, PSK>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> AuthKeeper<SK> for Keeper<SK, PSK> {
    type Params = AuthsParams;

    fn get_auth_params<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Self::Params {
        self.auth_params_keeper.get(ctx)
    }

    fn has_account<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        addr: &AccAddress,
    ) -> bool {
        let auth_store = ctx.kv_store(&self.store_key);
        let key = create_auth_store_key(addr.to_owned());
        auth_store.get(&key).is_some()
    }

    fn get_account<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        addr: &AccAddress,
    ) -> Option<Account> {
        let auth_store = ctx.kv_store(&self.store_key);
        let key = create_auth_store_key(addr.to_owned());
        let account = auth_store.get(&key);

        if let Some(buf) = account {
            let account = Account::decode::<Bytes>(buf.to_owned().into())
                .ok()
                .unwrap_or_corrupt();

            return Some(account);
        }

        None
    }

    fn set_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        acct: Account,
    ) {
        let mut auth_store = ctx.kv_store_mut(&self.store_key);
        let key = create_auth_store_key(acct.get_address().to_owned());

        auth_store.set(key, acct.encode_vec().expect(IBC_ENCODE_UNWRAP)); // TODO:IBC
    }

    fn create_new_base_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        addr: &AccAddress,
    ) {
        let acct = BaseAccount {
            address: addr.clone(),
            pub_key: None,
            account_number: self.get_next_account_number(ctx),
            sequence: 0,
        };

        self.set_account(ctx, Account::Base(acct))
    }

    fn check_create_new_module_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        module: &Module,
    ) {
        let addr = module.get_address();

        if self.has_account(ctx, &addr) {
        } else {
            let account = ModuleAccount {
                base_account: BaseAccount {
                    address: addr.clone(),
                    pub_key: None,
                    account_number: self.get_next_account_number(ctx),
                    sequence: 0,
                },
                name: module.get_name(),
                permissions: module.get_permissions(),
            };

            self.set_account(ctx, Account::Module(account))
        }
    }
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> Keeper<SK, PSK> {
    pub fn new(store_key: SK, params_keeper: ParamsKeeper<SK>, params_subspace_key: PSK) -> Self {
        let auth_params_keeper = AuthParamsKeeper {
            params_keeper,
            params_subspace_key,
        };
        Keeper {
            store_key,
            auth_params_keeper,
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
            acct.account_number = self.get_next_account_number(ctx);
            self.set_account(ctx, Account::Base(acct));
        }

        // Create the fee collector account
        self.check_create_new_module_account(ctx, &Module::FeeCollector);
    }

    pub fn query_account<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        req: QueryAccountRequest,
    ) -> Result<QueryAccountResponse, AppError> {
        let auth_store = ctx.kv_store(&self.store_key);
        let key = create_auth_store_key(req.address);
        let account = auth_store.get(&key);

        if let Some(buf) = account {
            let account = Account::decode::<Bytes>(buf.to_owned().into())
                .ok()
                .unwrap_or_corrupt();

            return Ok(QueryAccountResponse { account });
        }

        Err(AppError::AccountNotFound)
    }

    fn get_next_account_number<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
    ) -> u64 {
        let mut auth_store = ctx.kv_store_mut(&self.store_key);

        // NOTE: The next available account number is what's stored in the KV store
        let acct_num = auth_store.get(&GLOBAL_ACCOUNT_NUMBER_KEY);

        let acct_num: u64 = match acct_num {
            None => 0, //initialize account numbers
            Some(num) => u64::decode::<Bytes>(num.to_owned().into())
                .ok()
                .unwrap_or_corrupt(),
        };

        let next_acct_num = acct_num + 1;
        auth_store.set(GLOBAL_ACCOUNT_NUMBER_KEY, next_acct_num.encode_to_vec());

        acct_num
    }

    pub fn set_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        acct: Account,
    ) {
        let mut auth_store = ctx.kv_store_mut(&self.store_key);
        let key = create_auth_store_key(acct.get_address().to_owned());

        auth_store.set(key, acct.encode_vec().expect(IBC_ENCODE_UNWRAP)); // TODO:IBC
    }
}

fn create_auth_store_key(address: AccAddress) -> Vec<u8> {
    let mut auth_store_key: Vec<u8> = address.into();
    let mut prefix = Vec::new();
    prefix.extend(ACCOUNT_STORE_PREFIX);
    prefix.append(&mut auth_store_key);

    prefix
}
