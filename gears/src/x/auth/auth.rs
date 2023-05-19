use bytes::Bytes;
use database::DB;
use ibc_proto::{cosmos::auth::v1beta1::QueryAccountResponse, protobuf::Protobuf};
use prost::Message;
use proto_messages::cosmos::auth::v1beta1::{
    Account, BaseAccount, ModuleAccount, QueryAccountRequest,
};
use proto_types::AccAddress;
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    store::Store,
    types::{Context, QueryContext},
};

use super::Params;

const ACCOUNT_STORE_PREFIX: [u8; 1] = [1];
const GLOBAL_ACCOUNT_NUMBER_KEY: [u8; 19] = [
    103, 108, 111, 098, 097, 108, 065, 099, 099, 111, 117, 110, 116, 078, 117, 109, 098, 101, 114,
]; // "globalAccountNumber"

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct GenesisState {
    pub accounts: Vec<BaseAccount>,
    pub params: Params,
}

pub enum Module {
    FeeCollector,
}

//TODO: use properly typed QueryAccountResponse and QueryAccountRequest

impl Module {
    pub fn get_address(&self) -> AccAddress {
        match self {
            Module::FeeCollector => {
                //TODO: construct address from Vec<u8> + make address constant
                //TODO: where do these addresses come from?
                AccAddress::from_bech32("cosmos17xpfvakm2amg962yls6f84z3kell8c5lserqta")
                    .expect("hard coded address is valid")
            }
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            Module::FeeCollector => "fee_collector".into(),
        }
    }

    pub fn get_permissions(&self) -> Vec<String> {
        match self {
            Module::FeeCollector => vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Auth {}

impl Auth {
    pub fn init_genesis<T: DB>(ctx: &mut Context<T>, genesis: GenesisState) {
        //TODO: sdk sanitizes accounts
        Params::set(ctx, genesis.params);

        for mut acct in genesis.accounts {
            acct.account_number = Auth::get_next_account_number(ctx);
            Auth::set_account(ctx, Account::Base(acct));
        }

        // Create the fee collector account
        Auth::check_create_new_module_account(ctx, &Module::FeeCollector);
    }

    pub fn query_account<T: DB>(
        ctx: &QueryContext<T>,
        req: QueryAccountRequest,
    ) -> Result<QueryAccountResponse, AppError> {
        let auth_store = ctx.get_kv_store(Store::Auth);
        let key = create_auth_store_key(req.address);
        let account = auth_store.get(&key);

        if let Some(buf) = account {
            let account = Account::decode::<Bytes>(buf.to_owned().into())
                .expect("invalid data in database - possible database corruption");

            return Ok(QueryAccountResponse {
                account: Some(account.into()),
            });
        }

        return Err(AppError::AccountNotFound);
    }

    fn get_next_account_number<T: DB>(ctx: &mut Context<T>) -> u64 {
        let auth_store = ctx.get_mutable_kv_store(Store::Auth);

        // NOTE: The next available account number is what's stored in the KV store
        let acct_num = auth_store.get(&GLOBAL_ACCOUNT_NUMBER_KEY);

        let acct_num: u64 = match acct_num {
            None => 0, //initialize account numbers
            Some(num) => u64::decode::<Bytes>(num.to_owned().into())
                .expect("invalid data in database - possible database corruption"),
        };

        let next_acct_num = acct_num + 1;
        auth_store.set(
            GLOBAL_ACCOUNT_NUMBER_KEY.clone().into(),
            next_acct_num.encode_to_vec(),
        );

        return acct_num;
    }

    pub fn has_account<T: DB>(ctx: &Context<T>, addr: &AccAddress) -> bool {
        let auth_store = ctx.get_kv_store(Store::Auth);
        let key = create_auth_store_key(addr.to_owned());
        auth_store.get(&key).is_some()
    }

    pub fn set_account<T: DB>(ctx: &mut Context<T>, acct: Account) {
        let auth_store = ctx.get_mutable_kv_store(Store::Auth);
        let key = create_auth_store_key(acct.get_address().to_owned());

        auth_store.set(key, acct.encode_vec());
    }

    pub fn get_account<T: DB>(ctx: &Context<T>, addr: &AccAddress) -> Option<Account> {
        let auth_store = ctx.get_kv_store(Store::Auth);
        let key = create_auth_store_key(addr.to_owned());
        let account = auth_store.get(&key);

        if let Some(buf) = account {
            let account = Account::decode::<Bytes>(buf.to_owned().into())
                .expect("invalid data in database - possible database corruption");

            return Some(account);
        }

        return None;
    }

    /// Overwrites existing account
    pub fn create_new_base_account<T: DB>(ctx: &mut Context<T>, addr: &AccAddress) {
        let acct = BaseAccount {
            address: addr.clone(),
            pub_key: None,
            account_number: Auth::get_next_account_number(ctx),
            sequence: 0,
        };

        Auth::set_account(ctx, Account::Base(acct))
    }

    /// Creates a new module account if it doesn't already exist
    pub fn check_create_new_module_account<T: DB>(ctx: &mut Context<T>, module: &Module) {
        let addr = module.get_address();

        if Auth::has_account(ctx, &addr) {
            return;
        } else {
            let account = ModuleAccount {
                base_account: BaseAccount {
                    address: addr.clone(),
                    pub_key: None,
                    account_number: Auth::get_next_account_number(ctx),
                    sequence: 0,
                },
                name: module.get_name(),
                permissions: module.get_permissions(),
            };

            Auth::set_account(ctx, Account::Module(account))
        }
    }
}

fn create_auth_store_key(address: AccAddress) -> Vec<u8> {
    let mut auth_store_key: Vec<u8> = address.into();
    let mut prefix = Vec::new();
    prefix.extend(ACCOUNT_STORE_PREFIX);
    prefix.append(&mut auth_store_key);

    return prefix;
}

#[cfg(test)]
mod tests {

    use database::MemDB;
    use proto_messages::cosmos::crypto::secp256k1::v1beta1::RawPubKey;
    use proto_messages::cosmos::{
        crypto::secp256k1::v1beta1::PubKey as Secp256k1PubKey, tx::v1beta1::PublicKey,
    };

    use super::*;
    use crate::store::MultiStore;
    use crate::types::InitContext;

    #[test]
    fn address_store_key_works() {
        let expected = vec![1, 97, 98, 99, 100];
        let acc_address = AccAddress::try_from(vec![97, 98, 99, 100]).unwrap();
        let res = create_auth_store_key(acc_address);

        assert_eq!(expected, res);
    }

    #[test]
    fn query_account_on_unseen_account_works() {
        let expected = AppError::AccountNotFound;

        let req = QueryAccountRequest {
            address: AccAddress::from_bech32(
                "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".into(),
            )
            .unwrap(),
        };

        let db = MemDB::new();
        let store = MultiStore::new(db);
        let ctx = QueryContext::new(&store, 0);
        let res = Auth::query_account(&ctx, req).unwrap_err();

        assert_eq!(expected, res);
    }

    #[test]
    fn get_next_account_number_init_works() {
        let db = MemDB::new();
        let expected = 0;
        let mut store = MultiStore::new(db);
        let mut ctx = InitContext::new(&mut store, 0, "".into());
        let acct_num = Auth::get_next_account_number(&mut ctx.as_any());

        assert_eq!(expected, acct_num);
    }

    #[test]
    fn get_next_account_number_works() {
        let expected = 5038438478387;
        let db = MemDB::new();
        let mut store = MultiStore::new(db);
        let auth_store = store.get_mutable_kv_store(Store::Auth);

        auth_store.set(
            GLOBAL_ACCOUNT_NUMBER_KEY.clone().into(),
            expected.encode_to_vec(),
        );

        let mut ctx = InitContext::new(&mut store, 0, "".into());
        let acct_num = Auth::get_next_account_number(&mut ctx.as_any());

        assert_eq!(expected, acct_num);

        // check account number is being incremented
        let acct_num = Auth::get_next_account_number(&mut ctx.as_any());
        assert_eq!(expected + 1, acct_num);
    }

    #[test]
    fn set_public_key_works() {
        let address =
            AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".into())
                .unwrap();

        let key = hex::decode("02950e1cdfcb133d6024109fd489f734eeb4502418e538c28481f22bce276f248c")
            .unwrap();
        let raw = RawPubKey { key };
        let key: Secp256k1PubKey = raw.try_into().unwrap();

        let mut acct = Account::Base(BaseAccount {
            address,
            pub_key: None,
            account_number: 1,
            sequence: 1,
        });

        assert_eq!(acct.get_public_key(), &None);

        acct.set_public_key(PublicKey::Secp256k1(key.clone()));

        assert_eq!(acct.get_public_key(), &Some(PublicKey::Secp256k1(key)));
    }
}
