use gears::{
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    types::context::init_context::InitContext,
};

use super::{params::ClientParamsKeeper, GenesisState};
use gears::store::TransactionalKVStore;
use gears::types::context::TransactionalContext;

const KEY_NEXT_CLIENT_SEQUENCE: &[u8; 18] = b"nextClientSequence";

#[derive(Debug, Clone)]
pub struct Keeper<SK, PSK> {
    store_key: SK,
    client_params_keeper: ClientParamsKeeper<SK, PSK>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> Keeper<SK, PSK> {
    pub fn new(
        store_key: SK,
        params_keeper: gears::params::Keeper<SK, PSK>,
        params_subspace_key: PSK,
    ) -> Self {
        let client_params_keeper = ClientParamsKeeper {
            params_keeper,
            params_subspace_key,
        };
        Self {
            store_key,
            client_params_keeper,
        }
    }

    pub fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        genesis: GenesisState,
    ) {
        self.client_params_keeper.set(ctx, genesis.params.clone());

        // TODO: the following lines(from ibc-go) have not been implemented yet:

        // // Set all client metadata first. This will allow client keeper to overwrite client and consensus state keys
        // // if clients accidentally write to ClientKeeper reserved keys.
        // if len(gs.ClientsMetadata) != 0 {
        // 	k.SetAllClientMetadata(ctx, gs.ClientsMetadata)
        // }

        // for _, client := range gs.Clients {
        // 	cs, ok := client.ClientState.GetCachedValue().(exported.ClientState)
        // 	if !ok {
        // 		panic("invalid client state")
        // 	}

        // 	if !gs.Params.IsAllowedClient(cs.ClientType()) {
        // 		panic(fmt.Sprintf("client state type %s is not registered on the allowlist", cs.ClientType()))
        // 	}

        // 	k.SetClientState(ctx, client.ClientId, cs)
        // }

        // for _, cs := range gs.ClientsConsensus {
        // 	for _, consState := range cs.ConsensusStates {
        // 		consensusState, ok := consState.ConsensusState.GetCachedValue().(exported.ConsensusState)
        // 		if !ok {
        // 			panic(fmt.Sprintf("invalid consensus state with client ID %s at height %s", cs.ClientId, consState.Height))
        // 		}

        // 		k.SetClientConsensusState(ctx, cs.ClientId, consState.Height, consensusState)
        // 	}
        // }

        self.set_next_client_sequence(ctx, genesis.next_client_sequence);
    }

    pub fn set_next_client_sequence<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        sequence: u64,
    ) {
        let ibc_store = ctx.kv_store_mut(&self.store_key);
        ibc_store.set(KEY_NEXT_CLIENT_SEQUENCE.to_owned(), sequence.to_be_bytes());
    }
}

// // SetNextClientSequence sets the next client sequence to the store.
// func (k Keeper) SetNextClientSequence(ctx sdk.Context, sequence uint64) {
// 	store := ctx.KVStore(k.storeKey)
// 	bz := sdk.Uint64ToBigEndian(sequence)
// 	store.Set([]byte(types.KeyNextClientSequence), bz)
// }
