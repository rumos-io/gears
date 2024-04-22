use gears::{
    core::{address::AccAddress, any::google::Any},
    //    tendermint::types::proto::consensus::Consensus,
    types::tx::TxMessage,
};

use crate::ics02_client::message::MsgCreateClient;

#[derive(Debug, Clone, serde::Serialize)]
pub enum Message {
    ClientCreate(MsgCreateClient),
    // ClientUpdate(MsgUpdateClient),
    // ClientUpgrade(MsgUpgradeClient),
    // RecoverClient(MsgRecoverClient),
}

impl TxMessage for Message {
    fn get_signers(&self) -> Vec<&AccAddress> {
        unimplemented!()
    }

    fn validate_basic(&self) -> Result<(), String> {
        unimplemented!()
    }

    fn type_url(&self) -> &'static str {
        unimplemented!()
    }
}

impl From<Message> for Any {
    fn from(_msg: Message) -> Self {
        unimplemented!()
    }
}

impl TryFrom<Any> for Message {
    type Error = gears::core::errors::Error;

    fn try_from(_value: Any) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}
