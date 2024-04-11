#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct PublicKey {
    #[prost(oneof = "public_key::Sum", tags = "1, 2")]
    pub sum: Option<public_key::Sum>,
}

impl From<PublicKey> for inner::PublicKey {
    fn from(PublicKey { sum }: PublicKey) -> Self {
        Self {
            sum: sum.map(Into::into),
        }
    }
}

impl From<inner::PublicKey> for PublicKey {
    fn from(inner::PublicKey { sum }: inner::PublicKey) -> Self {
        Self {
            sum: sum.map(Into::into),
        }
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
    pub use tendermint_proto::crypto::ProofOp;
    pub use tendermint_proto::crypto::ProofOps;
    pub use tendermint_proto::crypto::PublicKey;
}

pub mod public_key {
    #[derive(Clone, PartialEq, Eq, ::prost::Oneof, serde::Serialize, serde::Deserialize)]
    #[serde(tag = "type", content = "value")]
    pub enum Sum {
        #[prost(bytes, tag = "1")]
        #[serde(
            rename = "tendermint/PubKeyEd25519",
            with = "crate::types::serializers::bytes::base64string"
        )]
        Ed25519(Vec<u8>),
        #[prost(bytes, tag = "2")]
        #[serde(
            rename = "tendermint/PubKeySecp256k1",
            with = "crate::types::serializers::bytes::base64string"
        )]
        Secp256k1(Vec<u8>),
    }

    impl From<Sum> for inner::Sum {
        fn from(value: Sum) -> Self {
            match value {
                Sum::Ed25519(sum) => Self::Ed25519(sum),
                Sum::Secp256k1(sum) => Self::Secp256k1(sum),
            }
        }
    }

    impl From<inner::Sum> for Sum {
        fn from(value: inner::Sum) -> Self {
            match value {
                inner::Sum::Ed25519(sum) => Self::Ed25519(sum),
                inner::Sum::Secp256k1(sum) => Self::Secp256k1(sum),
            }
        }
    }

    pub(crate) mod inner {
        pub use tendermint_proto::crypto::public_key::Sum;
    }
}
