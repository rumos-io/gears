use std::error::Error;

use proto_messages::cosmos::{
    bank::v1beta1::MsgSend,
    base::v1beta1::Coin,
    tx::v1beta1::{
        screen::{Content, Indent, Screen},
        tx_metadata::Metadata,
    },
};
use proto_types::Denom;

use crate::signing::renderer::value_renderer::ValueRenderer;

impl ValueRenderer for MsgSend {
    /// Format `MsgSend` with `MessageDefaultRenderer`
    ///
    /// ## Example
    ///
    /// `MsgSend` structure in json format
    /// ```json
    /// {
    /// "from_address": "cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs",
    /// "to_address": "cosmos1ejrf4cur2wy6kfurg9f2jppp2h3afe5h6pkh5t",
    /// "amount": []
    /// }
    /// ```
    ///
    /// Formatted into
    ///
    /// ```json
    /// [
    /// 	{ "title": "From address", "content": "cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs", "indent": 2 },
    ///     { "title": "To address", "content": "cosmos1ejrf4cur2wy6kfurg9f2jppp2h3afe5h6pkh5t", "indent": 2 }
    /// ]
    /// ```
    ///
    ///
    /// ## Note
    /// This implementation doesn't include `Screen` with information about beginning of message and name
    fn format<F: Fn(&Denom) -> Option<Metadata>>(
        &self,
        get_metadata: &F,
    ) -> Result<Vec<Screen>, Box<dyn Error>> {
        let mut screens_vec = Vec::new();

        screens_vec.push(Screen {
            title: "From address".to_string(),
            content: Content::new(self.from_address.clone())?,
            indent: Some(Indent::new(2)?),
            expert: false,
        });

        screens_vec.push(Screen {
            title: "To address".to_string(),
            content: Content::new(self.to_address.to_string())?,
            indent: Some(Indent::new(2)?),
            expert: false,
        });

        for coin_raw in self.amount.clone() {
            let coin: Coin = coin_raw.try_into()?;

            screens_vec.append(&mut ValueRenderer::format(&coin, get_metadata)?)
        }

        Ok(screens_vec)
    }
}

#[cfg(test)]
mod tests {
    use proto_messages::cosmos::{
        bank::v1beta1::MsgSend, ibc_types::bank::MsgSendRaw, tx::v1beta1::screen::Screen,
    };

    use crate::signing::renderer::{
        value_renderer::ValueRenderer, values::test_functions::get_metadata,
    };

    // TODO: fix this test
    // #[test]
    // fn screen_result_no_coins() -> anyhow::Result<()> {
    //     const MESSAGE: &str = r#"{
    //         "from_address": "cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs",
    //         "to_address": "cosmos1ejrf4cur2wy6kfurg9f2jppp2h3afe5h6pkh5t",
    //         "amount": []
    //     }"#;

    //     let msg: MsgSendRaw = serde_json::from_str(MESSAGE)?;
    //     let msg: MsgSend = msg.try_into()?;

    //     const SCREENS: &str = r#"[
    // 		{ "title": "From address", "content": "cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs", "indent": 2 },
    // 		{ "title": "To address", "content": "cosmos1ejrf4cur2wy6kfurg9f2jppp2h3afe5h6pkh5t", "indent": 2 }
    // 	]"#;

    //     let expected_screens: Vec<Screen> = serde_json::from_str(SCREENS)?;

    //     let actual_screens = ValueRenderer::format(&msg, &get_metadata);

    //     assert!(actual_screens.is_ok(), "Failed to retrieve screens");
    //     assert_eq!(expected_screens, actual_screens.expect("Unreachable"));

    //     Ok(())
    // }

    #[test]
    fn screen_result_with_coin() -> anyhow::Result<()> {
        const MESSAGE: &str = r#"{
            "from_address": "cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs",
            "to_address": "cosmos1ejrf4cur2wy6kfurg9f2jppp2h3afe5h6pkh5t",
            "amount": [{ "denom": "uatom", "amount": "10000000" }]
        }"#;

        let msg: MsgSendRaw = serde_json::from_str(MESSAGE)?;
        let msg: MsgSend = msg.try_into()?;

        const SCREENS: &str = r#"[
    		{ "title": "From address", "content": "cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs", "indent": 2 },
    		{ "title": "To address", "content": "cosmos1ejrf4cur2wy6kfurg9f2jppp2h3afe5h6pkh5t", "indent": 2 },
            { "title": "Amount", "content": "10 ATOM", "indent": 2 }
    	]"#;

        let expected_screens: Vec<Screen> = serde_json::from_str(SCREENS)?;

        let actual_screens = ValueRenderer::format(&msg, &get_metadata);

        assert!(actual_screens.is_ok(), "Failed to retrieve screens");
        assert_eq!(expected_screens, actual_screens.expect("Unreachable"));

        Ok(())
    }
}
