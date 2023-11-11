//! Trait for formatting primitive types

use std::error::Error;

use database::RocksDB;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::screen::Screen;
use store::StoreKey;

use crate::signing::errors::SigningErrors;

pub trait PrimitiveValueRenderer<V> {
    fn format(value: V) -> String;
}

pub trait MessageValueRenderer<MR> {
    fn format(&self) -> Result<Vec<Screen>, Box<dyn Error>>;
}

pub trait ValueRenderer<SK: StoreKey> {
    /// Format renders the Protobuf value to a list of Screens.
    fn format<R>(&self, ctx: &Context<'_, '_, RocksDB, SK>) -> Result<Vec<Screen>, SigningErrors>;
}

/// Static structure which implement trait for formating primitive types
/// like `i64` or `bool` and made for using in `gears`
pub struct PrimitiveDefaultRenderer;

/// Static structure which implement trait for formating messages
/// like `MsgSend` with proto name `/cosmos.bank.v1beta1.MsgSend` and made for using in `gears`
pub struct MessageDefaultRenderer;
