use bytes::Bytes;
use database::RocksDB;
use gears::types::context::Context;
use store::StoreKey;

use crate::signing::{encode::encode, types::textual_data::TextualData};

use super::{
    errors::SigningErrors,
    proto_file_resolver::ProtoFileResolver,
    renderer::ValueRenderer,
    types::{signer_data::SignerData, tx_data::TxData},
};

#[derive(Debug)]
pub struct SignModeHandler<T> {
    file_resolver: T,
}

impl<T: ProtoFileResolver> SignModeHandler<T> {
    pub fn sign_bytes_get<SK: StoreKey, VR: ValueRenderer>(
        &self,
        ctx: &Context<'_, '_, RocksDB, SK>,
        signer_data: SignerData,
        tx_data: TxData,
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

        Ok(encode(screens)?)
    }
}
