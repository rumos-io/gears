use crate::types::proto::event::Event;

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct ResponseCheckTx {
    #[prost(uint32, tag = "1")]
    pub code: u32,
    #[prost(bytes = "bytes", tag = "2")]
    pub data: ::prost::bytes::Bytes,
    /// nondeterministic
    #[prost(string, tag = "3")]
    pub log: String,
    /// nondeterministic
    #[prost(string, tag = "4")]
    pub info: String,
    #[prost(int64, tag = "5")]
    pub gas_wanted: i64,
    #[prost(int64, tag = "6")]
    pub gas_used: i64,
    #[prost(message, repeated, tag = "7")]
    pub events: Vec<Event>,
    #[prost(string, tag = "8")]
    pub codespace: String,
    #[prost(string, tag = "9")]
    pub sender: String,
    #[prost(int64, tag = "10")]
    pub priority: i64,
    /// mempool_error is set by Tendermint.
    /// ABCI applictions creating a ResponseCheckTX should not set mempool_error.
    #[prost(string, tag = "11")]
    pub mempool_error: String,
}

impl From<super::inner::ResponseCheckTx> for ResponseCheckTx {
    fn from(
        super::inner::ResponseCheckTx {
            code,
            data,
            log,
            info,
            gas_wanted,
            gas_used,
            events,
            codespace,
            sender,
            priority,
            mempool_error,
        }: super::inner::ResponseCheckTx,
    ) -> Self {
        Self {
            code,
            data,
            log,
            info,
            gas_wanted,
            gas_used,
            events: events.into_iter().map(Into::into).collect(),
            codespace,
            sender,
            priority,
            mempool_error,
        }
    }
}

impl From<ResponseCheckTx> for super::inner::ResponseCheckTx {
    fn from(
        ResponseCheckTx {
            code,
            data,
            log,
            info,
            gas_wanted,
            gas_used,
            events,
            codespace,
            sender,
            priority,
            mempool_error,
        }: ResponseCheckTx,
    ) -> Self {
        Self {
            code,
            data,
            log,
            info,
            gas_wanted,
            gas_used,
            events: events.into_iter().map(Into::into).collect(),
            codespace,
            sender,
            priority,
            mempool_error,
        }
    }
}
