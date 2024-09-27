use gears::baseapp::{NodeQueryHandler, QueryRequest, QueryResponse};
use ibc_proto::cosmos::staking::v1beta1::{
    query_server::{Query, QueryServer},
    QueryDelegationRequest, QueryDelegationResponse, QueryDelegatorDelegationsRequest,
    QueryDelegatorDelegationsResponse, QueryDelegatorUnbondingDelegationsRequest,
    QueryDelegatorUnbondingDelegationsResponse, QueryDelegatorValidatorRequest,
    QueryDelegatorValidatorResponse, QueryDelegatorValidatorsRequest,
    QueryDelegatorValidatorsResponse, QueryHistoricalInfoRequest, QueryHistoricalInfoResponse,
    QueryParamsRequest, QueryParamsResponse, QueryPoolRequest, QueryPoolResponse,
    QueryRedelegationsRequest, QueryRedelegationsResponse, QueryUnbondingDelegationRequest,
    QueryUnbondingDelegationResponse, QueryValidatorDelegationsRequest,
    QueryValidatorDelegationsResponse, QueryValidatorRequest, QueryValidatorResponse,
    QueryValidatorUnbondingDelegationsRequest, QueryValidatorUnbondingDelegationsResponse,
    QueryValidatorsRequest, QueryValidatorsResponse,
};
use std::marker::PhantomData;
use tonic::{Request, Response, Status};
use tracing::info;

use crate::{StakingNodeQueryRequest, StakingNodeQueryResponse};

#[derive(Debug, Default)]
pub struct StakingService<QH, QReq, QRes> {
    app: QH,
    _phantom: PhantomData<(QReq, QRes)>,
}

#[tonic::async_trait]
impl<
        QReq: Send + Sync + 'static,
        QRes: Send + Sync + 'static,
        QH: NodeQueryHandler<QReq, QRes>,
    > Query for StakingService<QH, QReq, QRes>
where
    QReq: QueryRequest + From<StakingNodeQueryRequest>,
    QRes: QueryResponse + TryInto<StakingNodeQueryResponse, Error = Status>,
{
    async fn validators(
        &self,
        request: Request<QueryValidatorsRequest>,
    ) -> Result<Response<QueryValidatorsResponse>, Status> {
        info!("Received a gRPC request staking::validators");
        let req = StakingNodeQueryRequest::Validators(request.into_inner().try_into()?);
        let response = self.app.typed_query(req)?;
        let response: StakingNodeQueryResponse = response.try_into()?;

        if let StakingNodeQueryResponse::Validators(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(
                "An internal error occurred while querying the application state.",
            ))
        }
    }

    async fn validator(
        &self,
        request: Request<QueryValidatorRequest>,
    ) -> Result<Response<QueryValidatorResponse>, Status> {
        info!("Received a gRPC request staking::validator");
        let req = StakingNodeQueryRequest::Validator(request.into_inner().try_into()?);
        let response = self.app.typed_query(req)?;
        let response: StakingNodeQueryResponse = response.try_into()?;

        if let StakingNodeQueryResponse::Validator(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(
                "An internal error occurred while querying the application state.",
            ))
        }
    }

    async fn validator_delegations(
        &self,
        request: Request<QueryValidatorDelegationsRequest>,
    ) -> Result<Response<QueryValidatorDelegationsResponse>, Status> {
        info!("Received a gRPC request staking::validator_delegations");
        let req = StakingNodeQueryRequest::ValidatorDelegations(request.into_inner().try_into()?);
        let response = self.app.typed_query(req)?;
        let response: StakingNodeQueryResponse = response.try_into()?;

        if let StakingNodeQueryResponse::ValidatorDelegations(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(
                "An internal error occurred while querying the application state.",
            ))
        }
    }

    async fn validator_unbonding_delegations(
        &self,
        _request: Request<QueryValidatorUnbondingDelegationsRequest>,
    ) -> Result<Response<QueryValidatorUnbondingDelegationsResponse>, Status> {
        unimplemented!()
    }

    async fn delegation(
        &self,
        request: Request<QueryDelegationRequest>,
    ) -> Result<Response<QueryDelegationResponse>, Status> {
        info!("Received a gRPC request staking::delegation");
        let req = StakingNodeQueryRequest::Delegation(request.into_inner().try_into()?);
        let response = self.app.typed_query(req)?;
        let response: StakingNodeQueryResponse = response.try_into()?;

        if let StakingNodeQueryResponse::Delegation(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(
                "An internal error occurred while querying the application state.",
            ))
        }
    }

    async fn unbonding_delegation(
        &self,
        request: Request<QueryUnbondingDelegationRequest>,
    ) -> Result<Response<QueryUnbondingDelegationResponse>, Status> {
        info!("Received a gRPC request staking::unbonding_delegation");
        let req = StakingNodeQueryRequest::UnbondingDelegation(request.into_inner().try_into()?);
        let response = self.app.typed_query(req)?;
        let response: StakingNodeQueryResponse = response.try_into()?;

        if let StakingNodeQueryResponse::UnbondingDelegation(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(
                "An internal error occurred while querying the application state.",
            ))
        }
    }

    async fn delegator_delegations(
        &self,
        request: Request<QueryDelegatorDelegationsRequest>,
    ) -> Result<Response<QueryDelegatorDelegationsResponse>, Status> {
        info!("Received a gRPC request staking::delegator_delegations");
        let req = StakingNodeQueryRequest::Delegations(request.into_inner().try_into()?);
        let response = self.app.typed_query(req)?;
        let response: StakingNodeQueryResponse = response.try_into()?;

        if let StakingNodeQueryResponse::Delegations(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(
                "An internal error occurred while querying the application state.",
            ))
        }
    }

    async fn delegator_unbonding_delegations(
        &self,
        request: Request<QueryDelegatorUnbondingDelegationsRequest>,
    ) -> Result<Response<QueryDelegatorUnbondingDelegationsResponse>, Status> {
        info!("Received a gRPC request staking::delegator_unbonding_delegations");
        let req = StakingNodeQueryRequest::UnbondingDelegations(request.into_inner().try_into()?);
        let response = self.app.typed_query(req)?;
        let response: StakingNodeQueryResponse = response.try_into()?;

        if let StakingNodeQueryResponse::UnbondingDelegations(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(
                "An internal error occurred while querying the application state.",
            ))
        }
    }

    async fn redelegations(
        &self,
        request: Request<QueryRedelegationsRequest>,
    ) -> Result<Response<QueryRedelegationsResponse>, Status> {
        info!("Received a gRPC request staking::redelegations");
        let req = StakingNodeQueryRequest::Redelegations(request.into_inner().try_into().map_err(
            |_| {
                Status::internal("An internal error occurred while querying the application state.")
            },
        )?);
        let response = self.app.typed_query(req)?;
        let response: StakingNodeQueryResponse = response.try_into()?;

        if let StakingNodeQueryResponse::Redelegations(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(
                "An internal error occurred while querying the application state.",
            ))
        }
    }

    async fn delegator_validators(
        &self,
        request: Request<QueryDelegatorValidatorsRequest>,
    ) -> Result<Response<QueryDelegatorValidatorsResponse>, Status> {
        info!("Received a gRPC request staking::delegator_validators");
        let req = StakingNodeQueryRequest::DelegatorValidators(
            request.into_inner().try_into().map_err(|_| {
                Status::internal("An internal error occurred while querying the application state.")
            })?,
        );
        let response = self.app.typed_query(req)?;
        let response: StakingNodeQueryResponse = response.try_into()?;

        if let StakingNodeQueryResponse::DelegatorValidators(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(
                "An internal error occurred while querying the application state.",
            ))
        }
    }

    async fn delegator_validator(
        &self,
        _request: Request<QueryDelegatorValidatorRequest>,
    ) -> Result<Response<QueryDelegatorValidatorResponse>, Status> {
        unimplemented!()
    }

    async fn historical_info(
        &self,
        request: Request<QueryHistoricalInfoRequest>,
    ) -> Result<Response<QueryHistoricalInfoResponse>, Status> {
        info!("Received a gRPC request staking::historical_info");
        let req = StakingNodeQueryRequest::HistoricalInfo(request.into_inner().try_into()?);
        let response = self.app.typed_query(req)?;
        let response: StakingNodeQueryResponse = response.try_into()?;

        if let StakingNodeQueryResponse::HistoricalInfo(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(
                "An internal error occurred while querying the application state.",
            ))
        }
    }

    async fn pool(
        &self,
        request: Request<QueryPoolRequest>,
    ) -> Result<Response<QueryPoolResponse>, Status> {
        info!("Received a gRPC request staking::pool");
        let req = StakingNodeQueryRequest::Pool(request.into_inner().try_into()?);
        let response = self.app.typed_query(req)?;
        let response: StakingNodeQueryResponse = response.try_into()?;

        if let StakingNodeQueryResponse::Pool(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(
                "An internal error occurred while querying the application state.",
            ))
        }
    }

    async fn params(
        &self,
        request: Request<QueryParamsRequest>,
    ) -> Result<Response<QueryParamsResponse>, Status> {
        info!("Received a gRPC request staking::params");
        let req = StakingNodeQueryRequest::Params(request.into_inner().try_into()?);
        let response = self.app.typed_query(req)?;
        let response: StakingNodeQueryResponse = response.try_into()?;

        if let StakingNodeQueryResponse::Params(response) = response {
            Ok(Response::new(response.into()))
        } else {
            Err(Status::internal(
                "An internal error occurred while querying the application state.",
            ))
        }
    }
}

pub fn new<QH, QReq, QRes>(app: QH) -> QueryServer<StakingService<QH, QReq, QRes>>
where
    QReq: QueryRequest + Send + Sync + 'static + From<StakingNodeQueryRequest>,
    QRes: QueryResponse + Send + Sync + 'static + TryInto<StakingNodeQueryResponse, Error = Status>,
    QH: NodeQueryHandler<QReq, QRes>,
{
    let grpc_service = StakingService {
        app,
        _phantom: Default::default(),
    };
    QueryServer::new(grpc_service)
}
