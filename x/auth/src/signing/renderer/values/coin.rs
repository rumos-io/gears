use crate::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, PrimitiveValueRenderer, ValueRenderer,
};
use proto_messages::cosmos::{
    base::v1beta1::Coin,
    tx::v1beta1::{
        screen::{Content, Screen},
        tx_metadata::Metadata,
    },
};
use proto_types::Uint256;
use proto_types::{Decimal256, Denom};

impl ValueRenderer for Coin {
    /// Format `Coin` into `Screen`.
    fn format<F: Fn(&Denom) -> Option<Metadata>>(
        &self,
        get_metadata: &F,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let Some(metadata) = get_metadata(&self.denom) else {
            let display = self.denom.to_string();
            return Ok(vec![Screen {
                title: "".to_string(),
                content: Content::new(format!(
                    "{} {display}",
                    DefaultPrimitiveRenderer::format(self.amount)
                ))
                .expect("this String is not empty so it will never fail to parse"),
                indent: None,
                expert: false,
            }]);
        };

        let Metadata {
            display,
            denom_units,
            ..
        } = metadata;

        if display.is_empty() || self.denom.to_string() == display {
            let display = self.denom.to_string();
            return Ok(vec![Screen {
                title: "".to_string(),
                content: Content::new(format!(
                    "{} {display}",
                    DefaultPrimitiveRenderer::format(self.amount)
                ))
                .expect("this String is not empty so it will never fail to parse"),
                indent: None,
                expert: false,
            }]);
        }

        let coin_exp = denom_units.iter().find(|this| this.denom == self.denom);
        let denom_exp = denom_units
            .iter()
            .find(|this| this.denom.as_ref() == display);

        match (coin_exp, denom_exp) {
            (Some(coin_exp), Some(denom_exp)) => {
                let power = match coin_exp.exponent > denom_exp.exponent {
                    true => coin_exp.exponent - denom_exp.exponent,
                    false => denom_exp.exponent - coin_exp.exponent,
                };

                let amount = Decimal256::from_atomics(self.amount, 0)?;
                let scaling = Uint256::from(10u32).checked_pow(power)?;

                let disp_amount = amount / scaling;

                let formatted_amount = DefaultPrimitiveRenderer::format(disp_amount);

                let screen = Screen {
                    title: "".to_string(),
                    content: Content::new(format!("{formatted_amount} {display}"))
                        .expect("this String is not empty so it will never fail to parse"),
                    indent: None,
                    expert: false,
                };

                Ok(vec![screen])
            }
            _ => {
                let display = self.denom.to_string();
                Ok(vec![Screen {
                    title: "".to_string(),
                    content: Content::new(format!(
                        "{} {display}",
                        DefaultPrimitiveRenderer::format(self.amount)
                    ))
                    .expect("this String is not empty so it will never fail to parse"),
                    indent: None,
                    expert: false,
                }])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::signing::renderer::{
        value_renderer::ValueRenderer, values::test_functions::get_metadata,
    };
    use anyhow::Ok;
    use proto_messages::cosmos::{
        base::v1beta1::Coin,
        tx::v1beta1::screen::{Content, Screen},
    };
    use proto_types::Uint256;

    #[test]
    fn coin_formatting() -> anyhow::Result<()> {
        let coin = Coin {
            denom: "uatom".try_into()?,
            amount: Uint256::from(10000000_u64).into(),
        };

        let expected_screens = Screen {
            title: "".to_string(),
            content: Content::new("10 ATOM".to_string())?,
            indent: None,
            expert: false,
        };

        let actual_screen = ValueRenderer::format(&coin, &get_metadata);

        assert!(actual_screen.is_ok(), "Failed to retrieve screens");
        assert_eq!(vec![expected_screens], actual_screen.unwrap());

        Ok(())
    }

    #[test]
    fn coin_formatting_small_amounts_works() -> anyhow::Result<()> {
        let coin = Coin {
            denom: "uatom".try_into()?,
            amount: Uint256::from(1u8).into(),
        };

        let expected_screens = Screen {
            title: "".to_string(),
            content: Content::new("0.000001 ATOM".to_string())?,
            indent: None,
            expert: false,
        };

        let actual_screen = ValueRenderer::format(&coin, &get_metadata);

        assert!(actual_screen.is_ok(), "Failed to retrieve screens");
        assert_eq!(vec![expected_screens], actual_screen.unwrap());

        Ok(())
    }

    #[test]
    fn coin_formatting_zero_amount_works() -> anyhow::Result<()> {
        let coin = Coin {
            denom: "uatom".try_into()?,
            amount: Uint256::from(0u8).into(),
        };

        let expected_screens = Screen {
            title: "".to_string(),
            content: Content::new("0 ATOM".to_string())?,
            indent: None,
            expert: false,
        };

        let actual_screen = ValueRenderer::format(&coin, &get_metadata);

        assert!(actual_screen.is_ok(), "Failed to retrieve screens");
        assert_eq!(vec![expected_screens], actual_screen.unwrap());

        Ok(())
    }

    #[test]
    fn coin_formatting_large_amount_works() -> anyhow::Result<()> {
        let coin = Coin {
            denom: "ATOM".try_into()?,
            amount: Uint256::from(10_000u16).into(),
        };

        let expected_screens = Screen {
            title: "".to_string(),
            content: Content::new("10'000 ATOM".to_string())?,
            indent: None,
            expert: false,
        };

        let actual_screen = ValueRenderer::format(&coin, &get_metadata);

        assert!(actual_screen.is_ok(), "Failed to retrieve screens");
        assert_eq!(vec![expected_screens], actual_screen.unwrap());

        Ok(())
    }
}
