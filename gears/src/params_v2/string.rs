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

impl From<ParamString> for bool {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<ParamString> for u8 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<ParamString> for u16 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<ParamString> for u32 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<ParamString> for u64 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<ParamString> for u128 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<ParamString> for i8 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<ParamString> for i16 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<ParamString> for i32 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<ParamString> for i64 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<ParamString> for i128 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<ParamString> for usize {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<ParamString> for isize {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<ParamString> for String {
    fn from(value: ParamString) -> Self {
        value.0
    }
}

// impl<T> From<ParamString> for Vec<T>
// {
//     fn from(value: ParamString) -> Self {
//         let vec = serde_json::from_str(&value.0).expect("should be valid");

//     }
// }
