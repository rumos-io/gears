// use std::ops::Mul;

// use database::RocksDB;
// use gears::types::context::context::Context;
// use proto_messages::cosmos::{
//     base::v1beta1::Coin,
//     tx::v1beta1::{
//         screen::{Content, Indent, Screen},
//         tx_metadata::Metadata,
//     },
// };
// use rust_decimal::{Decimal, MathematicalOps};
// use rust_decimal_macros::dec;
// use store::StoreKey;

// use crate::signing::renderer::{
//     fields::decimal::DecimalString,
//     value_renderer::{DefaultPrimitiveRenderer, PrimitiveValueRenderer, ValueRenderer},
// };

// impl<DefaultValueRenderer, SK: StoreKey> ValueRenderer<DefaultValueRenderer, SK> for Coin {
//     /// Format `Coin` into `Screen`.
//     fn format(
//         &self,
//         ctx: &Context<'_, '_, RocksDB, SK>,
//     ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
//         // let content_price = self.amount.parse::<i64>()?;
//         let content_price = self.amount;

//         let Metadata {
//             display,
//             denom_units,
//             ..
//         } = ctx.metadata_get();

//         let coin_exp = denom_units.iter().find(|this| this.denom == self.denom);
//         let denom_exp = denom_units.iter().find(|this| this.denom == display);

//         match (coin_exp, denom_exp) {
//             (Some(coin_exp), Some(denom_exp)) => {
//                 let power = match coin_exp.exponent > denom_exp.exponent {
//                     true => coin_exp.exponent - denom_exp.exponent,
//                     false => denom_exp.exponent - coin_exp.exponent,
//                 } as u64;

//                 let disp_amount: String = self
//                     .amount
//                     .parse::<Decimal>()?
//                     .mul(dec!(10).powu(power))
//                     .to_string();

//                 let formated_amount = DefaultPrimitiveRenderer::format(DecimalString(&disp_amount));

//                 let screen = Screen {
//                     title: "Amount".to_string(),
//                     content: Content::new(formated_amount)?,
//                     indent: Some(Indent::new(2)?),
//                     expert: false,
//                 };

//                 Ok(vec![screen])
//             }
//             _ => Ok(vec![Screen {
//                 title: "Amount".to_string(),
//                 content: Content::new(DefaultPrimitiveRenderer::format(content_price))?,
//                 indent: Some(Indent::new(2)?),
//                 expert: false,
//             }]),
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use crate::signing::renderer::value_renderer::{DefaultValueRenderer, ValueRenderer};
//     use anyhow::Ok;
//     use database::{Database, PrefixDB};
//     use gears::types::context::context::{Context, ContextTrait};
//     use ibc_proto::cosmos::base::v1beta1::Coin;
//     use proto_messages::cosmos::tx::v1beta1::{
//         screen::{Content, Indent, Screen},
//         tx_metadata::{DenomUnit, Metadata},
//     };
//     use store::StoreKey;
//     use strum::EnumIter;

//     #[test]
//     fn coin_formatting() -> anyhow::Result<()> {
//         let coin = Coin {
//             denom: "uatom".to_string(),
//             amount: "10000000".to_string(),
//         };

//         let expected_screens = Screen {
//             title: "Amount".to_string(),
//             content: Content::new("10 ATOM".to_string())?,
//             indent: Some(Indent::new(2)?),
//             expert: false,
//         };
//         let mut ctx = MockContext;

//         let context: Context<'_, '_, database::RocksDB, KeyMock> =
//             Context::DynamicContext(&mut ctx);

//         let actual_screen = ValueRenderer::<DefaultValueRenderer, KeyMock>::format(&coin, &context);

//         assert!(actual_screen.is_ok(), "Failed to retrieve screens");
//         assert_eq!(vec![expected_screens], actual_screen.unwrap());

//         Ok(())
//     }

//     // We use custom implementation instead of mock
//     // 1. Mockall requires generic parameters to be 'static
//     // 2. Diffuclties exporting mock on other crates
//     pub struct MockContext;

//     impl<T: Database, SK: StoreKey> ContextTrait<T, SK> for MockContext {
//         fn height(&self) -> u64 {
//             unimplemented!()
//         }

//         fn chain_id(&self) -> &str {
//             unimplemented!()
//         }

//         fn push_event(&mut self, _: tendermint_informal::abci::Event) {
//             unimplemented!()
//         }

//         fn append_events(&mut self, _: Vec<tendermint_informal::abci::Event>) {
//             unimplemented!()
//         }

//         fn metadata_get(&self) -> Metadata {
//             Metadata {
//                 description: String::new(),
//                 denom_units: vec![
//                     DenomUnit {
//                         denom: "ATOM".into(),
//                         exponent: 6,
//                         aliases: Vec::new(),
//                     },
//                     DenomUnit {
//                         denom: "uatom".into(),
//                         exponent: 0,
//                         aliases: Vec::new(),
//                     },
//                 ],
//                 base: "uatom".into(),
//                 display: "ATOM".into(),
//                 name: String::new(),
//                 symbol: String::new(),
//                 uri: String::new(),
//                 uri_hash: None,
//             }
//         }

//         fn get_kv_store(&self, _: &SK) -> &store::KVStore<PrefixDB<T>> {
//             unimplemented!()
//         }

//         fn get_mutable_kv_store(&mut self, _: &SK) -> &mut store::KVStore<PrefixDB<T>> {
//             unimplemented!()
//         }
//     }

//     #[derive(EnumIter, Debug, PartialEq, Eq, Hash, Clone)]
//     pub enum KeyMock {
//         Bank,
//         Auth,
//         Params,
//     }

//     impl StoreKey for KeyMock {
//         fn name(&self) -> &'static str {
//             match self {
//                 KeyMock::Bank => "bank",
//                 KeyMock::Auth => "acc",
//                 KeyMock::Params => "params",
//             }
//         }
//     }
// }
