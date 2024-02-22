use std::sync::{Arc, RwLock};

use database::Database;
use gears::{types::context::tx_context::TxContext, x::params::ParamsSubspaceKey};
use proto_messages::cosmos::ibc::tx::MsgCreateClient;
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
            Message::ClientUpdate(_) => todo!(),
            Message::ClientUpgrade(_) => todo!(),
            Message::SubmitMisbehaviour(_) => todo!(),
            Message::RecoverClient(_) => todo!(),
        }
    }
}
