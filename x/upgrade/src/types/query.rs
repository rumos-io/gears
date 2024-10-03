use gears::baseapp::{Query, QueryRequest, QueryResponse};

#[derive(Debug, Clone)]
pub enum UpgradeQueryRequest {}

impl QueryRequest for UpgradeQueryRequest {
    fn height(&self) -> u32 {
        todo!()
    }
}

impl Query for UpgradeQueryRequest {
    fn query_url(&self) -> &'static str {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum UpgradeQueryResponse {}

impl QueryResponse for UpgradeQueryResponse {
    fn into_bytes(self) -> Vec<u8> {
        todo!()
    }
}
