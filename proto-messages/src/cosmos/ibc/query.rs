use ibc::core::client::types::proto::v1::QueryClientParamsResponse as RawQueryClientParamsResponse;
pub use ibc_proto::cosmos::base::query::v1beta1::PageResponse;
use ibc_proto::Protobuf;
use serde::{Deserialize, Serialize};

use super::types::core::client::types::Params;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueryClientParamsResponse {
    pub params: Params,
}

impl Protobuf<RawQueryClientParamsResponse> for QueryClientParamsResponse {}

impl TryFrom<RawQueryClientParamsResponse> for QueryClientParamsResponse {
    type Error = std::convert::Infallible;

    fn try_from(value: RawQueryClientParamsResponse) -> Result<Self, Self::Error> {
        let params = value.params.unwrap_or_default().try_into()?;

        Ok(Self { params })
    }
}

impl From<QueryClientParamsResponse> for RawQueryClientParamsResponse {
    fn from(value: QueryClientParamsResponse) -> Self {
        let QueryClientParamsResponse { params } = value;

        Self {
            params: Some(params.into()),
        }
    }
}
