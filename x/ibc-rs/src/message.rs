use gears::derive::AppMessage;

use crate::ics02_client::message::MsgCreateClient;

#[derive(Debug, Clone, serde::Serialize, AppMessage)]
pub enum Message {
    #[msg(url(string = "/ibc.core.client.v1"))]
    ClientCreate(MsgCreateClient),
    // ClientUpdate(MsgUpdateClient),
    // ClientUpgrade(MsgUpgradeClient),
    // RecoverClient(MsgRecoverClient),
}
