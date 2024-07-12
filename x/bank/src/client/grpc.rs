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
            Err(Status::internal(
                "An internal error occurred while querying the application state.",
            ))
        }
    }

    async fn all_balances(
        &self,
        _request: Request<QueryAllBalancesRequest>,
    ) -> Result<Response<QueryAllBalancesResponse>, Status> {
        unimplemented!()
    }

    async fn spendable_balances(
        &self,
        _request: Request<QuerySpendableBalancesRequest>,
    ) -> Result<Response<QuerySpendableBalancesResponse>, Status> {
        unimplemented!()
    }

    async fn total_supply(
        &self,
        _request: Request<QueryTotalSupplyRequest>,
    ) -> Result<Response<QueryTotalSupplyResponse>, Status> {
        unimplemented!()
    }

    async fn supply_of(
        &self,
        _request: Request<QuerySupplyOfRequest>,
    ) -> Result<Response<QuerySupplyOfResponse>, Status> {
        unimplemented!()
    }

    async fn params(
        &self,
        _request: Request<QueryParamsRequest>,
    ) -> Result<Response<QueryParamsResponse>, Status> {
        unimplemented!()
    }

    async fn denom_metadata(
        &self,
        _request: Request<QueryDenomMetadataRequest>,
    ) -> Result<Response<QueryDenomMetadataResponse>, Status> {
        unimplemented!()
    }

    async fn denoms_metadata(
        &self,
        _request: Request<QueryDenomsMetadataRequest>,
    ) -> Result<Response<QueryDenomsMetadataResponse>, Status> {
        unimplemented!()
    }

    async fn denom_owners(
        &self,
        _request: Request<QueryDenomOwnersRequest>,
    ) -> Result<Response<QueryDenomOwnersResponse>, Status> {
        unimplemented!()
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
