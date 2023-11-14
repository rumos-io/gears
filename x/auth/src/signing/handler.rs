// use database::RocksDB;
// use gears::types::context::context::Context;
// use proto_messages::cosmos::tx::v1beta1::{
//     cbor::Cbor, message::Message, signer_data::SignerData, textual_data::TextualData,
//     tx_data::TxData,
// };
// use store::StoreKey;

// use super::renderer::value_renderer::ValueRenderer;

// #[derive(Debug)]
// pub struct SignModeHandler {}

// impl SignModeHandler {
//     pub fn sign_bytes_get<M: Message, SK: StoreKey, VR: ValueRenderer<TextualData<M>, SK>>(
//         &self,
//         ctx: &Context<'_, '_, RocksDB, SK>,
//         signer_data: SignerData,
//         tx_data: TxData<M>,
//     ) -> Result<Vec<u8>, SigningErrors> {
//         let data = TextualData {
//             body_bytes: tx_data.body_bytes.0,
//             auth_info_bytes: tx_data.auth_info_bytes.0,
//             signer_data,
//             body: tx_data.body,
//             auth_info: tx_data.auth_info,
//         }; // *Note:* smth we need bytes so I save bytes and serealized version too.

//         let screens = VR::format(ctx, data)?;

//         let mut bytes = Vec::new();

//         screens.encode(&mut bytes)?;

//         Ok(bytes)
//     }
// }
