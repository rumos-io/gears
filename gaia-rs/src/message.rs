use gears::derive::RoutingMessage;
use gears::{
    signing::{
        handler::MetadataGetter,
        renderer::value_renderer::{RenderError, ValueRenderer},
    },
    types::rendering::screen::Screen,
};
use serde::Serialize;

#[derive(Debug, Clone, RoutingMessage, Serialize)]
#[serde(untagged)]
pub enum Message {
    #[gears(url = "/cosmos.bank.v1beta1")]
    Bank(bank::Message),
    #[gears(url = "/cosmos.staking.v1beta1")]
    Staking(staking::Message),
    #[gears(url = "/ibc.core.client.v1")]
    IBC(ibc_rs::message::Message),
}

impl ValueRenderer for Message {
    fn format<MG: MetadataGetter>(&self, get_metadata: &MG) -> Result<Vec<Screen>, RenderError> {
        match self {
            Message::Bank(msg) => msg.format(get_metadata),
            Message::Staking(_) => Err(RenderError::NotImplemented),
            Message::IBC(_) => Err(RenderError::NotImplemented),
        }
    }
}
