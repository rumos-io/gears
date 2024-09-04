use gears::{
    baseapp::{NodeQueryHandler, QueryRequest, QueryResponse},
    types::address::AccAddress,
};
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

const ERROR_STATE_MSG: &str = "An internal error occurred while querying the application state.";
const UNIMPLEMENTED_MSG: &str = "Unimplemented";

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
        request: Request<QueryAccountsRequest>,
    ) -> Result<Response<QueryAccountsResponse>, Status> {
        info!("Received a gRPC request auth::accounts");
        let req = AuthNodeQueryRequest::Accounts(request.into_inner().try_into()?);
        let response = self.app.typed_query(req)?;
        let response: AuthNodeQueryResponse = response.try_into()?;
        let AuthNodeQueryResponse::Accounts(response) = response else {
            return Err(Status::internal(ERROR_STATE_MSG));
        };
        Ok(Response::new(response.into()))
    }

    async fn account(
        &self,
        request: Request<QueryAccountRequest>,
    ) -> Result<Response<QueryAccountResponse>, Status> {
        info!("Received a gRPC request auth::account");
        let req = AuthNodeQueryRequest::Account(request.into_inner().try_into()?);
        let response = self.app.typed_query(req)?;
        let response: AuthNodeQueryResponse = response.try_into()?;
        let AuthNodeQueryResponse::Account(response) = response else {
            return Err(Status::internal(ERROR_STATE_MSG));
        };
        Ok(Response::new(response.into()))
    }

    async fn account_address_by_id(
        &self,
        _request: Request<QueryAccountAddressByIdRequest>,
    ) -> Result<Response<QueryAccountAddressByIdResponse>, Status> {
        // We currently don't save anything to `accountNumber` prefix, so I omitted this impl
        Err(Status::internal(UNIMPLEMENTED_MSG))
    }

    async fn params(
        &self,
        request: Request<AuthQueryParamsRequest>,
    ) -> Result<Response<AuthQueryParamsResponse>, Status> {
        info!("Received a gRPC request auth::params");
        let req = AuthNodeQueryRequest::Params(request.into_inner().try_into()?);
        let response = self.app.typed_query(req)?;
        let response: AuthNodeQueryResponse = response.try_into()?;
        let AuthNodeQueryResponse::Params(response) = response else {
            return Err(Status::internal(ERROR_STATE_MSG));
        };
        Ok(Response::new(response.into()))
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
        Ok(Response::new(Bech32PrefixResponse {
            bech32_prefix: std::env!("BECH_32_MAIN_PREFIX").to_owned(),
        }))
    }

    async fn address_bytes_to_string(
        &self,
        request: Request<AddressBytesToStringRequest>,
    ) -> Result<Response<AddressBytesToStringResponse>, Status> {
        let AddressBytesToStringRequest { address_bytes } = request.into_inner();

        let address_string = match AccAddress::try_from(address_bytes) {
            Ok(addr) => addr.to_string(),
            Err(e) => Err(Status::invalid_argument(format!("Invalid address: {e}")))?,
        };

        Ok(Response::new(AddressBytesToStringResponse {
            address_string,
        }))
    }

    async fn address_string_to_bytes(
        &self,
        request: Request<AddressStringToBytesRequest>,
    ) -> Result<Response<AddressStringToBytesResponse>, Status> {
        let AddressStringToBytesRequest { address_string } = request.into_inner();

        let address_bytes: Vec<u8> = match AccAddress::from_bech32(&address_string) {
            Ok(addr) => addr.into(),
            Err(e) => Err(Status::invalid_argument(format!("Invalid address: {e}")))?,
        };

        Ok(Response::new(AddressStringToBytesResponse {
            address_bytes,
        }))
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
