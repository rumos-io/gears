use ibc_proto::cosmos::staking::v1beta1::{
    query_server::{Query as StakingQuery, QueryServer as StakingQueryServer},
    QueryParamsRequest as StakingQueryParamsRequest,
    QueryParamsResponse as StakingQueryParamsResponse, QueryValidatorsRequest,
};
use ibc_proto::cosmos::staking::v1beta1::{
    Params, QueryDelegationRequest, QueryDelegationResponse, QueryDelegatorDelegationsRequest,
    QueryDelegatorDelegationsResponse, QueryDelegatorUnbondingDelegationsRequest,
    QueryDelegatorUnbondingDelegationsResponse, QueryDelegatorValidatorRequest,
    QueryDelegatorValidatorResponse, QueryDelegatorValidatorsRequest,
    QueryDelegatorValidatorsResponse, QueryHistoricalInfoRequest, QueryHistoricalInfoResponse,
    QueryPoolRequest, QueryPoolResponse, QueryRedelegationsRequest, QueryRedelegationsResponse,
    QueryUnbondingDelegationRequest, QueryUnbondingDelegationResponse,
    QueryValidatorDelegationsRequest, QueryValidatorDelegationsResponse, QueryValidatorRequest,
    QueryValidatorResponse, QueryValidatorUnbondingDelegationsRequest,
    QueryValidatorUnbondingDelegationsResponse, QueryValidatorsResponse,
};

use ibc_proto::google::protobuf::Duration;

use tonic::{transport::Server, Request, Response, Status};

use crate::runtime::runtime;

mod auth;

#[derive(Debug, Default)]
pub struct StakingService;

#[tonic::async_trait]
impl StakingQuery for StakingService {
    async fn validators(
        &self,
        _request: Request<QueryValidatorsRequest>,
    ) -> Result<Response<QueryValidatorsResponse>, Status> {
        unimplemented!()
    }

    async fn validator(
        &self,
        _request: Request<QueryValidatorRequest>,
    ) -> Result<Response<QueryValidatorResponse>, Status> {
        unimplemented!()
    }

    async fn validator_delegations(
        &self,
        _request: Request<QueryValidatorDelegationsRequest>,
    ) -> Result<Response<QueryValidatorDelegationsResponse>, Status> {
        unimplemented!()
    }

    async fn validator_unbonding_delegations(
        &self,
        _request: Request<QueryValidatorUnbondingDelegationsRequest>,
    ) -> Result<Response<QueryValidatorUnbondingDelegationsResponse>, Status> {
        unimplemented!()
    }

    async fn delegation(
        &self,
        _request: Request<QueryDelegationRequest>,
    ) -> Result<Response<QueryDelegationResponse>, Status> {
        unimplemented!()
    }

    async fn unbonding_delegation(
        &self,
        _request: Request<QueryUnbondingDelegationRequest>,
    ) -> Result<Response<QueryUnbondingDelegationResponse>, Status> {
        unimplemented!()
    }

    async fn delegator_delegations(
        &self,
        _request: Request<QueryDelegatorDelegationsRequest>,
    ) -> Result<Response<QueryDelegatorDelegationsResponse>, Status> {
        unimplemented!()
    }

    async fn delegator_unbonding_delegations(
        &self,
        _request: Request<QueryDelegatorUnbondingDelegationsRequest>,
    ) -> Result<Response<QueryDelegatorUnbondingDelegationsResponse>, Status> {
        unimplemented!()
    }

    async fn redelegations(
        &self,
        _request: Request<QueryRedelegationsRequest>,
    ) -> Result<Response<QueryRedelegationsResponse>, Status> {
        unimplemented!()
    }

    async fn delegator_validators(
        &self,
        _request: Request<QueryDelegatorValidatorsRequest>,
    ) -> Result<Response<QueryDelegatorValidatorsResponse>, Status> {
        unimplemented!()
    }

    async fn delegator_validator(
        &self,
        _request: Request<QueryDelegatorValidatorRequest>,
    ) -> Result<Response<QueryDelegatorValidatorResponse>, Status> {
        unimplemented!()
    }

    async fn historical_info(
        &self,
        _request: Request<QueryHistoricalInfoRequest>,
    ) -> Result<Response<QueryHistoricalInfoResponse>, Status> {
        unimplemented!()
    }

    async fn pool(
        &self,
        _request: Request<QueryPoolRequest>,
    ) -> Result<Response<QueryPoolResponse>, Status> {
        unimplemented!()
    }

    async fn params(
        &self,
        _request: Request<StakingQueryParamsRequest>,
    ) -> Result<Response<StakingQueryParamsResponse>, Status> {
        let response = StakingQueryParamsResponse {
            params: Some(Params {
                unbonding_time: Some(Duration {
                    seconds: 1814400,
                    nanos: 0,
                }),
                max_validators: 12,
                max_entries: 100,
                historical_entries: 10,
                bond_denom: "uatom".to_string(),
                min_commission_rate: "0.1".to_string(),
            }),
        };
        Ok(Response::new(response))
    }
}

pub fn run_grpc_server() {
    std::thread::spawn(move || {
        let result = runtime().block_on(launch());
        if let Err(err) = result {
            panic!("Failed to run gRPC server with err: {}", err)
        }
    });
}

async fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let address = "127.0.0.1:8080"
        .parse()
        .expect("hard coded address is valid");
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(ibc_proto::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap(); //TODO: unwrap
    let staking_service = StakingService::default();

    let server = Server::builder()
        .add_service(reflection_service)
        .add_service(StakingQueryServer::new(staking_service));

    tracing::info!("gRPC server running at {}", address);
    server.serve(address).await?;
    Ok(())
}
