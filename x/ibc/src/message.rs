// use proto_messages::{
//     any::Any,
//     cosmos::ibc::tx::{MsgCreateClient, MsgRecoverClient, MsgUpdateClient, MsgUpgradeClient},
// };
// use proto_types::AccAddress;

use gears::{
    ibc::{address::AccAddress, any::google::Any},
    types::tx::TxMessage,
};

#[derive(Debug, Clone, serde::Serialize)]
pub enum Message {
    ClientCreate(MsgCreateClient),
    ClientUpdate(MsgUpdateClient),
    ClientUpgrade(MsgUpgradeClient),
    RecoverClient(MsgRecoverClient),
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
    type Error = gears::ibc::errors::Error;

    fn try_from(_value: Any) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}
