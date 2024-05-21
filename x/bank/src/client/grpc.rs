use std::{error::Error, marker::PhantomData};

use axum::http::response;
use gears::baseapp::{NodeQueryHandler, QueryRequest, QueryResponse};
use ibc_proto::cosmos::{
    bank::v1beta1::{
        query_server::{Query, QueryServer},
        QueryAllBalancesRequest, QueryAllBalancesResponse,
        QueryBalanceRequest as RawQueryBalanceRequest,
        QueryBalanceResponse as RawQueryBalanceResponse, QueryDenomMetadataRequest,
        QueryDenomMetadataResponse, QueryDenomOwnersRequest, QueryDenomOwnersResponse,
        QueryDenomsMetadataRequest, QueryDenomsMetadataResponse, QueryParamsRequest,
        QueryParamsResponse, QuerySpendableBalancesRequest, QuerySpendableBalancesResponse,
        QuerySupplyOfRequest, QuerySupplyOfResponse, QueryTotalSupplyRequest,
        QueryTotalSupplyResponse,
    },
    staking::v1beta1::QueryValidatorsRequest,
};

use prost::Message;
use tonic::{transport::Server, Request, Response, Status};

use crate::{
    types::query::{QueryBalanceRequest, QueryBalanceResponse},
    BankNodeQueryRequest, BankNodeQueryResponse,
};

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
    // QReq: TryFrom<QueryAccountRequest>,
    // QRes: Into<QueryAccountResponse>,
    QReq: QueryRequest + From<BankNodeQueryRequest>,
    QRes: QueryResponse + TryInto<BankNodeQueryResponse>,
    <QRes as TryInto<BankNodeQueryResponse>>::Error: Error, // TODO: remove this once unwrap is removed
{
    async fn balance(
        &self,
        request: Request<RawQueryBalanceRequest>,
    ) -> Result<Response<RawQueryBalanceResponse>, Status> {
        let req = BankNodeQueryRequest::Balance(request.into_inner().try_into().unwrap()); //TODO: unwrap

        let response = self.app.typed_query(req).unwrap(); //TODO: unwrap

        let response: BankNodeQueryResponse = response.try_into().unwrap(); //TODO: unwrap

        match response {
            BankNodeQueryResponse::Balance(response) => {
                let response = response.into();
                Ok(Response::new(response))
            }
            _ => unimplemented!(), //TODO: return an error
        }
    }

    async fn all_balances(
        &self,
        request: Request<QueryAllBalancesRequest>,
    ) -> Result<Response<QueryAllBalancesResponse>, Status> {
        unimplemented!()
    }

    async fn spendable_balances(
        &self,
        request: Request<QuerySpendableBalancesRequest>,
    ) -> Result<Response<QuerySpendableBalancesResponse>, Status> {
        unimplemented!()
    }

    async fn total_supply(
        &self,
        request: Request<QueryTotalSupplyRequest>,
    ) -> Result<Response<QueryTotalSupplyResponse>, Status> {
        unimplemented!()
    }

    async fn supply_of(
        &self,
        request: Request<QuerySupplyOfRequest>,
    ) -> Result<Response<QuerySupplyOfResponse>, Status> {
        unimplemented!()
    }

    async fn params(
        &self,
        request: Request<QueryParamsRequest>,
    ) -> Result<Response<QueryParamsResponse>, Status> {
        unimplemented!()
    }

    async fn denom_metadata(
        &self,
        request: Request<QueryDenomMetadataRequest>,
    ) -> Result<Response<QueryDenomMetadataResponse>, Status> {
        unimplemented!()
    }

    async fn denoms_metadata(
        &self,
        request: Request<QueryDenomsMetadataRequest>,
    ) -> Result<Response<QueryDenomsMetadataResponse>, Status> {
        unimplemented!()
    }

    async fn denom_owners(
        &self,
        request: Request<QueryDenomOwnersRequest>,
    ) -> Result<Response<QueryDenomOwnersResponse>, Status> {
        unimplemented!()
    }
}

pub fn new<QH, QReq, QRes>(app: QH) -> QueryServer<BankService<QH, QReq, QRes>>
where
    QReq: QueryRequest + Send + Sync + 'static + From<BankNodeQueryRequest>,
    QRes: QueryResponse + Send + Sync + 'static + TryInto<BankNodeQueryResponse>,
    QH: NodeQueryHandler<QReq, QRes>,
    <QRes as TryInto<BankNodeQueryResponse>>::Error: Error,
{
    let bank_service = BankService {
        app,
        _phantom: Default::default(),
    };
    QueryServer::new(bank_service)
}
