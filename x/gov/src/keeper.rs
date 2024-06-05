use gears::{
    application::keepers::params::ParamsKeeper,
    context::init::InitContext,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
};

use crate::{
    errors::SERDE_JSON_CONVERSION, genesis::GovGenesisState, params::GovParamsKeeper,
    types::deposit::Deposit,
};

const PROPOSAL_ID_KEY: [u8; 1] = [0x03];
pub(crate) const KEY_DEPOSIT_PREFIX: [u8; 1] = [0x10];

#[allow(dead_code)]
pub struct GovKeeper<SK: StoreKey, PSK: ParamsSubspaceKey> {
    store_key: SK,
    gov_params_keeper: GovParamsKeeper<PSK>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> GovKeeper<SK, PSK> {
    pub fn new(store_key: SK, params_subspace_key: PSK) -> Self {
        Self {
            store_key,
            gov_params_keeper: GovParamsKeeper {
                params_subspace_key,
            },
        }
    }

    pub fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        GovGenesisState {
            starting_proposal_id,
            deposits,
            votes,
            proposals,
            params,
        }: GovGenesisState,
    ) {
        {
            let mut store = ctx.kv_store_mut(&self.store_key);
            store.set(PROPOSAL_ID_KEY, starting_proposal_id.to_be_bytes())
        }
        self.gov_params_keeper.set(ctx, params);

        // https://github.com/cosmos/cosmos-sdk/blob/d3f09c222243bb3da3464969f0366330dcb977a8/x/gov/genesis.go#L18
        //     // check if the deposits pool account exists
        // moduleAcc := k.GetGovernanceAccount(ctx)
        // if moduleAcc == nil {
        // 	panic(fmt.Sprintf("%s module account has not been set", types.ModuleName))
        // }

        let _total_deposits = {
            let mut total_deposits = Vec::with_capacity(deposits.len());
            for deposit in deposits {
                self.deposit_set(ctx, &deposit);
                total_deposits.push(deposit.amount);
            }

            total_deposits.into_iter().flatten().collect::<Vec<_>>()
        };
    }

    fn deposit_set<DB: Database>(&self, ctx: &mut InitContext<'_, DB, SK>, deposit: &Deposit) {
        let mut store = ctx.kv_store_mut(&self.store_key);
        store.set(
            deposit.key(),
            serde_json::to_vec(deposit).expect(SERDE_JSON_CONVERSION),
        ) // TODO:NOW CORRECT SERIALIZATION
    }
}
