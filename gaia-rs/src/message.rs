use gears::{
    signing::renderer::value_renderer::{RenderError, ValueRenderer},
    types::{denom::Denom, rendering::screen::Screen, tx::metadata::Metadata},
};
use gears_derive::RoutingMessage;
use serde::Serialize;

#[derive(Debug, Clone, RoutingMessage, Serialize)]
#[serde(untagged)]
pub enum Message {
    #[gears(url = "/cosmos.bank.v1beta1")]
    Bank(bank::Message),
    // #[gears(url = "/ibc.core.client.v1")]
    // Ibc(ibc::message::Message),
}

impl ValueRenderer for Message {
    fn format<F: Fn(&Denom) -> Option<Metadata>>(
        &self,
        get_metadata: &F,
    ) -> Result<Vec<Screen>, RenderError> {
        match self {
            Message::Bank(msg) => msg.format(get_metadata),
            // Message::Ibc(_) => Err(Error::NotImplemented),
        }
    }
}
