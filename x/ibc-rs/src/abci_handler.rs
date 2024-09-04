use crate::{
    errors, ics02_client::client::cli::query::client_states::STATES_URL, keeper::Keeper,
    message::Message, types::genesis::GenesisState,
};
use gears::{
    application::handlers::node::TxError,
    baseapp::errors::QueryError,
    context::{init::InitContext, query::QueryContext, tx::TxContext},
    core::errors::CoreError,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
};
use ibc::primitives::proto::Protobuf;
//use ibc::core::client::types::{
//    msgs::{MsgUpdateClient, MsgUpgradeClient},
//    proto::v1::MsgRecoverClient,
//};
use prost::Message as ProstMessage;

#[derive(Debug, Clone)]
pub struct ABCIHandler<SK: StoreKey, PSK: ParamsSubspaceKey> {
    //tx_keeper: TxKeeper<SK, PSK>, // TODO: Should signature for Handler always be &self or allow &mut self?
    //query_keeper: QueryKeeper<SK, PSK>,
    keeper: Keeper<SK, PSK>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> ABCIHandler<SK, PSK> {
    // pub fn new(tx_keeper: TxKeeper<SK, PSK>, query_keeper: QueryKeeper<SK, PSK>) -> Self {
    //     Self {
    //         tx_keeper,
    //         query_keeper,
    //     }
    // }

    pub fn new(keeper: Keeper<SK, PSK>) -> Self {
        Self { keeper }
    }

    pub fn msg<DB: Database + Sync + Send>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: Message,
    ) -> Result<(), TxError> {
        match msg {
            Message::ClientCreate(msg) => {
                // let MsgCreateClient {
                //     client_state:,
                //     consensus_state,
                //     signer: _signer,
                // } = msg;

                // let _ = self
                //     .tx_keeper
                //     .client_create(ctx, &client_state, consensus_state.into())?;

                self.keeper.client_create(ctx, msg);

                Ok(())
            } // Message::ClientUpdate(msg) => {
              //     let MsgUpdateClient {
              //         client_id,
              //         client_message,
              //         signer: _signer,
              //     } = msg;

              //     self.tx_keeper
              //         .client_update(ctx, &client_id, client_message)?;

              //     Ok(())
              // }
              // Message::ClientUpgrade(msg) => {
              //     let MsgUpgradeClient {
              //         client_id,
              //         upgraded_client_state,
              //         upgraded_consensus_state,
              //         proof_upgrade_client,
              //         proof_upgrade_consensus_state,
              //         signer: _signer,
              //     } = msg;

              //     self.tx_keeper.client_upgrade(
              //         ctx,
              //         &client_id,
              //         upgraded_client_state,
              //         upgraded_consensus_state,
              //         proof_upgrade_client,
              //         proof_upgrade_consensus_state,
              //     )?;

              //     Ok(())
              // }
              // Message::RecoverClient(msg) => {
              //     let MsgRecoverClient {
              //         subject_client_id,
              //         substitute_client_id,
              //         signer: _signer,
              //     } = msg;

              //     self.tx_keeper
              //         .recover_client(ctx, &subject_client_id, &substitute_client_id)?;

              //     Ok(())
              // }
        }
    }

    pub fn query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: gears::tendermint::types::request::query::RequestQuery,
    ) -> Result<Vec<u8>, QueryError> {
        match query.path.as_str() {
            // "/ibc.core.client.v1.Query/ClientParams" => {
            //     //Ok(self.query_keeper.client_params(ctx)?.encode_vec().into())
            //     Ok(vec![].into())
            // }
            // "/ibc.core.client.v1.Query/UpgradedClientState" => Ok(self
            //     .query_keeper
            //     .client_state(ctx, ProstMessage::decode(query.data)?)?
            //     .encode_vec()
            //     .into()),
            STATES_URL => Ok(self
                .keeper
                .client_states(
                    ctx,
                    ProstMessage::decode(query.data)
                        .map_err(|e| QueryError::Proto(e.to_string()))?,
                )
                .encode_vec()
                .into()),
            // "/ibc.core.client.v1.Query/ClientStatus" => Ok(self
            //     .query_keeper
            //     .client_status(ctx, ProstMessage::decode(query.data)?)?
            //     .encode_vec()
            //     .into()),
            // "/ibc.core.client.v1.Query/ConsensusStateHeights" => Ok(self
            //     .query_keeper
            //     .consensus_state_heights(ctx, ProstMessage::decode(query.data)?)?
            //     .encode_vec()
            //     .into()),
            // "/ibc.core.client.v1.Query/ConsensusState" => Ok(self
            //     .query_keeper
            //     .consensus_state(ctx, ProstMessage::decode(query.data)?)?
            //     .encode_vec()
            //     .into()),
            // "/ibc.core.client.v1.Query/ConsensusStates" => Ok(self
            //     .query_keeper
            //     .consensus_states(ctx, ProstMessage::decode(query.data)?)?
            //     .encode_vec()
            //     .into()),
            _ => Err(QueryError::PathNotFound),
        }
    }

    pub fn genesis<DB: Database>(&self, ctx: &mut InitContext<'_, DB, SK>, genesis: GenesisState) {
        self.keeper.init_genesis(ctx, genesis)
    }
}
