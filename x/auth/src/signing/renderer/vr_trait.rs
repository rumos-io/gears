use database::RocksDB;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::{message::Message, screen::Screen};
use store::StoreKey;

use crate::signing::errors::SigningErrors;

/// ValueRenderer is an interface to produce formatted output for all
/// protobuf types as well as parse a string into those protobuf types.
///
/// The notion of "value renderer" is defined in ADR-050, and that ADR provides
/// a default spec for value renderers. However, we define it as an interface
/// here, so that optionally more value renderers could be built, for example, a
/// separate one for a different language.
pub trait ValueRendererTrait<V, SK: StoreKey, M: Message> {
    /// Format renders the Protobuf value to a list of Screens.
    fn format(ctx: &Context<'_, '_, RocksDB, SK>, value: V) -> Result<Vec<Screen>, SigningErrors>;

    /// Parse is the inverse of Format. It must be able to parse all valid
    /// screens, meaning only those generated using this renderer's Format method.
    /// However the behavior of Parse on invalid screens is not specified,
    /// and does not necessarily error.
    fn parse(
        ctx: &Context<'_, '_, RocksDB, SK>,
        screens: impl IntoIterator<Item = Screen>,
    ) -> Result<V, SigningErrors>;
}
