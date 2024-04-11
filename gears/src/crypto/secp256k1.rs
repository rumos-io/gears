use bytes::Bytes;
use core_types::any::google::Any;
use keyring::{error::DecodeError, key::secp256k1::Secp256k1PubKey};
use prost::Message;
use tendermint::types::proto::Protobuf;

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RawSecp256k1PubKey {
    #[prost(bytes = "vec", tag = "1")]
    pub key: Vec<u8>,
}

impl TryFrom<Any> for RawSecp256k1PubKey {
    type Error = DecodeError;

    fn try_from(any: Any) -> Result<Self, Self::Error> {
        match any.type_url.as_str() {
            "/cosmos.crypto.secp256k1.PubKey" => {
                let key = RawSecp256k1PubKey::decode::<Bytes>(any.value.into())
                    .map_err(|e| DecodeError(e.to_string()))?;
                Ok(key)
            }
            _ => Err(DecodeError(format!(
                "Key type not recognized: {}",
                any.type_url
            ))),
        }
    }
}

impl From<RawSecp256k1PubKey> for Any {
    fn from(key: RawSecp256k1PubKey) -> Self {
        Any {
            type_url: "/cosmos.crypto.secp256k1.PubKey".to_string(),
            value: key.encode_to_vec(),
        }
    }
}

impl TryFrom<RawSecp256k1PubKey> for Secp256k1PubKey {
    type Error = DecodeError;

    fn try_from(raw: RawSecp256k1PubKey) -> Result<Self, Self::Error> {
        Secp256k1PubKey::try_from(raw.key)
    }
}

impl From<Secp256k1PubKey> for RawSecp256k1PubKey {
    fn from(key: Secp256k1PubKey) -> RawSecp256k1PubKey {
        RawSecp256k1PubKey {
            key: Vec::from(key),
        }
    }
}

impl Protobuf<RawSecp256k1PubKey> for Secp256k1PubKey {}
