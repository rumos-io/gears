use crate::query::{GovQuery, GovQueryResponse};
use gears::baseapp::{NodeQueryHandler, QueryRequest, QueryResponse};
use ibc_proto::cosmos::gov::v1beta1::{
    query_server::{Query, QueryServer},
    QueryDepositRequest, QueryDepositResponse, QueryDepositsRequest, QueryDepositsResponse,
    QueryParamsRequest, QueryParamsResponse, QueryProposalRequest, QueryProposalResponse,
    QueryProposalsRequest, QueryProposalsResponse, QueryTallyResultRequest,
    QueryTallyResultResponse, QueryVoteRequest, QueryVoteResponse, QueryVotesRequest,
    QueryVotesResponse,
};
use std::marker::PhantomData;
use tonic::{Request, Response, Status};
use tracing::info;

const ERROR_STATE_MSG: &str = "An internal error occurred while querying the application state.";

#[derive(Debug, Default)]
pub struct GovService<QH, QReq, QRes> {
    app: QH,
    _phantom: PhantomData<(QReq, QRes)>,
}

#[tonic::async_trait]
impl<
        QReq: Send + Sync + 'static,
        QRes: Send + Sync + 'static,
        QH: NodeQueryHandler<QReq, QRes>,
    > Query for GovService<QH, QReq, QRes>
where
    QReq: QueryRequest + From<GovQuery>,
    QRes: QueryResponse + TryInto<GovQueryResponse, Error = Status>,
{
    async fn proposal(
        &self,
        request: Request<QueryProposalRequest>,
    ) -> Result<Response<QueryProposalResponse>, Status> {
        info!("Received a gRPC request gov::proposal");
        let req = GovQuery::Proposal(request.into_inner().try_into()?);
        let response: GovQueryResponse = self.app.typed_query(req)?.try_into()?;

        if let GovQueryResponse::Proposal(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }

    async fn proposals(
        &self,
        request: Request<QueryProposalsRequest>,
    ) -> Result<Response<QueryProposalsResponse>, Status> {
        info!("Received a gRPC request gov::proposals");
        let req = GovQuery::Proposals(request.into_inner().try_into()?);
        let response: GovQueryResponse = self.app.typed_query(req)?.try_into()?;

        if let GovQueryResponse::Proposals(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }

    async fn vote(
        &self,
        request: Request<QueryVoteRequest>,
    ) -> Result<Response<QueryVoteResponse>, Status> {
        info!("Received a gRPC request gov::vote");
        let req = GovQuery::Vote(request.into_inner().try_into()?);
        let response: GovQueryResponse = self.app.typed_query(req)?.try_into()?;

        if let GovQueryResponse::Vote(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }

    async fn votes(
        &self,
        request: Request<QueryVotesRequest>,
    ) -> Result<Response<QueryVotesResponse>, Status> {
        info!("Received a gRPC request gov::votes");
        let req = GovQuery::Votes(request.into_inner().try_into()?);
        let response: GovQueryResponse = self.app.typed_query(req)?.try_into()?;

        if let GovQueryResponse::Votes(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }

    async fn params(
        &self,
        request: Request<QueryParamsRequest>,
    ) -> Result<Response<QueryParamsResponse>, Status> {
        info!("Received a gRPC request gov::params");
        let req = GovQuery::Params(request.into_inner().try_into()?);
        let response: GovQueryResponse = self.app.typed_query(req)?.try_into()?;

        if let GovQueryResponse::Params(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }

    async fn deposit(
        &self,
        request: Request<QueryDepositRequest>,
    ) -> Result<Response<QueryDepositResponse>, Status> {
        info!("Received a gRPC request gov::deposit");
        let req = GovQuery::Deposit(request.into_inner().try_into()?);
        let response: GovQueryResponse = self.app.typed_query(req)?.try_into()?;

        if let GovQueryResponse::Deposit(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }

    async fn deposits(
        &self,
        request: Request<QueryDepositsRequest>,
    ) -> Result<Response<QueryDepositsResponse>, Status> {
        info!("Received a gRPC request gov::deposits");
        let req = GovQuery::Deposits(request.into_inner().try_into()?);
        let response: GovQueryResponse = self.app.typed_query(req)?.try_into()?;

        if let GovQueryResponse::Deposits(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }

    async fn tally_result(
        &self,
        request: Request<QueryTallyResultRequest>,
    ) -> Result<Response<QueryTallyResultResponse>, Status> {
        info!("Received a gRPC request gov::tally_result");
        let req = GovQuery::Tally(request.into_inner().try_into()?);
        let response: GovQueryResponse = self.app.typed_query(req)?.try_into()?;

        if let GovQueryResponse::Tally(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }
}

pub fn new<QH, QReq, QRes>(app: QH) -> QueryServer<GovService<QH, QReq, QRes>>
where
    QReq: QueryRequest + Send + Sync + 'static + From<GovQuery>,
    QRes: QueryResponse + Send + Sync + 'static + TryInto<GovQueryResponse, Error = Status>,
    QH: NodeQueryHandler<QReq, QRes>,
{
    let gov_service = GovService {
        app,
        _phantom: Default::default(),
    };
    QueryServer::new(gov_service)
}
