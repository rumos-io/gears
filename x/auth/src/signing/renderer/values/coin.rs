use std::ops::Mul;

use database::RocksDB;
use gears::types::context::context::Context;
use ibc_proto::cosmos::base::v1beta1::Coin;
use proto_messages::cosmos::tx::v1beta1::{
    screen::{Content, Indent, Screen},
    tx_metadata::Metadata,
};
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use store::StoreKey;

use crate::signing::renderer::{
    fields::decimal::DecimalString,
    value_renderer::{DefaultPrimitiveRenderer, PrimitiveValueRenderer, ValueRenderer},
};

impl<DefaultValueRenderer, SK: StoreKey> ValueRenderer<DefaultValueRenderer, SK> for Coin {
    /// Format `Coin` into `Screen`.
    fn format(
        &self,
        ctx: &Context<'_, '_, RocksDB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let content_price = self.amount.parse::<i64>()?;

        let Metadata {
            display,
            denom_units,
            ..
        } = ctx.metadata_get();

        let coin_exp = denom_units.iter().find(|this| this.denom == self.denom);
        let denom_exp = denom_units.iter().find(|this| this.denom == display);

        match (coin_exp, denom_exp) {
            (Some(coin_exp), Some(denom_exp)) => {
                let power = match coin_exp.exponent > denom_exp.exponent {
                    true => coin_exp.exponent - denom_exp.exponent,
                    false => denom_exp.exponent - coin_exp.exponent,
                } as u64;

                let disp_amount: String = self
                    .amount
                    .parse::<Decimal>()?
                    .mul(dec!(10).powu(power))
                    .to_string();

                let formated_amount = DefaultPrimitiveRenderer::format(DecimalString(&disp_amount));

                let screen = Screen {
                    title: "Amount".to_string(),
                    content: Content::new(formated_amount)?,
                    indent: Some(Indent::new(2)?),
                    expert: false,
                };

                Ok(vec![screen])
            }
            _ => Ok(vec![Screen {
                title: "Amount".to_string(),
                content: Content::new(DefaultPrimitiveRenderer::format(content_price))?,
                indent: Some(Indent::new(2)?),
                expert: false,
            }]),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use anyhow::Ok;
//     use gears::types::context::tx_context::TxContext;
//     use ibc_proto::cosmos::base::v1beta1::Coin;
//     use proto_messages::cosmos::tx::v1beta1::screen::{Content, Indent, Screen};
//     use store::StoreKey;

//     use crate::signing::renderer::value_renderer::{
//         MessageValueRenderer, DefaultValueRenderer, ValueRenderer,
//     };

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

//         let actual_screen = coin.format::<DefaultValueRenderer, StoreKey>(ctx);

//         assert!(actual_screens.is_ok(), "Failed to retrieve screens");
//         assert_eq!(vec![expected_screens], actual_screens.expect("Unreachable"));

//         Ok(())
//     }
// }
