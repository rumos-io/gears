// /!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\
/// TODO: move to staking module when it's hooked into gaia-rs
/// /!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\/!\
use gears::baseapp::{NodeQueryHandler, QueryRequest, QueryResponse};
use ibc_proto::{
    cosmos::staking::v1beta1::{
        query_server::{Query, QueryServer},
        Params, QueryDelegationRequest, QueryDelegationResponse, QueryDelegatorDelegationsRequest,
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
    },
    google::protobuf::Duration,
};
use std::marker::PhantomData;
use tonic::{Request, Response, Status};
use tracing::info;

#[derive(Debug, Default)]
pub struct StakingService<QH, QReq, QRes> {
    _app: QH,
    _phantom: PhantomData<(QReq, QRes)>,
}

#[tonic::async_trait]
impl<
        QReq: Send + Sync + 'static,
        QRes: Send + Sync + 'static,
        QH: NodeQueryHandler<QReq, QRes>,
    > Query for StakingService<QH, QReq, QRes>
where
// QReq: QueryRequest + From<StakingNodeQueryRequest>,
// QRes: QueryResponse + TryInto<StakingNodeQueryResponse, Error = Status>,
{
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
        _request: Request<QueryParamsRequest>,
    ) -> Result<Response<QueryParamsResponse>, Status> {
        info!("Received a gRPC request staking::params");
        // TODO: replace hard coded values with actual values from the app
        let response = QueryParamsResponse {
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

pub fn new<QH, QReq, QRes>(_app: QH) -> QueryServer<StakingService<QH, QReq, QRes>>
where
    QReq: QueryRequest + Send + Sync + 'static, // + From<StakingNodeQueryRequest>,
    QRes: QueryResponse + Send + Sync + 'static, // + TryInto<StakingNodeQueryResponse, Error = Status>,
    QH: NodeQueryHandler<QReq, QRes>,
{
    let bank_service = StakingService {
        _app,
        _phantom: Default::default(),
    };
    QueryServer::new(bank_service)
}
