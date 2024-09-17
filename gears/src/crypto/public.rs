use address::AccAddress;
use bytes::Bytes;
use core_types::any::google::Any;
use core_types::Protobuf;
use serde::{Deserialize, Serialize};

use super::{ed25519::Ed25519PubKey, secp256k1::Secp256k1PubKey};

pub type SigningError = secp256k1::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
#[error("invalid key: {0}")]
pub struct DecodeError(pub String);

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum PublicKey {
    #[serde(rename = "/cosmos.crypto.secp256k1.PubKey")]
    Secp256k1(Secp256k1PubKey),
    //Secp256r1(Vec<u8>),
    #[serde(rename = "/cosmos.crypto.ed25519.PubKey")]
    Ed25519(Ed25519PubKey),
    //Multisig(Vec<u8>),
}

impl PublicKey {
    pub fn verify_signature(
        &self,
        message: impl AsRef<[u8]>,
        signature: impl AsRef<[u8]>,
    ) -> Result<(), SigningError> {
        match self {
            PublicKey::Secp256k1(key) => key.verify_signature(message, signature),
            PublicKey::Ed25519(key) => key.verify_signature(message, signature),
        }
    }

    pub fn get_address(&self) -> AccAddress {
        match self {
            PublicKey::Secp256k1(key) => key.get_address(),
            PublicKey::Ed25519(key) => key.get_address(),
        }
    }
}

impl TryFrom<Any> for PublicKey {
    type Error = DecodeError;

    fn try_from(any: Any) -> Result<Self, Self::Error> {
        match any.type_url.as_str() {
            "/cosmos.crypto.secp256k1.PubKey" => {
                let key = Secp256k1PubKey::decode::<Bytes>(any.value.into())
                    .map_err(|e| DecodeError(e.to_string()))?;
                Ok(Self::Secp256k1(key))
            }
            "/cosmos.crypto.ed25519.PubKey" => {
                let key = Ed25519PubKey::decode::<Bytes>(any.value.into())
                    .map_err(|e| DecodeError(e.to_string()))?;
                Ok(Self::Ed25519(key))
            }

            _ => Err(DecodeError(format!(
                "Key type not recognized: {}",
                any.type_url
            ))),
        }
    }
}

impl From<PublicKey> for Any {
    fn from(key: PublicKey) -> Self {
        match key {
            PublicKey::Secp256k1(key) => Any {
                type_url: "/cosmos.crypto.secp256k1.PubKey".to_string(),
                value: key.encode_vec(),
            },
            PublicKey::Ed25519(key) => Any {
                type_url: "/cosmos.crypto.ed25519.PubKey".to_string(),
                value: key.encode_vec(),
            },
        }
    }
}

use tendermint::informal::PublicKey as InformalPublicKey;
use tendermint::types::proto::crypto::PublicKey as TendermintPublicKey;

/// This is needed for compatibility with the Cosmos SDK which uses the application
/// key in some modules instead of the consensus key.
impl From<TendermintPublicKey> for PublicKey {
    fn from(key: TendermintPublicKey) -> Self {
        match key {
            TendermintPublicKey::Ed25519(value) => PublicKey::Ed25519(value.try_into().unwrap()), //TODO: unwrap can be removed once tendermint type checks are in place (probably the safest thing to do is to expose the underlying key type)
            TendermintPublicKey::Secp256k1(value) => {
                PublicKey::Secp256k1(value.try_into().unwrap()) //TODO: unwrap can be removed once tendermint type checks are in place (probably the safest thing to do is to expose the underlying key type)
            }
        }
    }
}

impl From<InformalPublicKey> for PublicKey {
    fn from(key: InformalPublicKey) -> Self {
        match key {
            InformalPublicKey::Ed25519(value) => {
                PublicKey::Ed25519(value.as_bytes().to_owned().try_into().unwrap())
            }
            // TODO: under feature secp256k1
            _ => todo!(), // InformalPublicKey::Secp256k1(value) => {
                          //     PublicKey::Secp256k1(value.as_bytes().to_owned().try_into().unwrap())
                          // }
        }
    }
}

impl From<PublicKey> for TendermintPublicKey {
    fn from(key: PublicKey) -> Self {
        match key {
            PublicKey::Ed25519(value) => TendermintPublicKey::Ed25519(value.into()),
            PublicKey::Secp256k1(value) => TendermintPublicKey::Secp256k1(value.into()),
        }
    }
}
