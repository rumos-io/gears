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
        request: Request<QueryParamsRequest>,
    ) -> Result<Response<QueryParamsResponse>, Status> {
        info!("Received a gRPC request staking::params");
        // let response = QueryParamsResponse {
        //     params: Some(Params {
        //         unbonding_time: Some(Duration {
        //             seconds: 1814400,
        //             nanos: 0,
        //         }),
        //         max_validators: 12,
        //         max_entries: 100,
        //         historical_entries: 10,
        //         bond_denom: "uatom".to_string(),
        //         min_commission_rate: "0.1".to_string(),
        //     }),
        // };

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
