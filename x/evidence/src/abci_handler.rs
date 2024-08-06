use gears::{
    context::init::InitContext,
    store::{database::Database, StoreKey},
    x::{
        keepers::{slashing::EvidenceSlashingKeeper, staking::SlashingStakingKeeper},
        module::Module,
    },
};

use crate::{types::Evidence, GenesisState, Keeper};

#[derive(Debug, Clone)]
pub struct ABCIHandler<
    'a,
    SK: StoreKey,
    StkK: SlashingStakingKeeper<SK, M>,
    SlsK: EvidenceSlashingKeeper<SK, M>,
    DB: Database,
    E: Evidence,
    M: Module,
> {
    keeper: Keeper<'a, SK, StkK, SlsK, DB, E, M>,
}

impl<
        'a,
        SK: StoreKey,
        StkK: SlashingStakingKeeper<SK, M>,
        SlsK: EvidenceSlashingKeeper<SK, M>,
        DB: Database,
        E: Evidence + Default,
        M: Module,
    > ABCIHandler<'a, SK, StkK, SlsK, DB, E, M>
{
    pub fn new(keeper: Keeper<'a, SK, StkK, SlsK, DB, E, M>) -> Self {
        ABCIHandler { keeper }
    }

    pub fn genesis(&self, ctx: &mut InitContext<'_, DB, SK>, genesis: GenesisState) {
        self.keeper.init_genesis(ctx, genesis)
    }
}
