//! Trait for formatting all kind of values into `Screen`

use proto_messages::cosmos::tx::v1beta1::{
    screen::{Content, Screen},
    tx_metadata::Metadata,
};
use proto_types::Denom;

/// Render primitive type into content for `Screen`.
pub trait PrimitiveValueRenderer<V> {
    /// Get string representation of some `V` wrapped in a Content
    fn format(value: V) -> Content;
}

pub trait TryPrimitiveValueRenderer<V> {
    /// Try to get a string representation of some `V` wrapped in a Content
    fn try_format(value: V) -> Result<Content, Error>;
}

pub trait TryPrimitiveValueRendererWithMetadata<V> {
    /// Try to get a string representation of some `V` wrapped in a Content. This method also
    /// takes a function to get metadata for the denom.
    fn try_format_with_metadata<F: Fn(&Denom) -> Option<Metadata>>(
        value: V,
        get_metadata: &F,
    ) -> Result<Content, Error>;
}

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("not implemented")]
    NotImplemented,
    #[error("`{0}`")]
    Rendering(String),
}

pub trait ValueRenderer {
    /// Format renders the Protobuf value to a list of Screens.
    fn format<F: Fn(&Denom) -> Option<Metadata>>(
        &self,
        get_metadata: &F,
    ) -> Result<Vec<Screen>, Error>;
}

/// Default implementation of `PrimitiveValueRenderer` for `Screen`. This is an attempt
/// at a blanket implementation for all primitive types described in the Cosmos SDK:
/// https://docs.cosmos.network/v0.50/build/architecture/adr-050-sign-mode-textual-annex1#bytes
pub struct DefaultPrimitiveRenderer;
