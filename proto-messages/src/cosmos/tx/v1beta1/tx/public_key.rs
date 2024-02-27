use ibc_proto::{google::protobuf::Any, Protobuf};
use prost::bytes::Bytes;
use proto_types::AccAddress;
use serde::{Deserialize, Serialize};

use crate::{cosmos::crypto::secp256k1::v1beta1::PubKey as Secp256k1PubKey, error::Error};

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
    pub fn get_address(&self) -> AccAddress {
        match self {
            PublicKey::Secp256k1(key) => key.get_address(),
        }
    }
}

impl TryFrom<Any> for PublicKey {
    type Error = Error;

    fn try_from(any: Any) -> Result<Self, Self::Error> {
        match any.type_url.as_str() {
            "/cosmos.crypto.secp256k1.PubKey" => {
                let key = Secp256k1PubKey::decode::<Bytes>(any.value.into())
                    .map_err(|e| Error::DecodeGeneral(e.to_string()))?;
                Ok(PublicKey::Secp256k1(key))
            }
            _ => Err(Error::DecodeAny(format!(
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
        }
    }
}
