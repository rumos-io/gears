//! Trait for formatting primitive types

use database::RocksDB;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::screen::Screen;
use store::StoreKey;

use crate::signing::errors::SigningErrors;

pub trait PrimitiveValueRenderer<V> {
    fn format(value: V) -> String;
}

pub trait ValueRenderer<SK: StoreKey> {
    /// Format renders the Protobuf value to a list of Screens.
    fn format<R>(&self, ctx: &Context<'_, '_, RocksDB, SK>) -> Result<Vec<Screen>, SigningErrors>;
}

pub struct DefaultRenderer;
