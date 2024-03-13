use proto_messages::cosmos::tx::v1beta1::{
    mode_info::ModeInfo, screen::Screen, tx_metadata::Metadata,
};
use proto_types::Denom;

use crate::signing::renderer::value_renderer::{Error, ValueRenderer};

impl ValueRenderer for ModeInfo {
    fn format<F: Fn(&Denom) -> Option<Metadata>>(
        &self,
        _get_metadata: &F,
    ) -> Result<Vec<Screen>, Error> {
        // I don't see that mode ino is used in screen formatting for now, but leave this as things may change
        Ok(Vec::new())
    }
}
