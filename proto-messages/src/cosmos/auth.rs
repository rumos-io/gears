pub mod v1beta1 {

    use ibc_proto::{
        cosmos::auth::v1beta1::{
            BaseAccount as RawBaseAccount, ModuleAccount as RawModuleAccount,
            QueryAccountRequest as RawQueryAccountRequest,
            QueryAccountResponse as RawQueryAccountResponse,
        },
        google::protobuf::Any, Protobuf,
    };
    use prost::bytes::Bytes;
    use proto_types::AccAddress;
    use serde::{Deserialize, Serialize};
    use serde_aux::prelude::deserialize_number_from_string;

    use crate::{
        cosmos::tx::v1beta1::public_key::PublicKey, utils::serialize_number_to_string, Error,
    };

    /// BaseAccount defines a base account type. It contains all the necessary fields
    /// for basic account functionality. Any custom account type should extend this
    /// type for additional functionality (e.g. vesting).
    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    pub struct BaseAccount {
        pub address: proto_types::AccAddress,
        pub pub_key: Option<PublicKey>,
        #[serde(deserialize_with = "deserialize_number_from_string")]
        #[serde(serialize_with = "serialize_number_to_string")]
        pub account_number: u64,
        #[serde(deserialize_with = "deserialize_number_from_string")]
        #[serde(serialize_with = "serialize_number_to_string")]
        pub sequence: u64,
    }

    impl TryFrom<RawBaseAccount> for BaseAccount {
        type Error = Error;

        fn try_from(raw: RawBaseAccount) -> Result<Self, Self::Error> {
            let address = AccAddress::from_bech32(&raw.address)
                .map_err(|e| Error::DecodeAddress(e.to_string()))?;

            let pub_key = match raw.pub_key {
                Some(key) => {
                    let key = key.try_into()?;
                    Some(key)
                }
                None => None,
            };

            Ok(BaseAccount {
                address,
                pub_key,
                account_number: raw.account_number,
                sequence: raw.sequence,
            })
        }
    }

    impl From<BaseAccount> for RawBaseAccount {
        fn from(acct: BaseAccount) -> RawBaseAccount {
            let pub_key = acct.pub_key.map(|key| key.into());
            RawBaseAccount {
                address: acct.address.into(),
                pub_key,
                account_number: acct.account_number,
                sequence: acct.sequence,
            }
        }
    }

    impl Protobuf<RawBaseAccount> for BaseAccount {}

    /// ModuleAccount defines an account for modules that holds coins on a pool.
    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    pub struct ModuleAccount {
        pub base_account: BaseAccount,
        pub name: String,
        pub permissions: Vec<String>,
    }

    impl TryFrom<RawModuleAccount> for ModuleAccount {
        type Error = Error;

        fn try_from(raw: RawModuleAccount) -> Result<Self, Self::Error> {
            let base_account = match raw.base_account {
                Some(base) => base.try_into()?,
                None => return Err(Error::DecodeGeneral("missing base account field".into())),
            };

            Ok(ModuleAccount {
                base_account,
                name: raw.name,
                permissions: raw.permissions,
            })
        }
    }

    impl From<ModuleAccount> for RawModuleAccount {
        fn from(acct: ModuleAccount) -> RawModuleAccount {
            RawModuleAccount {
                base_account: Some(acct.base_account.into()),
                name: acct.name,
                permissions: acct.permissions,
            }
        }
    }

    impl Protobuf<RawModuleAccount> for ModuleAccount {}

    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    pub enum Account {
        Base(BaseAccount),
        Module(ModuleAccount),
    }

    impl Account {
        pub fn get_public_key(&self) -> &Option<PublicKey> {
            match self {
                Account::Base(acct) => &acct.pub_key,
                Account::Module(acct) => &acct.base_account.pub_key,
            }
        }

        pub fn get_address(&self) -> &AccAddress {
            match self {
                Account::Base(base) => &base.address,
                Account::Module(module) => &module.base_account.address,
            }
        }

        pub fn set_public_key(&mut self, key: PublicKey) {
            match self {
                Account::Base(acct) => acct.pub_key = Some(key),
                Account::Module(acct) => acct.base_account.pub_key = Some(key),
            }
        }

        pub fn increment_sequence(&mut self) {
            match self {
                Account::Base(acct) => acct.sequence += 1,
                Account::Module(acct) => acct.base_account.sequence += 1,
            }
        }

        pub fn get_sequence(&self) -> u64 {
            match self {
                Account::Base(acct) => acct.sequence,
                Account::Module(acct) => acct.base_account.sequence,
            }
        }

        pub fn get_account_number(&self) -> u64 {
            match self {
                Account::Base(acct) => acct.account_number,
                Account::Module(acct) => acct.base_account.account_number,
            }
        }
    }

    impl TryFrom<Any> for Account {
        type Error = Error;

        fn try_from(any: Any) -> Result<Self, Self::Error> {
            match any.type_url.as_str() {
                "/cosmos.auth.v1beta1.BaseAccount" => {
                    let base = BaseAccount::decode::<Bytes>(any.value.into())
                        .map_err(|e| Error::DecodeGeneral(e.to_string()))?;
                    Ok(Account::Base(base))
                }
                "/cosmos.auth.v1beta1.ModuleAccount" => {
                    let module = ModuleAccount::decode::<Bytes>(any.value.into())
                        .map_err(|e| Error::DecodeGeneral(e.to_string()))?;
                    Ok(Account::Module(module))
                }
                _ => Err(Error::DecodeAny(format!(
                    "account type not recognized: {}",
                    any.type_url
                ))),
            }
        }
    }

    impl From<Account> for Any {
        fn from(account: Account) -> Self {
            match account {
                Account::Base(base) => Any {
                    type_url: "/cosmos.auth.v1beta1.BaseAccount".to_string(),
                    value: base.encode_vec(),
                },
                Account::Module(module) => Any {
                    type_url: "/cosmos.auth.v1beta1.ModuleAccount".to_string(),
                    value: module.encode_vec(),
                },
            }
        }
    }

    impl Protobuf<Any> for Account {}

    /// QueryAccountRequest is the request type for the Query/Account RPC method.
    #[derive(Clone, PartialEq)]
    pub struct QueryAccountRequest {
        /// address defines the address to query for.
        pub address: proto_types::AccAddress,
    }

    impl TryFrom<RawQueryAccountRequest> for QueryAccountRequest {
        type Error = Error;

        fn try_from(raw: RawQueryAccountRequest) -> Result<Self, Self::Error> {
            let address = AccAddress::from_bech32(&raw.address)
                .map_err(|e| Error::DecodeAddress(e.to_string()))?;

            Ok(QueryAccountRequest { address })
        }
    }

    impl From<QueryAccountRequest> for RawQueryAccountRequest {
        fn from(query: QueryAccountRequest) -> RawQueryAccountRequest {
            RawQueryAccountRequest {
                address: query.address.to_string(),
            }
        }
    }

    impl Protobuf<RawQueryAccountRequest> for QueryAccountRequest {}

    /// QueryAccountResponse is the response type for the Query/Account RPC method.
    #[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
    pub struct QueryAccountResponse {
        /// account defines the account of the corresponding address.
        pub account: Account,
    }

    impl TryFrom<RawQueryAccountResponse> for QueryAccountResponse {
        type Error = Error;

        fn try_from(raw: RawQueryAccountResponse) -> Result<Self, Self::Error> {
            let account = raw
                .account
                .ok_or(Error::MissingField("account".into()))?
                .try_into()?;

            Ok(QueryAccountResponse { account })
        }
    }

    impl From<QueryAccountResponse> for RawQueryAccountResponse {
        fn from(query: QueryAccountResponse) -> RawQueryAccountResponse {
            RawQueryAccountResponse {
                account: Some(query.account.into()),
            }
        }
    }

    impl Protobuf<RawQueryAccountResponse> for QueryAccountResponse {}
}

#[cfg(test)]
mod tests {

    use ibc_proto::Protobuf;
    use proto_types::AccAddress;

    use super::v1beta1::*;

    #[test]
    fn base_account_encode_works() {
        let account = BaseAccount {
            address: AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux")
                .unwrap(),
            pub_key: None,
            account_number: 0,
            sequence: 0,
        };

        let exp = "0a2d636f736d6f73317379617679326e706679743974636e63647473647a66376b6e79396c68373737706168757578";

        assert_eq!(exp, hex::encode(account.encode_vec()))
    }
}
