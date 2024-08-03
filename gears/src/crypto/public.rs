use address::AccAddress;
use bytes::Bytes;
use core_types::any::google::Any;
use core_types::Protobuf;
use ripemd::Ripemd160;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{keys::SIZE_ERR_MSG, secp256k1::Secp256k1PubKey};

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
    Ed25519(Vec<u8>),
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
            PublicKey::Ed25519(_) => todo!(), //TODO: implement
        }
    }

    pub fn get_address(&self) -> AccAddress {
        match self {
            PublicKey::Secp256k1(key) => key.get_address(),
            PublicKey::Ed25519(key) => get_address(key),
        }
    }
}

pub fn get_address(key_bytes: impl AsRef<[u8]>) -> AccAddress {
    let mut hasher = Sha256::new();
    hasher.update(key_bytes);
    let hash = hasher.finalize();

    let mut hasher = Ripemd160::new();
    hasher.update(hash);
    let hash = hasher.finalize();

    let res: AccAddress = hash.as_slice().try_into().expect(SIZE_ERR_MSG);

    res
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
            "/cosmos.crypto.ed25519.PubKey" => Ok(Self::Ed25519(any.value)),

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
            PublicKey::Ed25519(value) => Any {
                type_url: "/cosmos.crypto.ed25519.PubKey".to_string(),
                value,
            },
        }
    }
}
