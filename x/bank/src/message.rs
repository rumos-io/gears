use gears::{
    derive::AppMessage,
    signing::{
        handler::MetadataGetter,
        renderer::value_renderer::{RenderError, ValueRenderer},
    },
    types::{msg::send::MsgSend, rendering::screen::Screen},
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, AppMessage)]
#[serde(tag = "@type")]
pub enum Message {
    #[serde(rename = "/cosmos.bank.v1beta1.MsgSend")]
    #[msg(url(path = MsgSend::TYPE_URL))]
    Send(MsgSend),
}

impl ValueRenderer for Message {
    fn format<MG: MetadataGetter>(&self, get_metadata: &MG) -> Result<Vec<Screen>, RenderError> {
        match self {
            Message::Send(msg) => msg.format(get_metadata),
        }
    }
}
