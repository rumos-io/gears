use gears::baseapp::{NodeQueryHandler, QueryRequest, QueryResponse};
use ibc_proto::cosmos::auth::v1beta1::{
    query_server::{Query, QueryServer},
    AddressBytesToStringRequest, AddressBytesToStringResponse, AddressStringToBytesRequest,
    AddressStringToBytesResponse, Bech32PrefixRequest, Bech32PrefixResponse,
    QueryAccountAddressByIdRequest, QueryAccountAddressByIdResponse, QueryAccountRequest,
    QueryAccountResponse, QueryAccountsRequest, QueryAccountsResponse,
    QueryModuleAccountByNameRequest, QueryModuleAccountByNameResponse, QueryModuleAccountsRequest,
    QueryModuleAccountsResponse, QueryParamsRequest as AuthQueryParamsRequest,
    QueryParamsResponse as AuthQueryParamsResponse,
};
use std::marker::PhantomData;
use tonic::{Request, Response, Status};
use tracing::info;

use crate::{AuthNodeQueryRequest, AuthNodeQueryResponse};

#[derive(Debug, Default)]
pub struct AuthService<QH, QReq, QRes> {
    app: QH,
    _phantom: PhantomData<(QReq, QRes)>,
}

#[tonic::async_trait]
impl<
        QReq: Send + Sync + 'static,
        QRes: Send + Sync + 'static,
        QH: NodeQueryHandler<QReq, QRes>,
    > Query for AuthService<QH, QReq, QRes>
where
    QReq: QueryRequest + From<AuthNodeQueryRequest>,
    QRes: QueryResponse + TryInto<AuthNodeQueryResponse, Error = Status>,
{
    async fn accounts(
        &self,
        _request: Request<QueryAccountsRequest>,
    ) -> Result<Response<QueryAccountsResponse>, Status> {
        unimplemented!() //TODO: implement
    }

    async fn account(
        &self,
        request: Request<QueryAccountRequest>,
    ) -> Result<Response<QueryAccountResponse>, Status> {
        info!("Received a gRPC request auth::account");
        let req = AuthNodeQueryRequest::Account(request.into_inner().try_into()?);
        let response = self.app.typed_query(req)?;
        let response: AuthNodeQueryResponse = response.try_into()?;
        let AuthNodeQueryResponse::Account(response) = response;
        Ok(Response::new(response.into()))
    }

    async fn account_address_by_id(
        &self,
        _request: Request<QueryAccountAddressByIdRequest>,
    ) -> Result<Response<QueryAccountAddressByIdResponse>, Status> {
        unimplemented!() //TODO: implement
    }

    async fn params(
        &self,
        _request: Request<AuthQueryParamsRequest>,
    ) -> Result<Response<AuthQueryParamsResponse>, Status> {
        unimplemented!() //TODO: implement
    }

    async fn module_accounts(
        &self,
        _request: Request<QueryModuleAccountsRequest>,
    ) -> Result<Response<QueryModuleAccountsResponse>, Status> {
        unimplemented!() //TODO: implement
    }

    async fn module_account_by_name(
        &self,
        _request: Request<QueryModuleAccountByNameRequest>,
    ) -> Result<Response<QueryModuleAccountByNameResponse>, Status> {
        unimplemented!() //TODO: implement
    }

    async fn bech32_prefix(
        &self,
        _request: Request<Bech32PrefixRequest>,
    ) -> Result<Response<Bech32PrefixResponse>, Status> {
        unimplemented!() //TODO: implement
    }

    async fn address_bytes_to_string(
        &self,
        _request: Request<AddressBytesToStringRequest>,
    ) -> Result<Response<AddressBytesToStringResponse>, Status> {
        unimplemented!() //TODO: implement
    }

    async fn address_string_to_bytes(
        &self,
        _request: Request<AddressStringToBytesRequest>,
    ) -> Result<Response<AddressStringToBytesResponse>, Status> {
        unimplemented!() //TODO: implement
    }
}

pub fn new<QH, QReq, QRes>(app: QH) -> QueryServer<AuthService<QH, QReq, QRes>>
where
    QReq: QueryRequest + Send + Sync + 'static + From<AuthNodeQueryRequest>,
    QRes: QueryResponse + Send + Sync + 'static + TryInto<AuthNodeQueryResponse, Error = Status>,
    QH: NodeQueryHandler<QReq, QRes>,
{
    let auth_service = AuthService {
        app,
        _phantom: Default::default(),
    };
    QueryServer::new(auth_service)
}
