use serde::{Deserialize, Serialize};

use super::secp256k1::Secp256k1PubKey;

// cosmos::crypto::secp256k1::v1beta1::PubKey as Secp256k1PubKey,

pub type SigningError = secp256k1::Error;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum PublicKey {
    #[serde(rename = "/cosmos.crypto.secp256k1.PubKey")]
    Secp256k1(Secp256k1PubKey),
    //Secp256r1(Vec<u8>),
    //Ed25519(Vec<u8>),
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
        }
    }
}

// #[derive(Debug, Clone, thiserror::Error)]
// pub enum PublicDecodeError {
//     #[error("Prost decode: {0}")]
//     Decode(#[from] prost::DecodeError),
//     #[error("Key type not recognized: {0}")]
//     NotRecognized(String),
// }

// impl TryFrom<Vec<u8>> for PublicKey {
//     type Error = PublicDecodeError;

//     fn try_from(any: Vec<u8>) -> Result<Self, Self::Error> {
//         match any.type_url.as_str() {
//             "/cosmos.crypto.secp256k1.PubKey" => {
//                 let key = Secp256k1PubKey::decode::<prost::bytes::Bytes>(any.value.into())?;
//                 Ok(PublicKey::Secp256k1(key))
//             }
//             _ => Err(PublicDecodeError::NotRecognized(any.type_url)),
//         }
//     }
// }

// impl From<PublicKey> for Any {
//     fn from(key: PublicKey) -> Self {
//         match key {
//             PublicKey::Secp256k1(key) => Any {
//                 type_url: "/cosmos.crypto.secp256k1.PubKey".to_string(),
//                 value: key.encode_vec().expect("TODO"), // TODO:NOW
//             },
//         }
//     }
// }
