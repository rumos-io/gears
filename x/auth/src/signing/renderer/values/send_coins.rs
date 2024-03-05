use proto_messages::cosmos::{
    base::v1beta1::SendCoins,
    tx::v1beta1::{
        screen::{Content, Screen},
        tx_metadata::Metadata,
    },
};
use proto_types::Denom;

use crate::signing::renderer::value_renderer::ValueRenderer;
impl ValueRenderer for SendCoins {
    fn format<F: Fn(&Denom) -> Option<Metadata>>(
        &self,
        get_metadata: &F,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let inner_coins = self.clone().into_inner();

        let mut formatted_coins = Vec::with_capacity(inner_coins.len());

        for coin in inner_coins.into_iter() {
            let formatted_coin = ValueRenderer::format(&coin, get_metadata)?
                .get(0)
                .expect("this vec always contains exactly one element")
                .content
                .clone()
                .into_inner();

            formatted_coins.push(formatted_coin);
        }

        formatted_coins.sort();
        let formatted_coins = formatted_coins.iter().fold(String::new(), |mut acc, coin| {
            if !acc.is_empty() {
                acc.push_str(", ");
            }
            acc.push_str(&coin);
            acc
        });

        Ok(vec![Screen {
            title: "".to_string(),
            content: Content::new(formatted_coins).expect("send coins are never empty"),
            indent: None,
            expert: false,
        }])
    }
}

#[cfg(test)]
mod tests {
    use proto_messages::cosmos::{
        base::v1beta1::{Coin, SendCoins},
        tx::v1beta1::screen::{Content, Screen},
    };
    use proto_types::Uint256;

    use crate::signing::renderer::{
        value_renderer::ValueRenderer, values::test_functions::get_metadata,
    };

    #[test]
    fn send_coins_check_format() -> anyhow::Result<()> {
        let coin = Coin {
            denom: "uatom".try_into()?,
            amount: Uint256::from(2000u32),
        };

        let expected_screens = Screen {
            title: "".to_string(),
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
    fn send_coins_check_format_multi_denom_alphabetical() -> anyhow::Result<()> {
        let coin1 = Coin {
            denom: "uatom".try_into()?,
            amount: Uint256::from(2000u32).into(),
        };

        let coin2 = Coin {
            denom: "uon".try_into()?,
            amount: Uint256::from(2000u32).into(),
        };

        let expected_screens = Screen {
            title: "".to_string(),
            content: Content::new("0.002 AAUON, 0.002 ATOM".to_string())?,
            indent: None,
            expert: false,
        };

        let actual_screen =
            ValueRenderer::format(&SendCoins::new(vec![coin1, coin2])?, &get_metadata)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(vec![expected_screens], actual_screen);

        Ok(())
    }

    #[test]
    fn send_coins_check_format_more_sig_figs() -> anyhow::Result<()> {
        let coin = Coin {
            denom: "uatom".try_into()?,
            amount: Uint256::from(2047u32).into(),
        };

        let expected_screens = Screen {
            title: "".to_string(),
            content: Content::new("0.002047 ATOM".to_string())?,
            indent: None,
            expert: false,
        };

        let actual_screen = ValueRenderer::format(&SendCoins::new(vec![coin])?, &get_metadata)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(vec![expected_screens], actual_screen);

        Ok(())
    }

    #[test]
    fn send_coins_check_format_int_and_dec_part() -> anyhow::Result<()> {
        let coin = Coin {
            denom: "uatom".try_into()?,
            amount: Uint256::from(2_123_456u32).into(),
        };

        let expected_screens = Screen {
            title: "".to_string(),
            content: Content::new("2.123456 ATOM".to_string())?,
            indent: None,
            expert: false,
        };

        let actual_screen = ValueRenderer::format(&SendCoins::new(vec![coin])?, &get_metadata)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(vec![expected_screens], actual_screen);

        Ok(())
    }
}
