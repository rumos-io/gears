#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct ResponseEcho {
    #[prost(string, tag = "1")]
    pub message: String,
}

impl From<super::inner::ResponseEcho> for ResponseEcho {
    fn from(super::inner::ResponseEcho { message }: super::inner::ResponseEcho) -> Self {
        Self { message }
    }
}

impl From<ResponseEcho> for super::inner::ResponseEcho {
    fn from(ResponseEcho { message }: ResponseEcho) -> Self {
        Self { message }
    }
}
