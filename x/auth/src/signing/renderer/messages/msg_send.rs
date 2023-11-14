use std::error::Error;

use gears::types::context::context::Context;
use ibc_proto::cosmos::bank::v1beta1::MsgSend;
use proto_messages::cosmos::tx::v1beta1::screen::{Content, Indent, Screen};
use store::StoreKey;

use crate::signing::renderer::value_renderer::ValueRenderer;

impl<DefaultValueRenderer, SK: StoreKey> ValueRenderer<DefaultValueRenderer, SK> for MsgSend {
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
    fn format(
        &self,
        ctx: &Context<'_, '_, database::RocksDB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn Error>> {
        let mut screens_vec = Vec::with_capacity(match self.amount.is_empty() {
            true => 2,
            false => self.amount.len() + 2,
        });

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

        for coin in &self.amount {
            screens_vec.append(&mut ValueRenderer::<DefaultValueRenderer, SK>::format(
                coin, ctx,
            )?)
        }

        Ok(screens_vec)
    }
}

#[cfg(test)]
mod tests {
    // use ibc_proto::cosmos::bank::v1beta1::MsgSend;
    // use proto_messages::cosmos::tx::v1beta1::screen::Screen;

    // use crate::signing::renderer::value_renderer::{MessageDefaultRenderer, MessageValueRenderer};

    // #[test]
    // fn screen_result_no_coins() -> anyhow::Result<()> {
    //     const MESSAGE: &str = r#"{
    //         "from_address": "cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs",
    //         "to_address": "cosmos1ejrf4cur2wy6kfurg9f2jppp2h3afe5h6pkh5t",
    //         "amount": []
    //     }"#;

    //     let msg: MsgSend = serde_json::from_str(MESSAGE)?;

    //     const SCREENS: &str = r#"[
    // 		{ "title": "From address", "content": "cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs", "indent": 2 },
    // 		{ "title": "To address", "content": "cosmos1ejrf4cur2wy6kfurg9f2jppp2h3afe5h6pkh5t", "indent": 2 }
    // 	]"#;

    //     let expected_screens: Vec<Screen> = serde_json::from_str(SCREENS)?;

    //     let actual_screens = MessageValueRenderer::<MessageDefaultRenderer>::format(&msg);

    //     assert!(actual_screens.is_ok(), "Failed to retrieve screens");
    //     assert_eq!(expected_screens, actual_screens.expect("Unreachable"));

    //     Ok(())
    // }

    // #[test]
    // fn screen_result_with_coin() -> anyhow::Result<()> {
    //     const MESSAGE: &str = r#"{
    //         "from_address": "cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs",
    //         "to_address": "cosmos1ejrf4cur2wy6kfurg9f2jppp2h3afe5h6pkh5t",
    //         "amount": [{ "denom": "uatom", "amount": "10000000" }]
    //     }"#;

    //     let msg: MsgSend = serde_json::from_str(MESSAGE)?;

    //     const SCREENS: &str = r#"[
    // 		{ "title": "From address", "content": "cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs", "indent": 2 },
    // 		{ "title": "To address", "content": "cosmos1ejrf4cur2wy6kfurg9f2jppp2h3afe5h6pkh5t", "indent": 2 },
    //         { "title": "Amount", "content": "10 ATOM", "indent": 2 }
    // 	]"#;

    //     let expected_screens: Vec<Screen> = serde_json::from_str(SCREENS)?;

    //     let actual_screens = MessageValueRenderer::<MessageDefaultRenderer>::format(&msg);

    //     assert!(actual_screens.is_ok(), "Failed to retrieve screens");
    //     assert_eq!(expected_screens, actual_screens.expect("Unreachable"));

    //     Ok(())
    // }
}
