// TODO: uncomment and add impl for new version of ibc_proto
// use crate::{DistributionNodeQueryRequest, DistributionNodeQueryResponse};
// use gears::baseapp::{NodeQueryHandler, QueryRequest, QueryResponse};
// use ibc_proto::cosmos::distribution::v1beta1::{
//     query_server::{Query, QueryServer},
//     QueryParamsRequest, QueryParamsResponse,
// };
// use std::marker::PhantomData;
// use tonic::{Request, Response, Status};
//
// #[derive(Debug, Default)]
// pub struct DistributionService<QH, QReq, QRes> {
//     app: QH,
//     _phantom: PhantomData<(QReq, QRes)>,
// }
//
// #[tonic::async_trait]
// impl<
//         QReq: Send + Sync + 'static,
//         QRes: Send + Sync + 'static,
//         QH: NodeQueryHandler<QReq, QRes>,
//     > Query for DistributionService<QH, QReq, QRes>
// where
//     QReq: QueryRequest + From<DistributionNodeQueryRequest>,
//     QRes: QueryResponse + TryInto<DistributionNodeQueryResponse, Error = Status>,
// {
//     // fill
// }
//
// pub fn new<QH, QReq, QRes>(app: QH) -> QueryServer<DistributionService<QH, QReq, QRes>>
// where
//     QReq: QueryRequest + Send + Sync + 'static + From<DistributionNodeQueryRequest>,
//     QRes: QueryResponse
//         + Send
//         + Sync
//         + 'static
//         + TryInto<DistributionNodeQueryResponse, Error = Status>,
//     QH: NodeQueryHandler<QReq, QRes>,
// {
//     let distribution_service = DistributionService {
//         app,
//         _phantom: Default::default(),
//     };
//     QueryServer::new(distribution_service)
// }
