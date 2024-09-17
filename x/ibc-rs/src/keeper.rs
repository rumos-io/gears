use gears::{
    context::{init::InitContext, query::QueryContext, tx::TxContext},
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
};

use crate::{
    ics02_client::{
        message::MsgCreateClient, types::query::QueryClientStatesResponse, Keeper as ClientKeeper,
    },
    ics03_connection::Keeper as ConnectionKeeper,
    ics04_channel::Keeper as ChannelKeeper,
    types::{
        context::{ClientRouter, Context},
        genesis::GenesisState,
    },
};
use ibc::core::{client::types::proto::v1::QueryClientStatesRequest, entrypoint::dispatch};

#[derive(Debug, Clone)]
pub struct Keeper<SK, PSK> {
    _store_key: SK, //TOOD: remove this
    client_keeper: ClientKeeper<SK, PSK>,
    connection_keeper: ConnectionKeeper<SK, PSK>,
    channel_keeper: ChannelKeeper<SK>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> Keeper<SK, PSK> {
    pub fn new(store_key: SK, params_subspace_key: PSK) -> Self {
        Self {
            _store_key: store_key.clone(),
            client_keeper: ClientKeeper::new(store_key.clone(), params_subspace_key.clone()),
            connection_keeper: ConnectionKeeper::new(store_key.clone(), params_subspace_key),
            channel_keeper: ChannelKeeper::new(store_key),
        }
    }

    pub fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        genesis: GenesisState,
    ) {
        self.client_keeper.init_genesis(ctx, genesis.client_genesis);
        self.connection_keeper
            .init_genesis(ctx, genesis.connection_genesis);
        self.channel_keeper
            .init_genesis(ctx, genesis.channel_genesis);
    }

    pub fn client_create<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: MsgCreateClient,
    ) -> Result<(), ibc::core::handler::types::error::ContextError> {
        let mut ctx = Context {
            gears_ctx: ctx,
            client_keeper: &self.client_keeper,
            connection_keeper: &self.connection_keeper,
            channel_keeper: &self.channel_keeper,
            store_key: self._store_key.clone(),
        };

        let mut router = ClientRouter;

        dispatch(&mut ctx, &mut router, msg.into())
    }

    pub fn client_states<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        req: QueryClientStatesRequest,
    ) -> QueryClientStatesResponse {
        self.client_keeper.client_states(ctx, req)
    }
}
