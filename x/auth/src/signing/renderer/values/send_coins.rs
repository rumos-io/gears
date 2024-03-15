use database::Database;
use gears::types::context::context::Context;
use proto_messages::cosmos::{
    base::v1beta1::SendCoins,
    tx::v1beta1::{
        screen::{Content, Screen},
        tx_metadata::Metadata,
    },
};
use proto_types::Uint256;
use store::StoreKey;

use crate::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, PrimitiveValueRenderer, ValueRenderer,
};
impl<SK: StoreKey, DB: Database> ValueRenderer<SK, DB> for SendCoins {
    fn format(
        &self,
        ctx: &Context<'_, '_, DB, SK>,
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

                    let denominator = Uint256::from(10u32).pow(power);

                    let amount = coin.amount;

                    let disp_amount = amount / denominator;

                    if disp_amount.is_zero() {
                        let reminder = amount % denominator;
                        let padding = power - 4; //TODO: this is hard coded for now but will be removed in the future when this is properly implemented
                        let padding_str = {
                            let mut var = String::with_capacity(padding as usize);
                            for _ in 0..padding {
                                var.push('0');
                            }
                            var
                        };

                        let mut formated_string = format!("0.{}{}", padding_str, reminder);

                        while formated_string.ends_with('0') {
                            let _ = formated_string.pop();
                        }

                        format!("{formated_string} {display}")
                    } else {
                        let formated_amount = DefaultPrimitiveRenderer::format(disp_amount);

                        format!("{formated_amount} {display}")
                    }
                }
                _ => format!(
                    "{} {display}",
                    DefaultPrimitiveRenderer::format(coin.amount)
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
    use proto_messages::cosmos::{
        base::v1beta1::{Coin, SendCoins},
        tx::v1beta1::screen::{Content, Screen},
    };
    use proto_types::Uint256;

    use crate::signing::renderer::{
        value_renderer::ValueRenderer,
        values::test_mocks::{KeyMock, MockContext},
    };

    #[test]
    fn check_formate() -> anyhow::Result<()> {
        let coin = Coin {
            denom: "uatom".try_into()?,
            amount: Uint256::from(2000u32),
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

        let actual_screen =
            ValueRenderer::<KeyMock, _>::format(&SendCoins::new(vec![coin])?, &context)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(vec![expected_screens], actual_screen);

        Ok(())
    }
}
