use core_types::errors::CoreError as IbcError;
use core_types::Protobuf;
use core_types::{any::google::Any, serializers::serialize_number_to_string};
use prost::bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_number_from_string;

use crate::crypto::public::{DecodeError, PublicKey};

use super::address::AccAddress;

pub mod inner {
    pub use core_types::account::BaseAccount;
    pub use core_types::account::ModuleAccount;
}

/// BaseAccount defines a base account type. It contains all the necessary fields
/// for basic account functionality. Any custom account type should extend this
/// type for additional functionality (e.g. vesting).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BaseAccount {
    pub address: AccAddress,
    pub pub_key: Option<PublicKey>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(serialize_with = "serialize_number_to_string")]
    pub account_number: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(serialize_with = "serialize_number_to_string")]
    pub sequence: u64,
}

impl TryFrom<inner::BaseAccount> for BaseAccount {
    type Error = IbcError;

    fn try_from(raw: inner::BaseAccount) -> Result<Self, Self::Error> {
        let address = AccAddress::from_bech32(&raw.address)
            .map_err(|e| core_types::errors::CoreError::DecodeAddress(e.to_string()))?;

        let pub_key = raw
            .pub_key
            .map(TryInto::try_into)
            .transpose()
            .map_err(|e: DecodeError| IbcError::DecodeAny(e.to_string()))?;

        Ok(BaseAccount {
            address,
            pub_key,
            account_number: raw.account_number,
            sequence: raw.sequence,
        })
    }
}

impl From<BaseAccount> for inner::BaseAccount {
    fn from(
        BaseAccount {
            address,
            pub_key,
            account_number,
            sequence,
        }: BaseAccount,
    ) -> inner::BaseAccount {
        Self {
            address: address.into(),
            pub_key: pub_key.map(Into::into),
            account_number,
            sequence,
        }
    }
}

impl Protobuf<inner::BaseAccount> for BaseAccount {}

/// ModuleAccount defines an account for modules that holds coins on a pool.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ModuleAccount {
    pub base_account: BaseAccount,
    pub name: String,
    pub permissions: Vec<String>,
}

impl TryFrom<inner::ModuleAccount> for ModuleAccount {
    type Error = core_types::errors::CoreError;

    fn try_from(raw: inner::ModuleAccount) -> Result<Self, Self::Error> {
        let base_account = match raw.base_account {
            Some(base) => base.try_into()?,
            None => {
                return Err(core_types::errors::CoreError::DecodeGeneral(
                    "missing base account field".into(),
                ))
            }
        };

        Ok(ModuleAccount {
            base_account,
            name: raw.name,
            permissions: raw.permissions,
        })
    }
}

impl From<ModuleAccount> for inner::ModuleAccount {
    fn from(acct: ModuleAccount) -> inner::ModuleAccount {
        Self {
            base_account: Some(acct.base_account.into()),
            name: acct.name,
            permissions: acct.permissions,
        }
    }
}

impl Protobuf<inner::ModuleAccount> for ModuleAccount {}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum Account {
    #[serde(rename = "/cosmos.auth.v1beta1.BaseAccount")]
    Base(BaseAccount),
    #[serde(rename = "/cosmos.auth.v1beta1.ModuleAccount")]
    Module(ModuleAccount),
}

impl Account {
    pub fn get_public_key(&self) -> Option<&PublicKey> {
        match self {
            Account::Base(acct) => acct.pub_key.as_ref(),
            Account::Module(acct) => acct.base_account.pub_key.as_ref(),
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

    pub fn set_account_number(&mut self, number: u64) {
        match self {
            Account::Base(acct) => acct.account_number = number,
            Account::Module(acct) => acct.base_account.account_number = number,
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

    pub fn has_permissions(&self, perm: impl AsRef<str>) -> bool {
        match self {
            Account::Base(_) => false, // TODO:NOW
            Account::Module(var) => var.permissions.iter().any(|this| this == perm.as_ref()),
        }
    }
}

impl TryFrom<Any> for Account {
    type Error = core_types::errors::CoreError;

    fn try_from(any: Any) -> Result<Self, Self::Error> {
        match any.type_url.as_str() {
            "/cosmos.auth.v1beta1.BaseAccount" => {
                let base = BaseAccount::decode::<Bytes>(any.value.into())
                    .map_err(|e| core_types::errors::CoreError::DecodeGeneral(e.to_string()))?;
                Ok(Account::Base(base))
            }
            "/cosmos.auth.v1beta1.ModuleAccount" => {
                let module = ModuleAccount::decode::<Bytes>(any.value.into())
                    .map_err(|e| core_types::errors::CoreError::DecodeGeneral(e.to_string()))?;
                Ok(Account::Module(module))
            }
            _ => Err(core_types::errors::CoreError::DecodeAny(format!(
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

#[cfg(test)]
mod tests {

    use core_types::Protobuf;
    use extensions::testing::UnwrapTesting;

    use crate::types::{account::BaseAccount, address::AccAddress};

    #[test]
    fn base_account_encode_works() {
        let account = BaseAccount {
            address: AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux")
                .unwrap_test(),
            pub_key: None,
            account_number: 0,
            sequence: 0,
        };

        let exp = "0a2d636f736d6f73317379617679326e706679743974636e63647473647a66376b6e79396c68373737706168757578";

        assert_eq!(exp, data_encoding::HEXLOWER.encode(&account.encode_vec()))
    }
}
