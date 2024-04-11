#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct RequestCheckTx {
    #[prost(bytes = "bytes", tag = "1")]
    pub tx: ::prost::bytes::Bytes,
    #[prost(enumeration = "CheckTxType", tag = "2")]
    pub r#type: i32,
}

impl From<super::inner::RequestCheckTx> for RequestCheckTx {
    fn from(super::inner::RequestCheckTx { tx, r#type }: super::inner::RequestCheckTx) -> Self {
        Self { tx, r#type }
    }
}

impl From<RequestCheckTx> for super::inner::RequestCheckTx {
    fn from(RequestCheckTx { tx, r#type }: RequestCheckTx) -> Self {
        Self { tx, r#type }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CheckTxType {
    New = 0,
    Recheck = 1,
}

impl CheckTxType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            CheckTxType::New => "NEW",
            CheckTxType::Recheck => "RECHECK",
        }
    }
}

impl From<inner::CheckTxType> for CheckTxType {
    fn from(value: inner::CheckTxType) -> Self {
        match value {
            inner::CheckTxType::New => Self::New,
            inner::CheckTxType::Recheck => Self::Recheck,
        }
    }
}

impl From<CheckTxType> for inner::CheckTxType {
    fn from(value: CheckTxType) -> Self {
        match value {
            CheckTxType::New => Self::New,
            CheckTxType::Recheck => Self::Recheck,
        }
    }
}

pub mod inner {
    pub use tendermint_proto::abci::CheckTxType;
}
