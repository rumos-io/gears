use crate::query::{
    request::{
        ParamsQuery, QueryDepositsRequest, QueryParamsRequest, QueryProposalRequest,
        QueryProposalsRequest, QueryTallyResultRequest, QueryVoteRequest, QueryVotesRequest,
    },
    GovQuery, GovQueryResponse,
};
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use gears::{
    baseapp::{NodeQueryHandler, QueryRequest, QueryResponse},
    rest::{error::HTTPError, Pagination, RestState},
    types::{address::AccAddress, pagination::request::PaginationRequest},
};

pub async fn proposals<
    QReq: QueryRequest + From<GovQuery>,
    QRes: QueryResponse + TryInto<GovQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    Query(pagination): Query<Pagination>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = GovQuery::Proposals(QueryProposalsRequest {
        voter: None,
        depositor: None,
        proposal_status: None,
        pagination: Some(PaginationRequest::from(pagination)),
    });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn proposals_proposal_id<
    QReq: QueryRequest + From<GovQuery>,
    QRes: QueryResponse + TryInto<GovQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    Path(proposal_id): Path<u64>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = GovQuery::Proposal(QueryProposalRequest { proposal_id });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn proposals_deposits<
    QReq: QueryRequest + From<GovQuery>,
    QRes: QueryResponse + TryInto<GovQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    Path(proposal_id): Path<u64>,
    Query(pagination): Query<Pagination>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = GovQuery::Deposits(QueryDepositsRequest {
        proposal_id,
        pagination: Some(PaginationRequest::from(pagination)),
    });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn proposals_tally<
    QReq: QueryRequest + From<GovQuery>,
    QRes: QueryResponse + TryInto<GovQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    Path(proposal_id): Path<u64>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = GovQuery::Tally(QueryTallyResultRequest { proposal_id });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn proposals_votes<
    QReq: QueryRequest + From<GovQuery>,
    QRes: QueryResponse + TryInto<GovQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    Path(proposal_id): Path<u64>,
    Query(pagination): Query<Pagination>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = GovQuery::Votes(QueryVotesRequest {
        proposal_id,
        pagination: Some(PaginationRequest::from(pagination)),
    });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn proposals_votes_voter<
    QReq: QueryRequest + From<GovQuery>,
    QRes: QueryResponse + TryInto<GovQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    Path((proposal_id, voter)): Path<(u64, AccAddress)>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = GovQuery::Vote(QueryVoteRequest { proposal_id, voter });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn params_voting<
    QReq: QueryRequest + From<GovQuery>,
    QRes: QueryResponse + TryInto<GovQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = GovQuery::Params(QueryParamsRequest {
        kind: ParamsQuery::Voting,
    });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn params_tally<
    QReq: QueryRequest + From<GovQuery>,
    QRes: QueryResponse + TryInto<GovQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = GovQuery::Params(QueryParamsRequest {
        kind: ParamsQuery::Tally,
    });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn params_deposit<
    QReq: QueryRequest + From<GovQuery>,
    QRes: QueryResponse + TryInto<GovQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = GovQuery::Params(QueryParamsRequest {
        kind: ParamsQuery::Deposit,
    });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub fn get_router<
    QReq: QueryRequest + From<GovQuery>,
    QRes: QueryResponse + TryInto<GovQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>() -> Router<RestState<QReq, QRes, App>> {
    Router::new()
        .route("/v1beta1/proposals", get(proposals))
        .route(
            "/v1beta1/proposals/:proposal_id",
            get(proposals_proposal_id),
        )
        .route(
            "/v1beta1/proposals/:proposal_id/deposits",
            get(proposals_deposits),
        )
        .route(
            "/v1beta1/proposals/:proposal_id/tally",
            get(proposals_tally),
        )
        .route(
            "/v1beta1/proposals/:proposal_id/votes",
            get(proposals_votes),
        )
        .route(
            "/v1beta1/proposals/:proposal_id/votes/:voter",
            get(proposals_votes_voter),
        )
        .route("/v1beta1/params/voting", get(params_voting))
        .route("/v1beta1/params/tallying", get(params_tally))
        .route("/v1beta1/params/deposit", get(params_deposit))
}
