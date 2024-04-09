use auth::signing::renderer::value_renderer::{Error, ValueRenderer};
use gears::{
    proto_types::Denom,
    types::{rendering::screen::Screen, tx::metadata::Metadata},
};
use gears_derive::RoutingMessage;
// use proto_messages::cosmos::tx::v1beta1::{screen::Screen, tx_metadata::Metadata};
// use proto_types::{AccAddress, Denom};
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
    ) -> Result<Vec<Screen>, Error> {
        match self {
            Message::Bank(msg) => msg.format(get_metadata),
            // Message::Ibc(_) => Err(Error::NotImplemented),
        }
    }
}
