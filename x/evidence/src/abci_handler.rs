use crate::{
    errors::TxEvidenceError,
    message::Message,
    types::{
        Equivocation, Evidence, QueryAllEvidenceRequest, QueryAllEvidenceResponse,
        QueryEvidenceRequest, QueryEvidenceResponse,
    },
    GenesisState, Keeper,
};
use gears::{
    baseapp::{errors::QueryError, QueryResponse},
    context::{block::BlockContext, init::InitContext, query::QueryContext, tx::TxContext},
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
pub enum EvidenceNodeQueryRequest {
    Evidence(QueryEvidenceRequest),
    AllEvidence(QueryAllEvidenceRequest),
}
#[derive(Debug, Clone)]
pub enum EvidenceNodeQueryResponse {
    Evidence(QueryEvidenceResponse),
    AllEvidence(QueryAllEvidenceResponse),
}

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

    pub fn tx<DB: Database + Sync + Send>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: &Message,
    ) -> Result<(), TxEvidenceError> {
        match msg {
            Message::SubmitEvidence(msg) => Ok(self.keeper.submit_evidence_cmd(ctx, msg)?),
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

    pub fn typed_query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: EvidenceNodeQueryRequest,
    ) -> EvidenceNodeQueryResponse {
        match query {
            EvidenceNodeQueryRequest::Evidence(req) => {
                EvidenceNodeQueryResponse::Evidence(self.keeper.query_evidence(ctx, req))
            }
            EvidenceNodeQueryRequest::AllEvidence(req) => {
                EvidenceNodeQueryResponse::AllEvidence(self.keeper.query_all_evidence(ctx, req))
            }
        }
    }
}
