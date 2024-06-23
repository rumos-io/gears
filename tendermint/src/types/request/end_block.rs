#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct RequestEndBlock {
    #[prost(int64, tag = "1")]
    pub height: i64, // TODO: make u32
}

impl From<super::inner::RequestEndBlock> for RequestEndBlock {
    fn from(super::inner::RequestEndBlock { height }: super::inner::RequestEndBlock) -> Self {
        Self { height }
    }
}

impl From<RequestEndBlock> for super::inner::RequestEndBlock {
    fn from(RequestEndBlock { height }: RequestEndBlock) -> Self {
        Self { height }
    }
}
