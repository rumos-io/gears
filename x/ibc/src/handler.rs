use std::sync::{Arc, RwLock};

use database::Database;
use gears::{types::context::tx_context::TxContext, x::params::ParamsSubspaceKey};
use proto_messages::cosmos::ibc::tx::{MsgCreateClient, MsgUpdateClient, MsgUpgradeClient};
use store::StoreKey;

use crate::{errors::ModuleErrors, keeper::Keeper, message::Message};

#[derive(Debug, Clone)]
pub struct Handler<SK: StoreKey, PSK: ParamsSubspaceKey> {
    keeper: Arc<RwLock<Keeper<SK, PSK>>>, // TODO: Should signature for Handler always be &self or allow &mut self?
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> Handler<SK, PSK> {
    pub fn new(keeper: Keeper<SK, PSK>) -> Self {
        Self {
            keeper: Arc::new(RwLock::new(keeper)),
        }
    }

    pub fn tx<DB: Database + Sync + Send>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: Message,
    ) -> Result<(), ModuleErrors> {
        match msg {
            Message::ClientCreate(msg) => {
                let MsgCreateClient {
                    client_state,
                    consensus_state,
                    signer: _signer,
                } = msg;

                let _ = self.keeper.write().expect("poisoned lock").client_create(
                    ctx,
                    &client_state,
                    consensus_state.into(),
                )?;

                Ok(())
            }
            Message::ClientUpdate(msg) => {
                let MsgUpdateClient {
                    client_id,
                    client_message,
                    signer: _signer,
                } = msg;

                self.keeper.write().expect("poisoned lock").client_update(
                    ctx,
                    &client_id,
                    client_message,
                )?;

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

                self.keeper.write().expect("poisoned lock").client_upgrade(
                    ctx,
                    &client_id,
                    upgraded_client_state,
                    upgraded_consensus_state,
                    proof_upgrade_client,
                    proof_upgrade_consensus_state,
                )?;

                Ok(())
            }
            Message::RecoverClient(_) => todo!(),
        }
    }
}
