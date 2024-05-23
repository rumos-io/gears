use gears_derive::RoutingMessage;

use crate::ics02_client::message::MsgCreateClient;

#[derive(Debug, Clone, serde::Serialize, RoutingMessage)]
pub enum Message {
    #[gears(url = "/ibc.core.client.v1")]
    ClientCreate(MsgCreateClient),
    // ClientUpdate(MsgUpdateClient),
    // ClientUpgrade(MsgUpgradeClient),
    // RecoverClient(MsgRecoverClient),
}
