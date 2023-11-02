use bytes::Bytes;
use database::RocksDB;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::{
    message::Message, signer_data::SignerData, textual_data::TextualData, tx_data::TxData,
};
use store::StoreKey;

use crate::signing::encode::encode;

use super::{
    errors::SigningErrors, proto_file_resolver::ProtoFileResolver, renderer::ValueRenderer,
};

#[derive(Debug)]
pub struct SignModeHandler<T> {
    file_resolver: T,
}

impl<T: ProtoFileResolver> SignModeHandler<T> {
    pub fn sign_bytes_get<M: Message, SK: StoreKey, VR: ValueRenderer>(
        &self,
        ctx: &Context<'_, '_, RocksDB, SK>,
        signer_data: SignerData,
        tx_data: TxData<M>,
    ) -> Result<Bytes, SigningErrors> {
        let data = TextualData {
            body_bytes: tx_data.body_bytes,
            auth_info_bytes: tx_data.auth_info_bytes,
            signer_data,
        };

        let screens = VR::format(
            ctx,
            data.value_get(), /*protoreflect.ValueOf(data.ProtoReflect())) */
        )?;

        encode(screens)
    }
}
