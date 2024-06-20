use std::str::FromStr;

use crate::error::Error;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum PublicKey {
    #[serde(
        rename = "tendermint/PubKeyEd25519",
        with = "crate::types::serializers::bytes::base64string"
    )]
    Ed25519(Vec<u8>), //TODO: should we check that bytes contain a valid public key?
    #[serde(
        rename = "tendermint/PubKeySecp256k1",
        with = "crate::types::serializers::bytes::base64string"
    )]
    Secp256k1(Vec<u8>), //TODO: should we check that bytes contain a valid public key?
}

impl PublicKey {
    pub fn raw(&self) -> &[u8] {
        match self {
            PublicKey::Ed25519(value) => value,
            PublicKey::Secp256k1(value) => value,
        }
    }

    pub fn str_type(&self) -> &'static str {
        match self {
            PublicKey::Secp256k1(_) => "secp256k1",
            PublicKey::Ed25519(_) => "ed25519",
        }
    }
}

impl From<PublicKey> for inner::PublicKey {
    fn from(key: PublicKey) -> Self {
        match key {
            PublicKey::Ed25519(value) => inner::PublicKey {
                sum: Some(inner::Sum::Ed25519(value)),
            },
            PublicKey::Secp256k1(value) => inner::PublicKey {
                sum: Some(inner::Sum::Secp256k1(value)),
            },
        }
    }
}

impl TryFrom<inner::PublicKey> for PublicKey {
    type Error = Error;

    fn try_from(inner::PublicKey { sum }: inner::PublicKey) -> Result<Self, Self::Error> {
        let sum = sum.ok_or(Error::InvalidData("public key is empty".to_string()))?;
        match sum {
            inner::Sum::Ed25519(value) => Ok(PublicKey::Ed25519(value)),
            inner::Sum::Secp256k1(value) => Ok(PublicKey::Secp256k1(value)),
        }
    }
}

impl FromStr for PublicKey {
    type Err = serde_json::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct ProofOp {
    #[prost(string, tag = "1")]
    pub r#type: String,
    #[prost(bytes = "vec", tag = "2")]
    pub key: Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub data: Vec<u8>,
}

impl From<ProofOp> for inner::ProofOp {
    fn from(ProofOp { r#type, key, data }: ProofOp) -> Self {
        Self { r#type, key, data }
    }
}

impl From<inner::ProofOp> for ProofOp {
    fn from(inner::ProofOp { r#type, key, data }: inner::ProofOp) -> Self {
        Self { r#type, key, data }
    }
}

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct ProofOps {
    #[prost(message, repeated, tag = "1")]
    pub ops: Vec<ProofOp>,
}

impl From<ProofOps> for inner::ProofOps {
    fn from(ProofOps { ops }: ProofOps) -> Self {
        Self {
            ops: ops.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<inner::ProofOps> for ProofOps {
    fn from(inner::ProofOps { ops }: inner::ProofOps) -> Self {
        Self {
            ops: ops.into_iter().map(Into::into).collect(),
        }
    }
}

pub(crate) mod inner {
    pub use tendermint_proto::crypto::public_key::Sum;
    pub use tendermint_proto::crypto::ProofOp;
    pub use tendermint_proto::crypto::ProofOps;
    pub use tendermint_proto::crypto::PublicKey;
}
