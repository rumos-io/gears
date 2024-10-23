use bytes::Bytes;

use crate::types::proto::crypto::ProofOps;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResponseQuery {
    pub code: u32,
    /// bytes data = 2; // use "value" instead.
    ///
    /// nondeterministic
    pub log: String,
    /// nondeterministic
    pub info: String,
    pub index: i64,
    pub key: Bytes,
    pub value: Bytes,
    pub proof_ops: Option<ProofOps>,
    pub height: u32,
    pub codespace: String,
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
            height: height.into(),
            codespace,
        }
    }
}
