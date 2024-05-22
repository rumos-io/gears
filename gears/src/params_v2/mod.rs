use std::hash::Hash;

pub mod errors;
pub mod keeper;
pub mod space;
pub mod space_mut;

pub trait ParamsSubspaceKeyV2: Hash + Eq + Clone + Send + Sync + 'static {
    fn name(&self) -> &'static str; // TODO:NOW Cow<'static>?
}

pub trait ParamsPath {
    fn key(&self) -> &'static str;
}
