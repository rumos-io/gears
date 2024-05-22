use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use self::string::ParamString;

pub mod keeper;
pub mod space;
pub mod space_mut;
pub mod string;

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
fn parse_param_bytes(value: Vec<u8>) -> ParamString {
    let value = String::from_utf8(value).expect("should be valid utf-8");

    // Some types like `bool` gets saved without
    if let Some(cleared) = value
        .strip_suffix('\"')
        .and_then(|this| this.strip_prefix('\"'))
    {
        cleared.into()
    } else {
        value.into()
    }
}

pub fn parse_primitive<T: From<ParamString>>(value: Vec<u8>) -> T {
    parse_param_bytes(value).into()
}

pub fn parse_primitive_optional<T: From<ParamString>>(value: Option<Vec<u8>>) -> T {
    parse_param_bytes(value.expect("Params expected to exists")).into()
}
