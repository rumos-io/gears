use ibc_proto::{
    cosmos::auth::v1beta1::{BaseAccount, QueryAccountRequest, QueryAccountResponse},
    google::protobuf::Any,
};
use prost::Message;

use crate::{store::Store, types::AccAddress};

const ADDRESS_STORE_KEY_PREFIX: [u8; 1] = [1];

#[derive(Debug, Clone)]
pub struct Auth {
    store: Store,
}

impl Auth {
    pub fn new(store: Store) -> Self {
        Auth { store }
    }

    pub fn query_account(&self, req: QueryAccountRequest) -> QueryAccountResponse {
        let address = AccAddress::from_bech32(&req.address);

        let address = match address {
            Ok(address) => address,
            Err(_) => return QueryAccountResponse { account: None },
        };

        let key = address_store_key(address);
        let account = self.store.get(&key);

        match account {
            Some(account) => QueryAccountResponse {
                account: Some(Any {
                    type_url: "/cosmos.auth.v1beta1.BaseAccount".to_string(),
                    value: account,
                }),
            },
            None => {
                let account = BaseAccount {
                    address: req.address,
                    pub_key: None,
                    account_number: 0,
                    sequence: 1,
                };
                QueryAccountResponse {
                    account: Some(Any {
                        type_url: "/cosmos.auth.v1beta1.BaseAccount".to_string(),
                        value: account.encode_to_vec(),
                    }),
                }
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

    use super::*;

    #[test]
    fn address_store_key_works() {
        let expected = vec![1, 97, 98, 99, 100];
        let acc_address = AccAddress::try_from(vec![97, 98, 99, 100]).unwrap();
        let res = address_store_key(acc_address);

        assert_eq!(expected, res);
    }

    #[test]
    fn new_account_query_account_works() {
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
        let auth = Auth::new(store);

        let res = auth.query_account(req);

        assert_eq!(expected, res);
    }
}
