use gears::{
    baseapp::{Query, QueryRequest, QueryResponse},
    derive::{Protobuf, Query},
};
use serde::{Deserialize, Serialize};

use super::{plan::Plan, ModuleVersion};

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

mod inner {
    pub use ibc_proto::cosmos::upgrade::v1beta1::QueryAppliedPlanRequest;
    pub use ibc_proto::cosmos::upgrade::v1beta1::QueryAppliedPlanResponse;
    pub use ibc_proto::cosmos::upgrade::v1beta1::QueryCurrentPlanRequest;
    pub use ibc_proto::cosmos::upgrade::v1beta1::QueryCurrentPlanResponse;
    pub use ibc_proto::cosmos::upgrade::v1beta1::QueryModuleVersionsRequest;
    pub use ibc_proto::cosmos::upgrade::v1beta1::QueryModuleVersionsResponse;

    /*
       NOTE: these are deprecated
       pub use ibc_proto::cosmos::upgrade::v1beta1::QueryUpgradedConsensusStateRequest;
       pub use ibc_proto::cosmos::upgrade::v1beta1::QueryUpgradedConsensusStateResponse;
    */
}

#[derive(Debug, Clone, Query, Protobuf, Serialize, Deserialize)]
#[proto(raw = "inner::QueryCurrentPlanRequest")]
#[query(url = "/cosmos.upgrade.v1beta1.QueryCurrentPlanRequest")]
pub struct QueryCurrentPlanRequest {}

#[derive(Debug, Clone, Query, Protobuf, Serialize, Deserialize)]
#[proto(raw = "inner::QueryCurrentPlanResponse")]
pub struct QueryCurrentPlanResponse {
    #[proto(optional)]
    pub plan: Option<Plan>,
}

#[derive(Debug, Clone, Query, Protobuf, Serialize, Deserialize)]
#[proto(raw = "inner::QueryAppliedPlanRequest")]
#[query(url = "/cosmos.upgrade.v1beta1.QueryAppliedPlanRequest")]
pub struct QueryAppliedPlanRequest {
    pub name: String,
}

#[derive(Debug, Clone, Query, Protobuf, Serialize, Deserialize)]
#[proto(raw = "inner::QueryAppliedPlanResponse")]
pub struct QueryAppliedPlanResponse {
    pub height: u32,
}

#[derive(Debug, Clone, Query, Protobuf, Serialize, Deserialize)]
#[proto(raw = "inner::QueryModuleVersionsRequest")]
#[query(url = "/cosmos.upgrade.v1beta1.QueryModuleVersionsRequest")]
pub struct QueryModuleVersionsRequest {
    pub module_name: String,
}

#[derive(Debug, Clone, Query, Protobuf, Serialize, Deserialize)]
#[proto(raw = "inner::QueryModuleVersionsResponse")]
pub struct QueryModuleVersionsResponse {
    #[proto(repeated)]
    pub module_versions: Vec<ModuleVersion>,
}
