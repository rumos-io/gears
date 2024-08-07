use core_types::any::google::Any;
use core_types::errors::CoreError as IbcError;
use core_types::tx::mode_info::ModeInfo;
use core_types::Protobuf;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;

use crate::crypto::public::DecodeError;
use crate::crypto::public::PublicKey;

pub mod inner {
    pub use core_types::signing::SignerInfo;
}

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

impl TryFrom<inner::SignerInfo> for SignerInfo {
    type Error = IbcError;

    fn try_from(raw: inner::SignerInfo) -> Result<Self, Self::Error> {
        let key = raw
            .public_key
            .map(TryInto::try_into)
            .transpose()
            .map_err(|e: DecodeError| IbcError::DecodeAny(e.to_string()))?;
        Ok(SignerInfo {
            public_key: key,
            mode_info: raw
                .mode_info
                .ok_or(core_types::errors::CoreError::MissingField(String::from(
                    "mode_info",
                )))?
                .try_into()?,
            sequence: raw.sequence,
        })
    }
}

impl From<SignerInfo> for inner::SignerInfo {
    fn from(info: SignerInfo) -> inner::SignerInfo {
        let key: Option<Any> = info.public_key.map(Into::into);

        Self {
            public_key: key,
            mode_info: Some(info.mode_info.into()),
            sequence: info.sequence,
        }
    }
}

impl Protobuf<inner::SignerInfo> for SignerInfo {}
