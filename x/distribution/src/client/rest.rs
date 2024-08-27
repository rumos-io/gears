use crate::{DistributionNodeQueryRequest, DistributionNodeQueryResponse};
use axum::{extract::State, routing::get, Json, Router};
use gears::{
    baseapp::{NodeQueryHandler, QueryRequest, QueryResponse},
    rest::{error::HTTPError, RestState},
};

pub async fn community_pool<
    QReq: QueryRequest + From<DistributionNodeQueryRequest>,
    QRes: QueryResponse + TryInto<DistributionNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = DistributionNodeQueryRequest::CommunityPool(crate::QueryCommunityPoolRequest {});
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub fn get_router<
    QReq: QueryRequest + From<DistributionNodeQueryRequest>,
    QRes: QueryResponse + TryInto<DistributionNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>() -> Router<RestState<QReq, QRes, App>> {
    Router::new().route("/v1beta1/community_pool", get(community_pool))
}
