use database::RocksDB;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::{
    cbor::Cbor, message::Message, signer_data::SignerData, textual_data::TextualData,
    tx_data::TxData,
};
use store::StoreKey;

use super::{
    errors::SigningErrors,
    renderer::value_renderer::{DefaultValueRenderer, ValueRenderer},
};

#[derive(Debug)]
pub struct SignModeHandler {}

impl SignModeHandler {
    pub fn sign_bytes_get<M: Message + ValueRenderer<DefaultValueRenderer, SK>, SK: StoreKey>(
        &self,
        ctx: &Context<'_, '_, RocksDB, SK>,
        signer_data: SignerData,
        tx_data: TxData<M>,
    ) -> Result<Vec<u8>, SigningErrors> {
        let data = TextualData {
            body_bytes: tx_data.body_bytes.0,
            auth_info_bytes: tx_data.auth_info_bytes.0,
            signer_data,
            body: tx_data.body,
            auth_info: tx_data.auth_info,
        }; // *Note:* smth we need bytes so I save bytes and serialized version too.

        let screens = data
            .format(ctx)
            .map_err(|e| SigningErrors::CustomError(e.to_string()))?;

        let mut bytes = Vec::new();

        screens.encode(&mut bytes)?;

        Ok(bytes)
    }
}
