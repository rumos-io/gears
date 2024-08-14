use crate::{
    types::{Equivocation, Evidence},
    GenesisState, Keeper,
};
use gears::{
    context::{block::BlockContext, init::InitContext},
    core::any::google::Any,
    store::{database::Database, StoreKey},
    tendermint::types::{proto::info::EvidenceType, request::begin_block::RequestBeginBlock},
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
}
