use crate::{types::Evidence, GenesisState, Keeper};
use gears::{
    context::init::InitContext,
    core::any::google::Any,
    store::{database::Database, StoreKey},
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
}
