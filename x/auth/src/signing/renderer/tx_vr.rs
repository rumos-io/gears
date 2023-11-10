// use std::marker::PhantomData;

// use database::RocksDB;
// use gears::types::context::context::Context;
// use once_cell::sync::Lazy;
// use proto_messages::cosmos::tx::v1beta1::{
//     envelope::Envelope,
//     message::Message,
//     public_key::PublicKey,
//     screen::{Content, Indent, Screen},
//     textual_data::TextualData,
// };
// use regex::Regex;
// use store::StoreKey;

// use crate::signing::{errors::SigningErrors, hasher::hash_get};

// use super::{message_vr::MessageValueRendererTrait, vr_trait::ValueRendererTrait};

// static MSG_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("([0-9]+) Any").expect("Invalid regex"));

// // static INVERSE_MSG_REG: Lazy<Regex> =
// //     Lazy::new(|| Regex::new("This transaction has ([0-9]+) Messages?").expect("Invalid regex"));

// pub trait TxValueRendererTrait<SK, M>: ValueRendererTrait<TextualData<M>, SK>
// where
//     SK: StoreKey,
//     M: Message,
// {
// }

// pub struct TxValueRenderer<MR>(PhantomData<MR>);

// impl<MR, SK, M> TxValueRendererTrait<SK, M> for TxValueRenderer<MR>
// where
//     MR: MessageValueRendererTrait<Envelope, SK>,
//     SK: StoreKey,
//     M: Message,
// {
// }

// impl<MR, SK, M> ValueRendererTrait<TextualData<M>, SK> for TxValueRenderer<MR>
// where
//     MR: MessageValueRendererTrait<Envelope, SK>,
//     SK: StoreKey,
//     M: Message,
// {
//     fn format(
//         ctx: &Context<'_, '_, RocksDB, SK>,
//         value: TextualData<M>,
//     ) -> Result<Vec<Screen>, SigningErrors> {
//         let TextualData {
//             body_bytes,
//             auth_info_bytes,
//             signer_data,
//             body,
//             auth_info,
//         } = value;

//         let envelope = Envelope {
//             chain_id: signer_data.chain_id,
//             account_number: signer_data.account_number,
//             sequence: signer_data.sequence,
//             address: signer_data.address,
//             public_key: signer_data.pub_key.clone(),
//             memo: body.memo,
//             fees: auth_info.fee.amount.unwrap_or_default().into_inner(),
//             fee_payer: auth_info
//                 .fee
//                 .payer
//                 .expect("Fee payer must be set")
//                 .into_inner()
//                 .into(), // TODO:
//             fee_granter: auth_info.fee.granter,
//             tip: Vec::new(),       //TODO
//             tipper: String::new(), //TODO
//             gas_limit: auth_info.fee.gas_limit,
//             timeout_height: body.timeout_height,
//             other_signer: auth_info
//                 .signer_infos
//                 .into_iter()
//                 .filter(|this| {
//                     this.public_key != Some(PublicKey::Secp256k1(signer_data.pub_key.clone()))
//                 })
//                 .collect(),
//             hash_of_raw_bytes: hash_get(&body_bytes, &auth_info_bytes),
//         };

//         let mut screens = MR::format(ctx, envelope)?;

//         // Since we're value-rendering the (internal) envelope message, we do some
//         // postprocessing. First, we remove first envelope header screen, and
//         // unindent 1 level.

//         // Remove 1st screen
//         let _ = screens.swap_remove(0); // TODO: Is order important?
//         screens.iter_mut().try_for_each(|this| {
//             Ok(
//                 this.indent = match Indent::new(this.indent.into_inner() - 1) {
//                     Ok(var) => var,
//                     Err(e) => return Err(SigningErrors::CustomError(e.to_string())),
//                 },
//             )
//         })?;

//         expertify(screens.iter_mut());

//         for screen in &mut screens {
//             if screen.indent.clone().into_inner() != 0 {
//                 continue;
//             }

//             // Replace:
//             // "Message: <N> Any"
//             // with:
//             // "This transaction has <N> Message"
//             if screen.title == "Message" {
//                 let content = screen.content.clone();

//                 let matches = MSG_REGEX.captures(content.as_ref());
//                 if let Some(matches) = matches {
//                     if matches.len() > 0 {
//                         screen.title.clear();
//                         screen.content =
//                             Content::new(format!("This transaction has {} Message", matches.len()))
//                                 .map_err(|e| {
//                                     SigningErrors::CustomError(format!(
//                                         "Unreachable: Content should be valid. {}",
//                                         e.to_string()
//                                     ))
//                                 })?; // Content still valid, but let's not unwrap here

//                         if matches.get(1).is_some_and(|this| this.as_str() != "1") {
//                             screen.content = {
//                                 let mut inner = screen.content.clone().into_inner();
//                                 inner.push('s');
//                                 Content::new(inner).map_err(|e| {
//                                     SigningErrors::CustomError(format!(
//                                         "Unreachable: Content should be valid. {}",
//                                         e.to_string()
//                                     ))
//                                 })?
//                             }
//                         }
//                     }
//                 }
//             }
//         }

//         Ok(screens)
//     }

//     fn parse(
//         _ctx: &Context<'_, '_, RocksDB, SK>,
//         _screens: impl IntoIterator<Item = Screen>,
//     ) -> Result<TextualData<M>, SigningErrors> {
//         todo!()
//     }
// }

// /// `expertify` marks all screens starting from `fromIdx` as expert, and stops
// /// just before it finds the next screen with Indent==0 (unless it's a "End of"
// /// termination screen). It modifies screens in-place.
// fn expertify<'a>(screens: impl Iterator<Item = &'a mut Screen>) {
//     for screen in screens {
//         if screen.indent.clone().into_inner() != 0
//             && screen.content.as_ref() == &format!("End of {}", screen.title)
//         {
//             continue;
//         }

//         static EXPERT: [&'static str; 10] = [
//             "Address",
//             "Public key",
//             "Fee payer",
//             "Fee granter",
//             "Gas limit",
//             "Timeout height",
//             "Other signer",
//             "Extension options",
//             "Non critical extension options",
//             "Hash of raw bytes",
//         ];

//         // Do expert fields.
//         if EXPERT.contains(&screen.title.as_str()) {
//             screen.expert = true;
//         }
//     }
// }
