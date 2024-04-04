//! Serialize and deserialize any `T` that implements [`FromStr`]
//! and [`Display`] to convert from or into string. Note this can be used for
//! all primitive data types.

use core::fmt::Display;
use core::str::FromStr;
use std::borrow::Cow;

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};


/// Deserialize string into T
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    <Cow<'_, str>>::deserialize(deserializer)?
        .parse::<T>()
        .map_err(D::Error::custom)
}

/// Serialize from T into string
pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Display,
{
    value.to_string().serialize(serializer)
}
