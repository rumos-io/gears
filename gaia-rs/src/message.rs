use gears::derive::AppMessage;
use gears::{
    signing::{
        handler::MetadataGetter,
        renderer::value_renderer::{RenderError, ValueRenderer},
    },
    types::rendering::screen::Screen,
};
use serde::Serialize;

#[derive(Debug, Clone, AppMessage, Serialize)]
#[serde(untagged)]
pub enum Message {
    #[msg(url(string = "/cosmos.bank.v1beta1"))]
    Bank(bank::Message),
    #[msg(url(string = "/cosmos.staking.v1beta1"))]
    Staking(staking::Message),
    #[msg(url(string = "/ibc.core.client.v1"))]
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
