use crate::{GenesisState, Keeper};
use gears::{
    context::init::InitContext,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    x::{
        keepers::{
            auth::AuthKeeper, bank::StakingBankKeeper as BankKeeper, staking::SlashingStakingKeeper,
        },
        module::Module,
    },
};

#[derive(Debug, Clone)]
pub struct ABCIHandler<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    AK: AuthKeeper<SK, M>,
    BK: BankKeeper<SK, M>,
    SSK: SlashingStakingKeeper<SK, M>,
    M: Module,
> {
    keeper: Keeper<SK, PSK, AK, BK, SSK, M>,
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: BankKeeper<SK, M>,
        SSK: SlashingStakingKeeper<SK, M>,
        M: Module,
    > ABCIHandler<SK, PSK, AK, BK, SSK, M>
{
    pub fn new(keeper: Keeper<SK, PSK, AK, BK, SSK, M>) -> Self {
        ABCIHandler { keeper }
    }

    pub fn genesis<DB: Database>(&self, ctx: &mut InitContext<'_, DB, SK>, genesis: GenesisState) {
        if let Err(e) = self.keeper.init_genesis(ctx, genesis) {
            panic!("Initialization of genesis failed with error:\n{e}")
        }
    }
}
