#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct ResponseInfo {
    #[prost(string, tag = "1")]
    #[serde(default)]
    pub data: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    #[serde(default)]
    pub version: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    #[serde(with = "crate::types::serializers::from_str", default)]
    pub app_version: u64,
    #[prost(int64, tag = "4")]
    #[serde(with = "crate::types::serializers::from_str", default)]
    pub last_block_height: i64,
    #[prost(bytes = "bytes", tag = "5")]
    #[serde(default)]
    #[serde(skip_serializing_if = "::bytes::Bytes::is_empty")]
    pub last_block_app_hash: ::prost::bytes::Bytes,
}

impl From<ResponseInfo> for super::inner::ResponseInfo {
    fn from(
        ResponseInfo {
            data,
            version,
            app_version,
            last_block_height,
            last_block_app_hash,
        }: ResponseInfo,
    ) -> Self {
        Self {
            data,
            version,
            app_version,
            last_block_height,
            last_block_app_hash,
        }
    }
}

impl From<super::inner::ResponseInfo> for ResponseInfo {
    fn from(
        super::inner::ResponseInfo {
            data,
            version,
            app_version,
            last_block_height,
            last_block_app_hash,
        }: super::inner::ResponseInfo,
    ) -> Self {
        Self {
            data,
            version,
            app_version,
            last_block_height,
            last_block_app_hash,
        }
    }
}
