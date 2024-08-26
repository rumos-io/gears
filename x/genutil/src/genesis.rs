use gears::{
    baseapp::genesis::Genesis,
    context::{InfallibleContext, TransactionalContext},
    store::database::Database,
    tendermint::types::proto::validator::ValidatorUpdate,
};
use staking::CreateValidator;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GenutilGenesis {
    pub gen_txs: Vec<CreateValidator>,
}

impl Genesis for GenutilGenesis {
    fn add_genesis_account(
        &mut self,
        _address: gears::types::address::AccAddress,
        _coins: gears::types::base::coins::UnsignedCoins,
    ) -> Result<(), gears::baseapp::genesis::GenesisError> {
        Ok(())
    }
}

pub trait GenutilStakingBankKeeper<SK> {
    fn apply_and_return_validator_set_updates<
        DB: Database,
        CTX: TransactionalContext<DB, SK> + InfallibleContext<DB, SK>,
    >(
        &self,
        ctx: &mut CTX,
    ) -> anyhow::Result<Vec<ValidatorUpdate>>;
}
