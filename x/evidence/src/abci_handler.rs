use crate::{
    types::{Equivocation, Evidence, QueryAllEvidenceRequest, QueryEvidenceRequest},
    GenesisState, Keeper,
};
use gears::{
    baseapp::{errors::QueryError, QueryResponse},
    context::{block::BlockContext, init::InitContext, query::QueryContext},
    core::{any::google::Any, Protobuf},
    store::{database::Database, StoreKey},
    tendermint::types::{
        proto::info::EvidenceType,
        request::{begin_block::RequestBeginBlock, query::RequestQuery},
    },
    x::{
        keepers::{slashing::EvidenceSlashingKeeper, staking::SlashingStakingKeeper},
        module::Module,
    },
};

#[derive(Debug, Clone)]
pub struct ABCIHandler<
    SK: StoreKey,
    StkK: SlashingStakingKeeper<SK, M>,
    SlsK: EvidenceSlashingKeeper<SK, M>,
    E: Evidence + Default,
    M: Module,
> where
    <E as std::convert::TryFrom<Any>>::Error: std::fmt::Debug,
{
    keeper: Keeper<SK, StkK, SlsK, E, M>,
}

impl<
        SK: StoreKey,
        StkK: SlashingStakingKeeper<SK, M>,
        SlsK: EvidenceSlashingKeeper<SK, M>,
        E: Evidence + Default,
        M: Module,
    > ABCIHandler<SK, StkK, SlsK, E, M>
where
    <E as std::convert::TryFrom<Any>>::Error: std::fmt::Debug,
{
    pub fn new(keeper: Keeper<SK, StkK, SlsK, E, M>) -> Self {
        ABCIHandler { keeper }
    }

    pub fn genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        genesis: GenesisState<E>,
    ) {
        if let Err(e) = self.keeper.init_genesis(ctx, genesis) {
            panic!("Cannot perform evidence genesis.\n{e}");
        }
    }

    /// begin_block iterates through and handles any newly discovered evidence of
    /// misbehavior submitted by Tendermint. Currently, only equivocation is handled.
    pub fn begin_block<DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, SK>,
        request: RequestBeginBlock,
    ) {
        for evidence in request.byzantine_validators {
            match evidence.kind() {
                // It's still ongoing discussion how should we treat and slash attacks with
                // premeditation. So for now we agree to treat them in the same way.
                EvidenceType::DuplicateVote | EvidenceType::LightClientAttack => {
                    let ev: Equivocation = evidence.into();
                    if let Err(e) = self.keeper.handle_equivocation_evidence(ctx, &ev) {
                        panic!("Cannot perform evidence begin block routine.\n{e}");
                    }
                }
                EvidenceType::Unknown => {
                    tracing::error!("ignored unknown evidence type: {}", evidence.r#type);
                }
            }
        }
    }

    pub fn query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: RequestQuery,
    ) -> Result<prost::bytes::Bytes, QueryError> {
        match query.path.as_str() {
            "/cosmos.evidence.v1beta1.Query/Evidence" => {
                let req = QueryEvidenceRequest::decode_vec(&query.data)?;

                Ok(self.keeper.query_evidence(ctx, req).into_bytes().into())
            }
            "/cosmos.evidence.v1beta1.Query/AllEvidence" => {
                let req = QueryAllEvidenceRequest::decode_vec(&query.data)?;

                Ok(self.keeper.query_all_evidence(ctx, req).into_bytes().into())
            }
            _ => Err(QueryError::PathNotFound),
        }
    }

    // pub fn typed_query<DB: Database + Send + Sync>(
    //     &self,
    //     ctx: &QueryContext<DB, SK>,
    //     query: SlashingNodeQueryRequest,
    // ) -> SlashingNodeQueryResponse {
    //     match query {
    //         SlashingNodeQueryRequest::SigningInfos(req) => {
    //             SlashingNodeQueryResponse::SigningInfos(self.query_signing_infos(ctx, req))
    //         }
    //         SlashingNodeQueryRequest::Params(req) => {
    //             SlashingNodeQueryResponse::Params(self.keeper.query_params(ctx, req))
    //         }
    //     }
    // }
}
