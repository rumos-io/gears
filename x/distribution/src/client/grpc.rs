// TODO: uncomment and add impl for new version of ibc_proto
// use crate::{SlashingNodeQueryRequest, SlashingNodeQueryResponse};
// use gears::baseapp::{NodeQueryHandler, QueryRequest, QueryResponse};
// use ibc_proto::cosmos::slashing::v1beta1::{
//     query_server::{Query, QueryServer},
//     QueryParamsRequest, QueryParamsResponse,
// };
// use std::marker::PhantomData;
// use tonic::{Request, Response, Status};
//
// #[derive(Debug, Default)]
// pub struct SlashingService<QH, QReq, QRes> {
//     app: QH,
//     _phantom: PhantomData<(QReq, QRes)>,
// }
//
// #[tonic::async_trait]
// impl<
//         QReq: Send + Sync + 'static,
//         QRes: Send + Sync + 'static,
//         QH: NodeQueryHandler<QReq, QRes>,
//     > Query for SlashingService<QH, QReq, QRes>
// where
//     QReq: QueryRequest + From<SlashingNodeQueryRequest>,
//     QRes: QueryResponse + TryInto<SlashingNodeQueryResponse, Error = Status>,
// {
//     // fill
// }
//
// pub fn new<QH, QReq, QRes>(app: QH) -> QueryServer<SlashingService<QH, QReq, QRes>>
// where
//     QReq: QueryRequest + Send + Sync + 'static + From<SlashingNodeQueryRequest>,
//     QRes: QueryResponse
//         + Send
//         + Sync
//         + 'static
//         + TryInto<SlashingNodeQueryResponse, Error = Status>,
//     QH: NodeQueryHandler<QReq, QRes>,
// {
//     let slashing_service = SlashingService {
//         app,
//         _phantom: Default::default(),
//     };
//     QueryServer::new(slashing_service)
// }
