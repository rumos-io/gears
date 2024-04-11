#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct RequestEcho {
    #[prost(string, tag = "1")]
    pub message: String,
}

impl From<super::inner::RequestEcho> for RequestEcho {
    fn from(super::inner::RequestEcho { message }: super::inner::RequestEcho) -> Self {
        Self { message }
    }
}

impl From<RequestEcho> for super::inner::RequestEcho {
    fn from(RequestEcho { message }: RequestEcho) -> Self {
        Self { message }
    }
}
