#[cfg(all(feature = "sled", feature = "rocksdb"))]
fn compile_check() {
    compile_error!("Can't use `sled` and `rocksdb` at one time. Chose only one DB")
}

pub mod error;
mod memory;
pub mod prefix;
#[cfg(feature = "rocksdb")]
pub mod rocks;
#[cfg(feature = "sled")]
pub mod sled;

use std::fmt::Debug;

pub use memory::*;

/// Default builder which implements(if enable) builds for all db's
#[derive(Debug, Clone, Default)]
pub struct DBBuilder;

pub trait Database: Clone + Send + Sync + 'static {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;

    fn put(&self, key: Vec<u8>, value: Vec<u8>);

    fn iterator<'a>(&'a self) -> impl Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a;

    fn prefix_iterator<'a>(
        &'a self,
        prefix: Vec<u8>,
    ) -> impl Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a;
}

pub trait DatabaseBuilder<DB> {
    type Err: Debug;

    fn build<P: AsRef<std::path::Path>>(self, path: P) -> Result<DB, Self::Err>;
}
