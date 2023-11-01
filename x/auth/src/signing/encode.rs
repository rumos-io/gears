use bytes::Bytes;

use super::{errors::SigningErrors, types::screen::Screen};

pub fn encode(screens: impl IntoIterator<Item = Screen>) -> Result<Bytes, SigningErrors> {
    let _arr = screens
        .into_iter()
        .map(|this| this.cbor())
        .collect::<Vec<_>>();

    unimplemented!()
}
