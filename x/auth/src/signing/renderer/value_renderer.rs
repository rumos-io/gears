//! Trait for formatting all kind of values into `Screen`

use proto_messages::cosmos::tx::v1beta1::{screen::Screen, tx_metadata::Metadata};
use proto_types::Denom;
use std::error::Error;

/// Render primitive type into content for `Screen`.
/// Use for formatting simple primitive `Copy` types that doesn't require error handling
pub trait PrimitiveValueRenderer<V> {
    /// Get string representation of some `V`
    fn format(value: V) -> String;

    /// Try format specific value
    fn format_try(value: V) -> Result<String, Box<dyn Error>>;
}

pub trait ValueRenderer {
    /// Format renders the Protobuf value to a list of Screens.
    fn format<F: Fn(&Denom) -> Option<Metadata>>(
        &self,
        get_metadata: &F,
    ) -> Result<Vec<Screen>, Box<dyn Error>>;
}

pub struct DefaultPrimitiveRenderer;
