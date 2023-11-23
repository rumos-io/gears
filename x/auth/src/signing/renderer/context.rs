// //! Context formating

// use database::RocksDB;
// use gears::types::context::context::Context;
// use proto_messages::cosmos::tx::v1beta1::{
//     envelope::Envelope,
//     message::Message,
//     public_key::PublicKey,
//     screen::{Content, Indent, Screen},
//     textual_data::TextualData,
// };
// use store::StoreKey;

// use crate::signing::hasher::hash_get;

// use super::value_renderer::{ContextRenderer, ValueRenderer};

// impl<DefaultContextRenderer, DVR, SK: StoreKey, M: Message + ValueRenderer<DVR, SK>>
//     ContextRenderer<DefaultContextRenderer, DVR, SK, M> for Context<'_, '_, RocksDB, SK>
// {
//     fn format(&self, value: TextualData<M>) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
//         let TextualData {
//             body_bytes,
//             auth_info_bytes,
//             signer_data,
//             body,
//             auth_info,
//         } = value;

//         let mut final_screens = Vec::<Screen>::new();

//         final_screens.append( &mut signer_data.fo)

//         // let envelope = Envelope {
//         //     chain_id: signer_data.chain_id,
//         //     account_number: signer_data.account_number,
//         //     sequence: signer_data.sequence,
//         //     address: signer_data.address,
//         //     public_key: signer_data.pub_key.clone(),
//         //     memo: body.memo,
//         //     fees: auth_info.fee.amount.unwrap_or_default().into_inner(),
//         //     fee_payer: auth_info
//         //         .fee
//         //         .payer
//         //         .expect("Fee payer must be set")
//         //         .into_inner()
//         //         .into(), // TODO:
//         //     fee_granter: auth_info.fee.granter,
//         //     tip: Vec::new(),       //TODO
//         //     tipper: String::new(), //TODO
//         //     gas_limit: auth_info.fee.gas_limit,
//         //     timeout_height: body.timeout_height,
//         //     other_signer: auth_info
//         //         .signer_infos
//         //         .into_iter()
//         //         .filter(|this| {
//         //             this.public_key != Some(PublicKey::Secp256k1(signer_data.pub_key.clone()))
//         //         })
//         //         .collect(),
//         //     hash_of_raw_bytes: hash_get(&body_bytes, &auth_info_bytes),
//         // };

//         // let message_screens = body.

//         Ok(vec![])
//     }
// }
