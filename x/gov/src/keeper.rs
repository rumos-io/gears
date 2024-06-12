use gears::{
    application::keepers::params::ParamsKeeper,
    context::init::InitContext,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
};

use crate::{
    errors::SERDE_JSON_CONVERSION, genesis::GovGenesisState, params::GovParamsKeeper,
    types::proposal::ProposalStatus,
};

const PROPOSAL_ID_KEY: [u8; 1] = [0x03];
pub(crate) const KEY_PROPOSAL_PREFIX: [u8; 1] = [0x00];
pub(crate) const KEY_DEPOSIT_PREFIX: [u8; 1] = [0x10];
pub(crate) const KEY_VOTES_PREFIX: [u8; 1] = [0x20];

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

    // https://github.com/cosmos/cosmos-sdk/blob/d3f09c222243bb3da3464969f0366330dcb977a8/x/gov/genesis.go#L12
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

        {
            let mut store_mut = ctx.kv_store_mut(&self.store_key);

            let _total_deposits = {
                let mut total_deposits = Vec::with_capacity(deposits.len());
                for deposit in deposits {
                    store_mut.set(
                        deposit.key(),
                        serde_json::to_vec(&deposit).expect(SERDE_JSON_CONVERSION),
                    ); // TODO:NOW IS THIS CORRECT SERIALIZATION?
                    total_deposits.push(deposit.amount);
                }

                total_deposits.into_iter().flatten().collect::<Vec<_>>()
            };

            for vote in votes {
                store_mut.set(
                    vote.key(),
                    serde_json::to_vec(&vote).expect(SERDE_JSON_CONVERSION),
                )
            }

            for proposal in proposals {
                match proposal.status {
                    ProposalStatus::DepositPeriod => {
                        store_mut.set(
                            proposal.inactive_queue_key(),
                            proposal.proposal_id.to_be_bytes(),
                        );
                    }
                    ProposalStatus::VotingPeriod => store_mut.set(
                        proposal.active_queue_key(),
                        proposal.proposal_id.to_be_bytes(),
                    ),
                    _ => (),
                }

                store_mut.set(
                    proposal.key(),
                    serde_json::to_vec(&proposal).expect(SERDE_JSON_CONVERSION),
                );
            }
        }
    }
}
