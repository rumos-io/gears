use bnum::types::U256 as BU256;
use proto_messages::cosmos::{
    base::v1beta1::SendCoins,
    tx::v1beta1::{
        screen::{Content, Screen},
        tx_metadata::Metadata,
    },
};
use proto_types::Denom;

use crate::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, PrimitiveValueRenderer, ValueRenderer,
};
impl ValueRenderer for SendCoins {
    fn format<F: Fn(&Denom) -> Option<Metadata>>(
        &self,
        get_metadata: &F,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let inner_coins = self.clone().into_inner();

        let mut final_content = String::new();

        for (i, coin) in inner_coins.into_iter().enumerate() {
            let Metadata {
                display,
                denom_units,
                ..
            } = get_metadata(&coin.denom).ok_or("metadata not found")?; //TODO: check that returning an error is the right thing to do here

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

                    let denominator = BU256::from_digit(10).pow(power);

                    let amount = coin.amount;

                    let disp_amount = amount.0.div(denominator);

                    if disp_amount.is_zero() {
                        let reminder = amount.0 % denominator;
                        let padding = power - amount.0.trailing_zeros();
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
                    DefaultPrimitiveRenderer::format(coin.amount.0.clone())
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
    use bnum::types::U256 as BU256;
    use proto_messages::cosmos::{
        base::v1beta1::{Coin, SendCoins},
        tx::v1beta1::screen::{Content, Screen},
    };

    use crate::signing::renderer::{
        value_renderer::ValueRenderer, values::test_functions::get_metadata,
    };

    #[test]
    fn check_format() -> anyhow::Result<()> {
        let coin = Coin {
            denom: "uatom".try_into()?,
            amount: BU256::from_digit(2000).into(),
        };

        let expected_screens = Screen {
            title: "Fees".to_string(),
            content: Content::new("0.002 ATOM".to_string())?,
            indent: None,
            expert: false,
        };

        let actual_screen = ValueRenderer::format(&SendCoins::new(vec![coin])?, &get_metadata)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(vec![expected_screens], actual_screen);

        Ok(())
    }

    #[test]
    fn check_format_multi_denom() -> anyhow::Result<()> {
        let coin1 = Coin {
            denom: "uatom".try_into()?,
            amount: BU256::from_digit(2000).into(),
        };

        let coin2 = Coin {
            denom: "uon".try_into()?,
            amount: BU256::from_digit(2000).into(),
        };

        let expected_screens = Screen {
            title: "Fees".to_string(),
            content: Content::new("0.002 ATOM, 0.002 UON".to_string())?,
            indent: None,
            expert: false,
        };

        let actual_screen =
            ValueRenderer::format(&SendCoins::new(vec![coin1, coin2])?, &get_metadata)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(vec![expected_screens], actual_screen);

        Ok(())
    }
}
