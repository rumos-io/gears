use gears::{
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    types::context::{init_context::InitContext, tx_context::TxContext},
};

use super::{
    context::{ClientContext, ClientRouter},
    message::MsgCreateClient,
    params::ClientParamsKeeper,
    GenesisState,
};
use gears::store::TransactionalKVStore;
use gears::types::context::TransactionalContext;
use ibc::core::{
    client::types::{
        msgs::{ClientMsg, MsgCreateClient as IBCMsgCreateClient},
        proto::v1::MsgCreateClient as RawMsgCreateClient,
    },
    entrypoint::dispatch,
    handler::types::msgs::MsgEnvelope,
};

pub const KEY_NEXT_CLIENT_SEQUENCE: &[u8; 18] = b"nextClientSequence";

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

    pub fn client_create<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: MsgCreateClient,
        // client_state: &(impl ClientStateCommon
        //       + ClientStateExecution<ContextShim<'a, 'b, DB, SK>>
        //       + ClientStateValidation<ContextShim<'a, 'b, DB, SK>>),
        // consensus_state: WrappedConsensusState,
    ) {
        //todo!()

        let mut ctx = ClientContext {
            gears_ctx: ctx,
            store_key: self.store_key.clone(),
            client_params_keeper: self.client_params_keeper.clone(),
        };
        let mut router = ClientRouter;
        // let msg = MsgEnvelope::Client(ClientMsg::CreateClient(IBCMsgCreateClient {
        //     client_state: todo!(),
        //     consensus_state: todo!(),
        //     signer: ,
        // }));

        let raw_msg = RawMsgCreateClient::from(msg);
        let msg = IBCMsgCreateClient::try_from(raw_msg).unwrap();
        let msg = MsgEnvelope::Client(ClientMsg::CreateClient(msg));

        dispatch(&mut ctx, &mut router, msg).unwrap()
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
