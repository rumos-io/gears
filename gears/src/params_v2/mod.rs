use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

pub mod keeper;
pub mod space;
pub mod space_mut;

pub trait ParamsSubspaceKey: Hash + Eq + Clone + Send + Sync + 'static {
    fn name(&self) -> &'static str; // TODO:NOW Cow<'static>?
}

pub trait ModuleParams {
    fn module_params<PSK: ParamsSubspaceKey, P: Params>() -> (PSK, P);
}

pub trait Params {
    /// Return all unique keys for this structure
    fn keys() -> HashSet<&'static str>;
    fn serialize(&self) -> HashMap<&'static str, Vec<u8>>; // TODO:NOW CHANGE NAME
}

pub trait ParamsDeserialize: Params {
    fn deserialize(fields: HashMap<&'static str, Vec<u8>>) -> Self;
}

/// Parse params bytes into valid `String` which must we able to parse into param ***field***
pub fn parse_param_bytes(value: Vec<u8>) -> ParamString {
    String::from_utf8(value)
        .expect("should be valid utf-8")
        .strip_suffix('\"')
        .expect("should have suffix")
        .strip_prefix('\"')
        .expect("should have prefix")
        .to_owned()
        .into()
}

pub struct ParamString(pub String);

impl From<String> for ParamString {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for ParamString {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
