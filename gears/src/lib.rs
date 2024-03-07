#![warn(rust_2018_idioms)]

#[allow(dead_code, unused_variables, unused_imports)] // TODO: remove
mod app;
pub mod baseapp;
#[cfg(feature = "cli")]
pub mod cli;
pub mod client;
pub mod config;
pub mod crypto;
pub mod error;
pub mod types;
pub mod utils;
pub mod x;

pub use app::*;
