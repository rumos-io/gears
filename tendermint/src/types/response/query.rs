use crate::types::proto::crypto::ProofOps;

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct ResponseQuery {
    #[prost(uint32, tag = "1")]
    pub code: u32,
    /// bytes data = 2; // use "value" instead.
    ///
    /// nondeterministic
    #[prost(string, tag = "3")]
    pub log: String,
    /// nondeterministic
    #[prost(string, tag = "4")]
    pub info: String,
    #[prost(int64, tag = "5")]
    pub index: i64,
    #[prost(bytes = "bytes", tag = "6")]
    pub key: ::prost::bytes::Bytes,
    #[prost(bytes = "bytes", tag = "7")]
    pub value: ::prost::bytes::Bytes,
    #[prost(message, optional, tag = "8")]
    pub proof_ops: Option<ProofOps>,
    #[prost(int64, tag = "9")]
    pub height: i64,
    #[prost(string, tag = "10")]
    pub codespace: String,
}

impl From<super::inner::ResponseQuery> for ResponseQuery {
    fn from(
        super::inner::ResponseQuery {
            code,
            log,
            info,
            index,
            key,
            value,
            proof_ops,
            height,
            codespace,
        }: super::inner::ResponseQuery,
    ) -> Self {
        Self {
            code,
            log,
            info,
            index,
            key,
            value,
            proof_ops: proof_ops.map(Into::into),
            height,
            codespace,
        }
    }
}

impl From<ResponseQuery> for super::inner::ResponseQuery {
    fn from(
        ResponseQuery {
            code,
            log,
            info,
            index,
            key,
            value,
            proof_ops,
            height,
            codespace,
        }: ResponseQuery,
    ) -> Self {
        Self {
            code,
            log,
            info,
            index,
            key,
            value,
            proof_ops: proof_ops.map(Into::into),
            height,
            codespace,
        }
    }
}
