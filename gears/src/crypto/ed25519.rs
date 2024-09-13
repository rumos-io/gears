use address::AccAddress;
use core_types::Protobuf;
use keyring::error::DecodeError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::public::SigningError;

//TODO: this module is not a full implementation

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Ed25519PubKey {
    #[serde(serialize_with = "serialize_key", deserialize_with = "deserialize_key")]
    key: inner::Ed25519PubKey,
}

impl Ed25519PubKey {
    pub fn verify_signature(
        &self,
        _message: impl AsRef<[u8]>,
        _signature: impl AsRef<[u8]>,
    ) -> Result<(), SigningError> {
        todo!()
    }

    pub fn get_address(&self) -> AccAddress {
        let key_bytes = Vec::from(self.to_owned());

        let mut hasher = Sha256::new();
        hasher.update(key_bytes);
        let hash = hasher.finalize();

        let address: AccAddress = hash[..20]
            .try_into()
            .expect("the slice is 20 bytes long which is less than AccAddress::MAX_ADDR_LEN");

        address
    }
}

impl TryFrom<Vec<u8>> for Ed25519PubKey {
    type Error = DecodeError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Ed25519PubKey {
            key: inner::Ed25519PubKey { key: value },
        })
    }
}

impl From<Ed25519PubKey> for Vec<u8> {
    fn from(key: Ed25519PubKey) -> Vec<u8> {
        key.key.key
    }
}

mod inner {
    // TODO: this isn't needed yet, but it probably will be once we have a proper implementation
    #[derive(Clone, PartialEq, ::prost::Message, ::serde::Serialize)]
    pub struct Ed25519PubKey {
        #[prost(bytes = "vec", tag = "1")]
        pub key: Vec<u8>,
    }
}

impl TryFrom<inner::Ed25519PubKey> for Ed25519PubKey {
    type Error = DecodeError;

    fn try_from(raw: inner::Ed25519PubKey) -> Result<Self, Self::Error> {
        Ok(Ed25519PubKey { key: raw })
    }
}

impl From<Ed25519PubKey> for inner::Ed25519PubKey {
    fn from(key: Ed25519PubKey) -> inner::Ed25519PubKey {
        inner::Ed25519PubKey { key: key.into() }
    }
}

impl Protobuf<inner::Ed25519PubKey> for Ed25519PubKey {}

// TODO: these will be needed once we have a proper implementation:
fn serialize_key<S>(key: &inner::Ed25519PubKey, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(&data_encoding::BASE64.encode(&key.key))
}

fn deserialize_key<'de, D>(deserializer: D) -> Result<inner::Ed25519PubKey, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_str(Ed25519Visitor)
}

struct Ed25519Visitor;

impl<'de> serde::de::Visitor<'de> for Ed25519Visitor {
    type Value = inner::Ed25519PubKey;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("string-encoded ed25519 public key")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let key = data_encoding::BASE64
            .decode(v.as_bytes())
            .map_err(|e| E::custom(format!("Error parsing public key '{}': {}", v, e)))?;

        Ok(inner::Ed25519PubKey { key })
    }
}
