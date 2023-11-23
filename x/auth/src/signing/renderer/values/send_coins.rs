use std::ops::Div;

use database::RocksDB;
use gears::types::context::context::Context;
use num_bigint::ToBigUint;
use proto_messages::cosmos::{
    base::v1beta1::SendCoins,
    tx::v1beta1::{
        screen::{Content, Screen},
        tx_metadata::Metadata,
    },
};
use store::StoreKey;

use crate::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, PrimitiveValueRenderer, ValueRenderer,
};
// TODO: FIX:
impl<DefaultValueRenderer, SK: StoreKey> ValueRenderer<DefaultValueRenderer, SK> for SendCoins {
    fn format(
        &self,
        ctx: &Context<'_, '_, RocksDB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let inner_coins = self.clone().into_inner();

        let Metadata {
            display,
            denom_units,
            ..
        } = ctx.metadata_get();

        let mut final_content = String::new();

        for (i, coin) in inner_coins.into_iter().enumerate() {
            let coin_exp = denom_units.iter().find(|this| this.denom == coin.denom);
            let denom_exp = denom_units
                .iter()
                .find(|this| this.denom.as_ref() == display);

            let formated = match (coin_exp, denom_exp) {
                (Some(coin_exp), Some(denom_exp)) => {
                    let power = match coin_exp.exponent > denom_exp.exponent {
                        true => coin_exp.exponent - denom_exp.exponent,
                        false => denom_exp.exponent - coin_exp.exponent,
                    };

                    let disp_amount = coin.amount.clone().div(
                        10.to_biguint()
                            .expect("Should be able to parse number")
                            .pow(power),
                    );

                    let formated_amount = DefaultPrimitiveRenderer::format(disp_amount);

                    format!("{formated_amount} {display}")
                }
                _ => format!(
                    "{} {display}",
                    DefaultPrimitiveRenderer::format(coin.amount.clone())
                ),
            };

            if i == 0 {
                final_content = formated;
            } else {
                final_content = format!("{final_content}, {formated}");
            }
        }

        Ok(vec![Screen {
            title: "Fees".to_string(),
            content: Content::new(final_content)?,
            indent: None,
            expert: false,
        }])
    }
}

#[cfg(test)]
mod tests {
    use gears::types::context::context::Context;
    use num_bigint::ToBigUint;
    use proto_messages::cosmos::{
        base::v1beta1::{Coin, SendCoins},
        tx::v1beta1::screen::{Content, Screen},
    };

    use crate::signing::renderer::{
        value_renderer::{DefaultValueRenderer, ValueRenderer},
        values::test_mocks::{KeyMock, MockContext},
    };

    #[test]
    fn check_formate() -> anyhow::Result<()> {
        let coin = Coin {
            denom: "uatom".try_into()?,
            amount: 2000
                .to_biguint()
                .ok_or(anyhow::anyhow!("Failed to parse to biguint"))?,
        };

        let expected_screens = Screen {
            title: "Fees".to_string(),
            content: Content::new("0.002 ATOM".to_string())?,
            indent: None,
            expert: false,
        };
        let mut ctx = MockContext;

        let context: Context<'_, '_, database::RocksDB, KeyMock> =
            Context::DynamicContext(&mut ctx);

        let actual_screen = ValueRenderer::<DefaultValueRenderer, KeyMock>::format(
            &SendCoins::new(vec![coin])?,
            &context,
        )
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(vec![expected_screens], actual_screen);

        Ok(())
    }
}
