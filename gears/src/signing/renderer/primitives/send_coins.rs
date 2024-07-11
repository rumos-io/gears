use crate::signing::handler::MetadataGetter;
use crate::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, RenderError, TryPrimitiveValueRendererWithMetadata,
};
use crate::types::{base::coins::UnsignedCoins, rendering::screen::Content};

impl TryPrimitiveValueRendererWithMetadata<UnsignedCoins> for DefaultPrimitiveRenderer {
    fn try_format_with_metadata<MG: MetadataGetter>(
        coins: UnsignedCoins,
        get_metadata: &MG,
    ) -> Result<Content, RenderError> {
        let inner_coins = coins.clone().into_inner();

        let mut formatted_coins = Vec::with_capacity(inner_coins.len());

        for coin in inner_coins.into_iter() {
            let formatted_coin =
                DefaultPrimitiveRenderer::try_format_with_metadata(coin, get_metadata)?
                    .into_inner();
            formatted_coins.push(formatted_coin);
        }

        formatted_coins.sort();
        let formatted_coins = formatted_coins.iter().fold(String::new(), |mut acc, coin| {
            if !acc.is_empty() {
                acc.push_str(", ");
            }
            acc.push_str(coin);
            acc
        });

        Ok(Content::try_new(formatted_coins).expect("send coins are never empty"))
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Uint256;

    use crate::signing::renderer::test_functions::TestMetadataGetter;
    use crate::signing::renderer::value_renderer::{
        DefaultPrimitiveRenderer, TryPrimitiveValueRendererWithMetadata,
    };
    use crate::types::{
        base::{coin::UnsignedCoin, coins::UnsignedCoins},
        rendering::screen::Content,
    };

    #[test]
    fn send_coins_check_format() -> anyhow::Result<()> {
        let coin = UnsignedCoin {
            denom: "uatom".try_into()?,
            amount: Uint256::from(2000u32),
        };

        let expected_content = Content::try_new("0.002 ATOM".to_string()).unwrap();

        let actual_content = DefaultPrimitiveRenderer::try_format_with_metadata(
            UnsignedCoins::new(vec![coin]).unwrap(),
            &TestMetadataGetter,
        );

        assert_eq!(expected_content, actual_content.unwrap());

        Ok(())
    }

    #[test]
    fn send_coins_check_format_multi_denom_alphabetical() -> anyhow::Result<()> {
        let coin1 = UnsignedCoin {
            denom: "uatom".try_into()?,
            amount: Uint256::from(2000u32),
        };

        let coin2 = UnsignedCoin {
            denom: "uon".try_into()?,
            amount: Uint256::from(2000u32),
        };

        let expected_content = Content::try_new("0.002 AAUON, 0.002 ATOM".to_string()).unwrap();

        let actual_content = DefaultPrimitiveRenderer::try_format_with_metadata(
            UnsignedCoins::new(vec![coin1, coin2]).unwrap(),
            &TestMetadataGetter,
        );

        assert_eq!(expected_content, actual_content.unwrap());

        Ok(())
    }

    #[test]
    fn send_coins_check_format_more_sig_figs() -> anyhow::Result<()> {
        let coin = UnsignedCoin {
            denom: "uatom".try_into()?,
            amount: Uint256::from(2047u32),
        };

        let expected_content = Content::try_new("0.002047 ATOM".to_string()).unwrap();

        let actual_content = DefaultPrimitiveRenderer::try_format_with_metadata(
            UnsignedCoins::new(vec![coin]).unwrap(),
            &TestMetadataGetter,
        );

        assert_eq!(expected_content, actual_content.unwrap());

        Ok(())
    }

    #[test]
    fn send_coins_check_format_int_and_dec_part() -> anyhow::Result<()> {
        let coin = UnsignedCoin {
            denom: "uatom".try_into()?,
            amount: Uint256::from(2_123_456u32),
        };

        let expected_content = Content::try_new("2.123456 ATOM".to_string()).unwrap();

        let actual_content = DefaultPrimitiveRenderer::try_format_with_metadata(
            UnsignedCoins::new(vec![coin]).unwrap(),
            &TestMetadataGetter,
        );

        assert_eq!(expected_content, actual_content.unwrap());

        Ok(())
    }
}
