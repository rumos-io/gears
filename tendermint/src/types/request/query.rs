#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct RequestQuery {
    #[prost(bytes = "bytes", tag = "1")]
    pub data: ::prost::bytes::Bytes,
    #[prost(string, tag = "2")]
    pub path: String,
    #[prost(int64, tag = "3")]
    pub height: i64,
    #[prost(bool, tag = "4")]
    pub prove: bool,
}

impl From<super::inner::RequestQuery> for RequestQuery {
    fn from(
        super::inner::RequestQuery {
            data,
            path,
            height,
            prove,
        }: super::inner::RequestQuery,
    ) -> Self {
        Self {
            data,
            path,
            height,
            prove,
        }
    }
}
impl From<RequestQuery> for super::inner::RequestQuery {
    fn from(
        RequestQuery {
            data,
            path,
            height,
            prove,
        }: RequestQuery,
    ) -> Self {
        Self {
            data,
            path,
            height,
            prove,
        }
    }
}
