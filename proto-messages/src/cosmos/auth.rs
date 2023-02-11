pub mod v1beta1 {

    use ibc_proto::{
        cosmos::auth::v1beta1::{BaseAccount as RawBaseAccount, ModuleAccount as RawModuleAccount},
        protobuf::Protobuf,
    };
    use proto_types::AccAddress;

    use crate::{cosmos::tx::v1beta1::PublicKey, Error};

    /// BaseAccount defines a base account type. It contains all the necessary fields
    /// for basic account functionality. Any custom account type should extend this
    /// type for additional functionality (e.g. vesting).
    #[derive(Clone, PartialEq)]
    pub struct BaseAccount {
        pub address: proto_types::AccAddress,
        pub pub_key: Option<PublicKey>,
        pub account_number: u64,
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
            let pub_key = match acct.pub_key {
                Some(key) => Some(key.into()),
                None => None,
            };
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
    #[derive(Clone, PartialEq)]
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
}
