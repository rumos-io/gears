use gears::baseapp::{QueryRequest, QueryResponse};
use serde::Serialize;

#[derive(Debug, Clone)]
pub enum GovQueryRequest {}

impl QueryRequest for GovQueryRequest {
    fn height(&self) -> u32 {
        todo!()
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum GovQueryResponse {}

impl QueryResponse for GovQueryResponse {}
