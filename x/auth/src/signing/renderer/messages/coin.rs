use ibc_proto::cosmos::base::v1beta1::Coin;
use proto_messages::cosmos::tx::v1beta1::screen::Screen;

use crate::signing::renderer::value_renderer::MessageValueRenderer;

impl<MessageDefaultRenderer> MessageValueRenderer<MessageDefaultRenderer> for Coin {
    fn format(&self) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        unimplemented!()
    }
}
