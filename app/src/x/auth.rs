use ibc_proto::{
    cosmos::auth::v1beta1::{BaseAccount, QueryAccountRequest, QueryAccountResponse},
    google::protobuf::Any,
};
use prost::Message;

use crate::{
    baseapp::AUTH_STORE_PREFIX,
    error::AppError,
    types::{AccAddress, Context},
};

const ADDRESS_STORE_KEY_PREFIX: [u8; 1] = [1];

#[derive(Debug, Clone)]
pub struct Auth {}

impl Auth {
    //TODO: return error if address is invalid
    pub fn query_account(
        ctx: &Context,
        req: QueryAccountRequest,
    ) -> Result<QueryAccountResponse, AppError> {
        let address = AccAddress::from_bech32(&req.address)?;

        let auth_store = ctx.get_store().get_sub_store(AUTH_STORE_PREFIX.into());

        let key = address_store_key(address);
        let account = auth_store.get(&key);

        match account {
            Some(account) => Ok(QueryAccountResponse {
                account: Some(Any {
                    type_url: "/cosmos.auth.v1beta1.BaseAccount".to_string(),
                    value: account,
                }),
            }),
            None => {
                let account = BaseAccount {
                    address: req.address,
                    pub_key: None,
                    account_number: 0,
                    sequence: 1,
                };
                Ok(QueryAccountResponse {
                    account: Some(Any {
                        type_url: "/cosmos.auth.v1beta1.BaseAccount".to_string(),
                        value: account.encode_to_vec(),
                    }),
                })
            }
        }
    }
}

fn address_store_key(addr: AccAddress) -> Vec<u8> {
    let mut addr: Vec<u8> = addr.into();
    let mut prefix = Vec::new();

    prefix.extend(ADDRESS_STORE_KEY_PREFIX);
    prefix.append(&mut addr);

    return prefix;
}

#[cfg(test)]
mod tests {

    use crate::store::Store;

    use super::*;

    #[test]
    fn address_store_key_works() {
        let expected = vec![1, 97, 98, 99, 100];
        let acc_address = AccAddress::try_from(vec![97, 98, 99, 100]).unwrap();
        let res = address_store_key(acc_address);

        assert_eq!(expected, res);
    }

    #[test]
    fn query_account_on_unseen_account_works() {
        let expected = QueryAccountResponse {
            account: Some(Any {
                type_url: "/cosmos.auth.v1beta1.BaseAccount".to_string(),
                value: BaseAccount {
                    address: "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".to_string(),
                    pub_key: None,
                    account_number: 0,
                    sequence: 1,
                }
                .encode_to_vec(),
            }),
        };

        let req = QueryAccountRequest {
            address: "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".into(),
        };

        let store = Store::new();
        let ctx = Context::new(store);

        let res = Auth::query_account(&ctx, req).unwrap();

        assert_eq!(expected, res);
    }
}
