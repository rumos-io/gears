#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]

pub struct RequestInfo {
    #[prost(string, tag = "1")]
    pub version: String,
    #[prost(uint64, tag = "2")]
    pub block_version: u64,
    #[prost(uint64, tag = "3")]
    pub p2p_version: u64,
}

impl From<RequestInfo> for super::inner::RequestInfo {
    fn from(
        RequestInfo {
            version,
            block_version,
            p2p_version,
        }: RequestInfo,
    ) -> Self {
        Self {
            version,
            block_version,
            p2p_version,
        }
    }
}

impl From<super::inner::RequestInfo> for RequestInfo {
    fn from(
        super::inner::RequestInfo {
            version,
            block_version,
            p2p_version,
        }: super::inner::RequestInfo,
    ) -> Self {
        Self {
            version,
            block_version,
            p2p_version,
        }
    }
}
