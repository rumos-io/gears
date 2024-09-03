use gears::baseapp::{NodeQueryHandler, QueryRequest, QueryResponse};
use ibc_proto::cosmos::bank::v1beta1::{
    query_server::{Query, QueryServer},
    QueryAllBalancesRequest, QueryAllBalancesResponse,
    QueryBalanceRequest as RawQueryBalanceRequest, QueryBalanceResponse as RawQueryBalanceResponse,
    QueryDenomMetadataRequest, QueryDenomMetadataResponse, QueryDenomOwnersRequest,
    QueryDenomOwnersResponse, QueryDenomsMetadataRequest, QueryDenomsMetadataResponse,
    QueryParamsRequest, QueryParamsResponse, QuerySpendableBalancesRequest,
    QuerySpendableBalancesResponse, QuerySupplyOfRequest, QuerySupplyOfResponse,
    QueryTotalSupplyRequest, QueryTotalSupplyResponse,
};
use std::marker::PhantomData;
use tonic::{Request, Response, Status};
use tracing::info;

use crate::{BankNodeQueryRequest, BankNodeQueryResponse};

const ERROR_STATE_MSG: &str = "An internal error occurred while querying the application state.";

#[derive(Debug, Default)]
pub struct BankService<QH, QReq, QRes> {
    app: QH,
    _phantom: PhantomData<(QReq, QRes)>,
}

#[tonic::async_trait]
impl<
        QReq: Send + Sync + 'static,
        QRes: Send + Sync + 'static,
        QH: NodeQueryHandler<QReq, QRes>,
    > Query for BankService<QH, QReq, QRes>
where
    QReq: QueryRequest + From<BankNodeQueryRequest>,
    QRes: QueryResponse + TryInto<BankNodeQueryResponse, Error = Status>,
{
    async fn balance(
        &self,
        request: Request<RawQueryBalanceRequest>,
    ) -> Result<Response<RawQueryBalanceResponse>, Status> {
        info!("Received a gRPC request bank::balance");
        let req = BankNodeQueryRequest::Balance(request.into_inner().try_into()?);
        let response = self.app.typed_query(req)?;
        let response: BankNodeQueryResponse = response.try_into()?;

        if let BankNodeQueryResponse::Balance(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }

    async fn all_balances(
        &self,
        request: Request<QueryAllBalancesRequest>,
    ) -> Result<Response<QueryAllBalancesResponse>, Status> {
        let req = BankNodeQueryRequest::AllBalances(request.into_inner().try_into()?);
        let response: BankNodeQueryResponse = self.app.typed_query(req)?.try_into()?;

        if let BankNodeQueryResponse::AllBalances(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }

    async fn spendable_balances(
        &self,
        _request: Request<QuerySpendableBalancesRequest>,
    ) -> Result<Response<QuerySpendableBalancesResponse>, Status> {
        unimplemented!() //TODO: implement
    }

    async fn total_supply(
        &self,
        request: Request<QueryTotalSupplyRequest>,
    ) -> Result<Response<QueryTotalSupplyResponse>, Status> {
        let req = BankNodeQueryRequest::TotalSupply(request.into_inner().try_into()?);
        let response: BankNodeQueryResponse = self.app.typed_query(req)?.try_into()?;

        if let BankNodeQueryResponse::TotalSupply(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }

    async fn supply_of(
        &self,
        _request: Request<QuerySupplyOfRequest>,
    ) -> Result<Response<QuerySupplyOfResponse>, Status> {
        unimplemented!() //TODO: implement
    }

    async fn params(
        &self,
        request: Request<QueryParamsRequest>,
    ) -> Result<Response<QueryParamsResponse>, Status> {
        let req = BankNodeQueryRequest::Params(request.into_inner().try_into()?);
        let response: BankNodeQueryResponse = self.app.typed_query(req)?.try_into()?;

        if let BankNodeQueryResponse::Params(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }

    async fn denom_metadata(
        &self,
        request: Request<QueryDenomMetadataRequest>,
    ) -> Result<Response<QueryDenomMetadataResponse>, Status> {
        let req = BankNodeQueryRequest::DenomMetadata(request.into_inner().try_into()?);
        let response: BankNodeQueryResponse = self.app.typed_query(req)?.try_into()?;

        if let BankNodeQueryResponse::DenomMetadata(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }

    async fn denoms_metadata(
        &self,
        request: Request<QueryDenomsMetadataRequest>,
    ) -> Result<Response<QueryDenomsMetadataResponse>, Status> {
        let req = BankNodeQueryRequest::DenomsMetadata(request.into_inner().try_into()?);
        let response: BankNodeQueryResponse = self.app.typed_query(req)?.try_into()?;

        if let BankNodeQueryResponse::DenomsMetadata(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }

    async fn denom_owners(
        &self,
        _request: Request<QueryDenomOwnersRequest>,
    ) -> Result<Response<QueryDenomOwnersResponse>, Status> {
        unimplemented!() //TODO: implement
    }
}

pub fn new<QH, QReq, QRes>(app: QH) -> QueryServer<BankService<QH, QReq, QRes>>
where
    QReq: QueryRequest + Send + Sync + 'static + From<BankNodeQueryRequest>,
    QRes: QueryResponse + Send + Sync + 'static + TryInto<BankNodeQueryResponse, Error = Status>,
    QH: NodeQueryHandler<QReq, QRes>,
{
    let bank_service = BankService {
        app,
        _phantom: Default::default(),
    };
    QueryServer::new(bank_service)
}
