// use core::panic;
// use std::marker::PhantomData;

// use gears::baseapp::NodeQueryHandler;
// use ibc_proto::cosmos::base::node::v1beta1::service_server::Service;
// use ibc_proto::cosmos::staking::v1beta1::{
//     Params, QueryDelegationRequest, QueryDelegationResponse, QueryDelegatorDelegationsRequest,
//     QueryDelegatorDelegationsResponse, QueryDelegatorUnbondingDelegationsRequest,
//     QueryDelegatorUnbondingDelegationsResponse, QueryDelegatorValidatorRequest,
//     QueryDelegatorValidatorResponse, QueryDelegatorValidatorsRequest,
//     QueryDelegatorValidatorsResponse, QueryHistoricalInfoRequest, QueryHistoricalInfoResponse,
//     QueryPoolRequest, QueryPoolResponse, QueryRedelegationsRequest, QueryRedelegationsResponse,
//     QueryUnbondingDelegationRequest, QueryUnbondingDelegationResponse,
//     QueryValidatorDelegationsRequest, QueryValidatorDelegationsResponse, QueryValidatorRequest,
//     QueryValidatorResponse, QueryValidatorUnbondingDelegationsRequest,
//     QueryValidatorUnbondingDelegationsResponse, QueryValidatorsResponse,
// };
// use ibc_proto::cosmos::{
//     auth::v1beta1::BaseAccount,
//     staking::v1beta1::{
//         query_server::{Query as StakingQuery, QueryServer as StakingQueryServer},
//         QueryParamsRequest as StakingQueryParamsRequest,
//         QueryParamsResponse as StakingQueryParamsResponse,
//     },
// };

// use ibc_proto::google::protobuf::Duration;
// use ibc_proto::{
//     cosmos::auth::v1beta1::{
//         query_server::{Query as AuthQuery, QueryServer as AuthQueryServer},
//         AddressBytesToStringRequest, AddressBytesToStringResponse, AddressStringToBytesRequest,
//         AddressStringToBytesResponse, Bech32PrefixRequest, Bech32PrefixResponse,
//         QueryAccountAddressByIdRequest, QueryAccountAddressByIdResponse, QueryAccountRequest,
//         QueryAccountResponse, QueryAccountsRequest, QueryAccountsResponse,
//         QueryModuleAccountByNameRequest, QueryModuleAccountByNameResponse,
//         QueryModuleAccountsRequest, QueryModuleAccountsResponse,
//         QueryParamsRequest as AuthQueryParamsRequest,
//         QueryParamsResponse as AuthQueryParamsResponse,
//     },
//     google::protobuf::Any,
// };
// use prost::Message;
// //use tendermint::types::request::query::RequestQuery;
// //use tendermint::types::response::query::ResponseQuery;
// use tonic::{transport::Server, Request, Response, Status};

// //use crate::baseapp::QueryHandler;

// // pub trait TypedQueryHandler: Send + Sync + 'static {
// //     fn query(&self, request: RequestQuery) -> ResponseQuery;
// // }

// pub trait ServiceBuilder<QReq, QRes, QH: NodeQueryHandler<QReq, QRes>> {
//     fn build(query_handler: QH) -> Self;
// }

// #[derive(Debug, Default)]
// pub struct AuthService<QH, QReq, QRes> {
//     app: QH,
//     _phantom: PhantomData<(QReq, QRes)>,
// }

// impl<QReq, QRes, QH: NodeQueryHandler<QReq, QRes>> ServiceBuilder<QReq, QRes, QH>
//     for AuthService<QH, QReq, QRes>
// {
//     fn build(app: QH) -> Self {
//         Self {
//             app,
//             _phantom: Default::default(),
//         }
//     }
// }

// #[tonic::async_trait]
// impl<
//         QReq: Send + Sync + 'static,
//         QRes: Send + Sync + 'static,
//         QH: NodeQueryHandler<QReq, QRes>,
//     > AuthQuery for AuthService<QH, QReq, QRes>
// where
//     QReq: TryFrom<QueryAccountRequest>,
//     QRes: Into<QueryAccountResponse>,
// {
//     async fn accounts(
//         &self,
//         request: Request<QueryAccountsRequest>,
//     ) -> Result<Response<QueryAccountsResponse>, Status> {
//         unimplemented!()
//     }

//     async fn account(
//         &self,
//         request: Request<QueryAccountRequest>,
//     ) -> Result<Response<QueryAccountResponse>, Status> {
//         let request = request.into_inner();

//         // let request: RequestQuery = RequestQuery {
//         //     data: request.encode_to_vec().into(),
//         //     path: "/cosmos.auth.v1beta1.Query/Account".into(),
//         //     height: 0,
//         //     prove: false,
//         // };

//         let req: QReq = request.try_into().unwrap_or_else(|_| panic!("TODO")); // TODO: unwrap

//         let response = self.app.typed_query(req).unwrap();

//         // let response = QueryAccountResponse::decode(response.value)
//         //     .expect("should be a valid QueryBalanceResponse");

//         // let account = BaseAccount {
//         //     address: "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".to_string(),
//         //     pub_key: None,
//         //     account_number: 0,
//         //     sequence: 0,
//         // };

//         // let response = QueryAccountResponse {
//         //     account: Some(Any {
//         //         type_url: "/cosmos.auth.v1beta1.BaseAccount".to_string(),
//         //         value: account.encode_to_vec(),
//         //     }),
//         // };
//         Ok(Response::new(response.into()))
//     }

//     async fn account_address_by_id(
//         &self,
//         request: Request<QueryAccountAddressByIdRequest>,
//     ) -> Result<Response<QueryAccountAddressByIdResponse>, Status> {
//         unimplemented!()
//     }

//     async fn params(
//         &self,
//         request: Request<AuthQueryParamsRequest>,
//     ) -> Result<Response<AuthQueryParamsResponse>, Status> {
//         unimplemented!()
//     }

//     async fn module_accounts(
//         &self,
//         request: Request<QueryModuleAccountsRequest>,
//     ) -> Result<Response<QueryModuleAccountsResponse>, Status> {
//         unimplemented!()
//     }

//     async fn module_account_by_name(
//         &self,
//         request: Request<QueryModuleAccountByNameRequest>,
//     ) -> Result<Response<QueryModuleAccountByNameResponse>, Status> {
//         unimplemented!()
//     }

//     async fn bech32_prefix(
//         &self,
//         request: Request<Bech32PrefixRequest>,
//     ) -> Result<Response<Bech32PrefixResponse>, Status> {
//         unimplemented!()
//     }

//     async fn address_bytes_to_string(
//         &self,
//         request: Request<AddressBytesToStringRequest>,
//     ) -> Result<Response<AddressBytesToStringResponse>, Status> {
//         unimplemented!()
//     }

//     async fn address_string_to_bytes(
//         &self,
//         request: Request<AddressStringToBytesRequest>,
//     ) -> Result<Response<AddressStringToBytesResponse>, Status> {
//         unimplemented!()
//     }

//     // async fn account_info(
//     //     &self,
//     //     request: Request<QueryAccountInfoRequest>,
//     // ) -> Result<Response<QueryAccountInfoResponse>, Status> {
//     //     unimplemented!()
//     // }
// }
