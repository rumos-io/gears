use crate::types::proto::event::Event;

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct ResponseDeliverTx {
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
    /// nondeterministic
    #[prost(message, repeated, tag = "7")]
    pub events: Vec<Event>,
    #[prost(string, tag = "8")]
    pub codespace: String,
}

impl From<ResponseDeliverTx> for super::inner::ResponseDeliverTx {
    fn from(
        ResponseDeliverTx {
            code,
            data,
            log,
            info,
            gas_wanted,
            gas_used,
            events,
            codespace,
        }: ResponseDeliverTx,
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
        }
    }
}

impl From<super::inner::DeliverTx> for ResponseDeliverTx {
    fn from(value: super::inner::DeliverTx) -> Self {
        super::inner::ResponseDeliverTx::from(value).into()
    }
}

impl From<super::inner::ResponseDeliverTx> for ResponseDeliverTx {
    fn from(
        super::inner::ResponseDeliverTx {
            code,
            data,
            log,
            info,
            gas_wanted,
            gas_used,
            events,
            codespace,
        }: super::inner::ResponseDeliverTx,
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
        }
    }
}
