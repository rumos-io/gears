use bytes::Bytes;
use ibc_proto::{cosmos::auth::v1beta1::QueryAccountResponse, google::protobuf::Any};
use prost::Message;
use proto_types::AccAddress;

use crate::{
    baseapp::AUTH_STORE_PREFIX,
    error::AppError,
    types::{
        proto::{BaseAccount, QueryAccountRequest},
        Context,
    },
};

const ACCOUNT_STORE_PREFIX: [u8; 1] = [1];
const GLOBAL_ACCOUNT_NUMBER_KEY: [u8; 19] = [
    103, 108, 111, 098, 097, 108, 065, 099, 099, 111, 117, 110, 116, 078, 117, 109, 098, 101, 114,
]; // "globalAccountNumber"

pub struct GenesisState {
    pub accounts: Vec<BaseAccount>,
}

#[derive(Debug, Clone)]
pub struct Auth {}

impl Auth {
    pub fn init_genesis(ctx: &mut Context, genesis: GenesisState) -> Result<(), AppError> {
        for mut acct in genesis.accounts {
            acct.account_number = Auth::get_next_account_number(ctx);
            Auth::set_account(ctx, acct);
        }

        Ok(())
    }

    pub fn query_account(
        ctx: &Context,
        req: QueryAccountRequest,
    ) -> Result<QueryAccountResponse, AppError> {
        let auth_store = ctx
            .get_multi_store()
            .get_immutable_sub_store(AUTH_STORE_PREFIX.into());
        let key = create_auth_store_key(req.address);
        let account = auth_store.get(&key);

        match account {
            Some(account) => Ok(QueryAccountResponse {
                account: Some(Any {
                    type_url: "/cosmos.auth.v1beta1.BaseAccount".to_string(),
                    value: account.to_owned(),
                }),
            }),
            None => Err(AppError::AccountNotFound),
        }
    }

    fn get_next_account_number(ctx: &mut Context) -> u64 {
        let mut auth_store = ctx
            .get_mutable_store()
            .get_mutable_sub_store(AUTH_STORE_PREFIX.into());

        // NOTE: The next available account number is what's stored in the KV store
        let acct_num = auth_store.get(&GLOBAL_ACCOUNT_NUMBER_KEY);

        let acct_num: u64 = match acct_num {
            None => 0, //initialize account numbers
            Some(num) => u64::decode::<Bytes>(num.to_owned().into()).unwrap(),
        };

        let next_acct_num = acct_num + 1;
        auth_store.set(
            GLOBAL_ACCOUNT_NUMBER_KEY.clone().into(),
            next_acct_num.encode_to_vec(),
        );

        return acct_num;
    }

    pub fn has_account(ctx: &Context, addr: &AccAddress) -> bool {
        let auth_store = ctx
            .get_multi_store()
            .get_immutable_sub_store(AUTH_STORE_PREFIX.into());
        let key = create_auth_store_key(addr.to_owned());
        auth_store.get(&key).is_some()
    }

    fn set_account(ctx: &mut Context, acct: BaseAccount) {
        let mut auth_store = ctx
            .get_mutable_store()
            .get_mutable_sub_store(AUTH_STORE_PREFIX.into());
        let key = create_auth_store_key(acct.address.to_owned());
        auth_store.set(key, acct.encode_to_vec());
    }

    pub fn new_account(ctx: &mut Context, addr: &AccAddress) {
        let acct = BaseAccount {
            address: addr.clone(),
            pub_key: None,
            account_number: Auth::get_next_account_number(ctx),
            sequence: 0,
        };

        Auth::set_account(ctx, acct)
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

    use super::*;
    use crate::store::MultiStore;

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

        let store = MultiStore::new();
        let ctx = Context::new(store);
        let res = Auth::query_account(&ctx, req).unwrap_err();

        assert_eq!(expected, res);
    }

    #[test]
    fn get_next_account_number_init_works() {
        let expected = 0;
        let store = MultiStore::new();
        let mut ctx = Context::new(store);
        let acct_num = Auth::get_next_account_number(&mut ctx);

        assert_eq!(expected, acct_num);
    }

    #[test]
    fn get_next_account_number_works() {
        let expected = 5038438478387;
        let mut store = MultiStore::new();
        let mut auth_store = store.get_mutable_sub_store(AUTH_STORE_PREFIX.into());

        auth_store.set(
            GLOBAL_ACCOUNT_NUMBER_KEY.clone().into(),
            expected.encode_to_vec(),
        );

        let mut ctx = Context::new(store);
        let acct_num = Auth::get_next_account_number(&mut ctx);

        assert_eq!(expected, acct_num);

        // check account number is being incremented
        let acct_num = Auth::get_next_account_number(&mut ctx);
        assert_eq!(expected + 1, acct_num);
    }
}
