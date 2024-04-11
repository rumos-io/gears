#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct RequestDeliverTx {
    #[prost(bytes = "bytes", tag = "1")]
    pub tx: ::prost::bytes::Bytes,
}

impl From<super::inner::RequestDeliverTx> for RequestDeliverTx {
    fn from(super::inner::RequestDeliverTx { tx }: super::inner::RequestDeliverTx) -> Self {
        Self { tx }
    }
}

impl From<RequestDeliverTx> for super::inner::RequestDeliverTx {
    fn from(RequestDeliverTx { tx }: RequestDeliverTx) -> Self {
        Self { tx }
    }
}
