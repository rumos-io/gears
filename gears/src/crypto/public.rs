use super::secp256k1::Secp256k1PubKey;
use crate::tendermint::types::proto::crypto::{public_key::Sum, PublicKey as TendermintPublicKey};
use serde::{Deserialize, Serialize};

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

impl From<Secp256k1PubKey> for PublicKey {
    fn from(value: Secp256k1PubKey) -> Self {
        Self::Secp256k1(value)
    }
}

impl From<PublicKey> for TendermintPublicKey {
    fn from(value: PublicKey) -> Self {
        match value {
            PublicKey::Secp256k1(key) => TendermintPublicKey {
                sum: Some(Sum::Secp256k1(key.into())),
            },
        }
    }
}
