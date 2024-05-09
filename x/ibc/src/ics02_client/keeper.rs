use bytes::Bytes;
use gears::{
    application::handlers::client,
    params::ParamsSubspaceKey,
    store::{database::Database, QueryableKVStore, StoreKey},
    types::context::{
        init_context::InitContext, query_context::QueryContext, tx_context::TxContext,
        QueryableContext,
    },
};
use ibc::primitives::proto::Protobuf;
use prost::Message;

use crate::{
    ics02_client::types::{client_state::ClientState, query::IdentifiedClientState},
    params::CLIENT_STATE_KEY,
};

use super::{
    client::cli::query::client_states, message::MsgCreateClient, params::ClientParamsKeeper,
    types::query::QueryClientStatesResponse, GenesisState,
};
use gears::store::TransactionalKVStore;
use gears::types::context::TransactionalContext;
use ibc::{
    core::{
        client::types::{
            msgs::{ClientMsg, MsgCreateClient as IBCMsgCreateClient},
            proto::v1::{MsgCreateClient as RawMsgCreateClient, QueryClientStatesRequest},
        },
        entrypoint::dispatch,
        handler::types::msgs::MsgEnvelope,
        host::types::identifiers::ClientId,
    },
    primitives::proto::Any,
};

pub const KEY_NEXT_CLIENT_SEQUENCE: &[u8; 18] = b"nextClientSequence";
pub const KEY_CLIENT_STORE_PREFIX: &str = "clients";

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

    // pub fn client_create<DB: Database>(
    //     &self,
    //     ctx: &mut TxContext<'_, DB, SK>,
    //     msg: MsgCreateClient,
    //     // client_state: &(impl ClientStateCommon
    //     //       + ClientStateExecution<ContextShim<'a, 'b, DB, SK>>
    //     //       + ClientStateValidation<ContextShim<'a, 'b, DB, SK>>),
    //     // consensus_state: WrappedConsensusState,
    // ) {
    //     //todo!()

    //     let mut ctx = ClientContext {
    //         gears_ctx: ctx,
    //         store_key: self.store_key.clone(),
    //         client_params_keeper: self.client_params_keeper.clone(),
    //     };
    //     let mut router = ClientRouter;
    //     // let msg = MsgEnvelope::Client(ClientMsg::CreateClient(IBCMsgCreateClient {
    //     //     client_state: todo!(),
    //     //     consensus_state: todo!(),
    //     //     signer: ,
    //     // }));

    //     let raw_msg = RawMsgCreateClient::from(msg);
    //     let msg = IBCMsgCreateClient::try_from(raw_msg).unwrap();
    //     let msg = MsgEnvelope::Client(ClientMsg::CreateClient(msg));

    //     dispatch(&mut ctx, &mut router, msg).unwrap()
    // }

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

    /// Query all client states
    pub fn client_states<DB: Database>(
        &self,
        ctx: &QueryContext<'_, DB, SK>,
        _req: QueryClientStatesRequest,
    ) -> QueryClientStatesResponse {
        let store = ctx
            .kv_store(&self.store_key)
            .prefix_store(format!("{KEY_CLIENT_STORE_PREFIX}").into_bytes());

        let mut client_states = vec![];

        for (key, raw_state) in store.range(..) {
            let key = String::from_utf8(key).unwrap(); // TODO: unwrap

            let key_split: Vec<&str> = key.split("/").collect();

            let [_, client_id, this_client_state_key] = key_split[..] else {
                continue;
            };

            if this_client_state_key != CLIENT_STATE_KEY {
                continue;
            }

            let client_id: ClientId = client_id.parse().unwrap(); //TODO: unwrap

            //let any: Any = Any::decode::<Bytes>(raw_state.into()).unwrap(); //TODO: unwrap
            //let client_id = key_split[1]; //TODO: check length

            let identified = IdentifiedClientState {
                client_id,
                client_state: ClientState::decode_vec(&raw_state).unwrap(),
            };

            client_states.push(identified);
        }

        // TODO: sort client_states (as is done in ibc-go)? https://github.com/cosmos/ibc-go/blob/46e020640e66f9043c14c53a4d215a5b457d6703/modules/core/02-client/keeper/grpc_query.go#L91

        QueryClientStatesResponse {
            client_states,
            pagination: None,
        }
    }

    // fn store_client_state(
    //     &mut self,
    //     client_state_path: ibc::core::host::types::path::ClientStatePath,
    //     client_state: Self::ClientStateRef,
    // ) -> Result<(), ibc::core::handler::types::error::ContextError> {
    //     //TODO: check impl

    //     //dbg!(client_state.clone());

    //     //let data = serde_json::to_string(&client_state.clone()).unwrap();

    //     //let data = format!("{:?}", client_state.clone());
    //     //std::fs::write("tmp.json", data).expect("Unable to write file");

    //     let any: Any = client_state.into();
    //     let encoded_bytes = any.to_vec();

    //     // println!("encoded bytes:\n {:?}", encoded_bytes.clone());

    //     // let prefix = format!("{KEY_CLIENT_STORE_PREFIX}/{}/", client_state_path.0).into_bytes();
    //     // println!("prefix: {:?}", prefix.clone());
    //     // println!("key: {:?}", CLIENT_STATE_KEY.bytes());

    //     self.gears_ctx
    //         .kv_store_mut(&self.store_key)
    //         .prefix_store_mut(
    //             format!("{KEY_CLIENT_STORE_PREFIX}/{}/", client_state_path.0).into_bytes(),
    //         )
    //         .set(CLIENT_STATE_KEY.bytes(), encoded_bytes);

    //     Ok(())
    // }
}
