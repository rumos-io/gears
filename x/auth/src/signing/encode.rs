use bytes::Bytes;
use proto_messages::cosmos::tx::v1beta1::screen::Screen;

use super::errors::SigningErrors;

pub fn encode(screens: impl IntoIterator<Item = Screen>) -> Result<Bytes, SigningErrors> {
    let _arr = screens
        .into_iter()
        .map(|this| this.cbor())
        .collect::<Vec<_>>();

    unimplemented!()
}
