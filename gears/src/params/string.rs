// TODO:LATER I need to remove pub from inner field

// TODO:NOW What about carring addional info about type inside this string? It would help later to implement xmod wich can change parameters
pub struct ParamString(pub String);

impl ParamString {
    pub fn into_bytes(self) -> Vec<u8> {
        self.0.into_bytes()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl AsRef<[u8]> for ParamString {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl AsRef<str> for ParamString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<ParamString> for Vec<u8> {
    fn from(value: ParamString) -> Self {
        value.into_bytes()
    }
}

impl From<Vec<u8>> for ParamString {
    fn from(value: Vec<u8>) -> Self {
        Self(String::from_utf8(value).expect("ParamString always should have valid string"))
    }
}

impl From<String> for ParamString {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<ParamString> for String {
    fn from(value: ParamString) -> Self {
        value.0
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

impl From<bool> for ParamString {
    fn from(value: bool) -> Self {
        Self(value.to_string())
    }
}

impl From<ParamString> for u8 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<u8> for ParamString {
    fn from(value: u8) -> Self {
        Self(format!("\"{value}\""))
    }
}

impl From<ParamString> for u16 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<u16> for ParamString {
    fn from(value: u16) -> Self {
        Self(format!("\"{value}\""))
    }
}

impl From<ParamString> for u32 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<u32> for ParamString {
    fn from(value: u32) -> Self {
        Self(format!("\"{value}\""))
    }
}

impl From<ParamString> for u64 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<u64> for ParamString {
    fn from(value: u64) -> Self {
        Self(format!("\"{value}\""))
    }
}

impl From<ParamString> for u128 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<u128> for ParamString {
    fn from(value: u128) -> Self {
        Self(format!("\"{value}\""))
    }
}

impl From<ParamString> for i8 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<i8> for ParamString {
    fn from(value: i8) -> Self {
        Self(format!("\"{value}\""))
    }
}

impl From<ParamString> for i16 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<i16> for ParamString {
    fn from(value: i16) -> Self {
        Self(format!("\"{value}\""))
    }
}

impl From<ParamString> for i32 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<i32> for ParamString {
    fn from(value: i32) -> Self {
        Self(format!("\"{value}\""))
    }
}

impl From<ParamString> for i64 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<i64> for ParamString {
    fn from(value: i64) -> Self {
        Self(format!("\"{value}\""))
    }
}

impl From<ParamString> for i128 {
    fn from(value: ParamString) -> Self {
        value.0.parse().expect("should be valid")
    }
}

impl From<i128> for ParamString {
    fn from(value: i128) -> Self {
        Self(format!("\"{value}\""))
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
