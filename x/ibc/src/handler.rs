use crate::{
    errors::tx::client::ClientErrors,
    keeper::{query::QueryKeeper, tx::TxKeeper},
    message::Message,
};
use database::Database;
use gears::{
    error::AppError,
    types::context::{query_context::QueryContext, tx_context::TxContext},
    x::params::ParamsSubspaceKey,
};
use prost::Message as ProstMessage;
use proto_messages::cosmos::ibc::protobuf::Protobuf;
use proto_messages::cosmos::ibc::tx::{
    MsgCreateClient, MsgRecoverClient, MsgUpdateClient, MsgUpgradeClient,
};
use store::StoreKey;

#[derive(Debug, Clone)]
pub struct Handler<SK: StoreKey, PSK: ParamsSubspaceKey> {
    tx_keeper: TxKeeper<SK, PSK>, // TODO: Should signature for Handler always be &self or allow &mut self?
    query_keeper: QueryKeeper<SK, PSK>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> Handler<SK, PSK> {
    pub fn new(tx_keeper: TxKeeper<SK, PSK>, query_keeper: QueryKeeper<SK, PSK>) -> Self {
        Self {
            tx_keeper,
            query_keeper,
        }
    }

    pub fn tx<DB: Database + Sync + Send>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: Message,
    ) -> Result<(), ClientErrors> {
        match msg {
            Message::ClientCreate(msg) => {
                let MsgCreateClient {
                    client_state,
                    consensus_state,
                    signer: _signer,
                } = msg;

                let _ = self
                    .tx_keeper
                    .client_create(ctx, &client_state, consensus_state.into())?;

                Ok(())
            }
            Message::ClientUpdate(msg) => {
                let MsgUpdateClient {
                    client_id,
                    client_message,
                    signer: _signer,
                } = msg;

                self.tx_keeper
                    .client_update(ctx, &client_id, client_message)?;

                Ok(())
            }
            Message::ClientUpgrade(msg) => {
                let MsgUpgradeClient {
                    client_id,
                    upgraded_client_state,
                    upgraded_consensus_state,
                    proof_upgrade_client,
                    proof_upgrade_consensus_state,
                    signer: _signer,
                } = msg;

                self.tx_keeper.client_upgrade(
                    ctx,
                    &client_id,
                    upgraded_client_state,
                    upgraded_consensus_state,
                    proof_upgrade_client,
                    proof_upgrade_consensus_state,
                )?;

                Ok(())
            }
            Message::RecoverClient(msg) => {
                let MsgRecoverClient {
                    subject_client_id,
                    substitute_client_id,
                    signer: _signer,
                } = msg;

                self.tx_keeper
                    .recover_client(ctx, &subject_client_id, &substitute_client_id)?;

                Ok(())
            }
        }
    }

    pub fn query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<'_, DB, SK>,
        query: tendermint::proto::abci::RequestQuery,
    ) -> Result<bytes::Bytes, AppError> {
        match query.path.as_str() {
            "/ibc.core.client.v1.Query/ClientParams" => Ok(self
                .query_keeper
                .client_params(ctx)
                .map_err(|_| AppError::AccountNotFound)?
                .encode_vec()
                .into()),
            "/ibc.core.client.v1.Query/UpgradedClientState" => Ok(self
                .query_keeper
                .client_state(
                    ctx,
                    ProstMessage::decode(query.data).map_err(|_| AppError::AccountNotFound)?,
                )
                .map_err(|_| AppError::AccountNotFound)?
                .encode_vec()
                .into()),
            "/ibc.core.client.v1.Query/ClientStates" => Ok(self
                .query_keeper
                .client_states(
                    ctx,
                    ProstMessage::decode(query.data).map_err(|_| AppError::AccountNotFound)?,
                )
                .map_err(|_| AppError::AccountNotFound)?
                .encode_vec()
                .into()),
            "/ibc.core.client.v1.Query/ClientStatus" => Ok(self
                .query_keeper
                .client_status(
                    ctx,
                    ProstMessage::decode(query.data).map_err(|_| AppError::AccountNotFound)?,
                )
                .map_err(|_| AppError::AccountNotFound)?
                .encode_vec()
                .into()),
            "/ibc.core.client.v1.Query/ConsensusStateHeights" => Ok(self
                .query_keeper
                .consensus_state_heights(
                    ctx,
                    ProstMessage::decode(query.data).map_err(|_| AppError::AccountNotFound)?,
                )
                .map_err(|_| AppError::AccountNotFound)?
                .encode_vec()
                .into()),
            "/ibc.core.client.v1.Query/ConsensusState" => Ok(self
                .query_keeper
                .consensus_state(
                    ctx,
                    ProstMessage::decode(query.data).map_err(|_| AppError::AccountNotFound)?,
                )
                .map_err(|_| AppError::AccountNotFound)?
                .encode_vec()
                .into()),
            "/ibc.core.client.v1.Query/ConsensusStates" => Ok(self
                .query_keeper
                .consensus_states(
                    ctx,
                    ProstMessage::decode(query.data).map_err(|_| AppError::AccountNotFound)?,
                )
                .map_err(|_| AppError::AccountNotFound)?
                .encode_vec()
                .into()),
            _ => Err(AppError::InvalidRequest("query path not found".into())),
        }
    }
}
