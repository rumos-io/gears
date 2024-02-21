use ibc_proto::{
    cosmos::tx::v1beta1::SignerInfo as RawSignerInfo, google::protobuf::Any, protobuf::Protobuf,
};

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;

use crate::error::Error;

use super::mode_info::ModeInfo;
use super::public_key::PublicKey;

/// SignerInfo describes the public key and signing mode of a single top-level
/// signer.
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SignerInfo {
    /// public_key is the public key of the signer. It is optional for accounts
    /// that already exist in state. If unset, the verifier can use the required \
    /// signer address for this position and lookup the public key.
    pub public_key: Option<PublicKey>,
    /// mode_info describes the signing mode of the signer and is a nested
    /// structure to support nested multisig pubkey's
    pub mode_info: ModeInfo, // TODO: this isn't serializing correctly
    /// sequence is the sequence of the account, which describes the
    /// number of committed transactions signed by a given address. It is used to
    /// prevent replay attacks.
    #[serde_as(as = "DisplayFromStr")]
    pub sequence: u64,
}

impl TryFrom<RawSignerInfo> for SignerInfo {
    type Error = Error;

    fn try_from(raw: RawSignerInfo) -> Result<Self, Self::Error> {
        let key: Option<PublicKey> = match raw.public_key {
            Some(any) => Some(any.try_into()?),
            None => None,
        };
        Ok(SignerInfo {
            public_key: key,
            mode_info: raw
                .mode_info
                .ok_or(Error::MissingField(String::from("mode_info")))?
                .try_into()?,
            sequence: raw.sequence,
        })
    }
}

impl From<SignerInfo> for RawSignerInfo {
    fn from(info: SignerInfo) -> RawSignerInfo {
        let key: Option<Any> = info.public_key.map(|key| key.into());

        RawSignerInfo {
            public_key: key,
            mode_info: Some(info.mode_info.into()),
            sequence: info.sequence,
        }
    }
}

impl Protobuf<RawSignerInfo> for SignerInfo {}
