use base64::{
    engine::general_purpose::{self, STANDARD},
    Engine,
};
use base64_serde::base64_serde_type;
use serde::ser::SerializeSeq;

pub fn serialize_number_to_string<T, S>(x: &T, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: std::string::ToString,
{
    s.serialize_str(&x.to_string())
}

base64_serde_type!(pub Base64Standard, STANDARD);

pub fn serialize_vec_of_vec_to_vec_of_base64<S>(x: &Vec<Vec<u8>>, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = s.serialize_seq(Some(x.len()))?;

    for inner in x {
        let b64 = general_purpose::STANDARD.encode(inner);
        seq.serialize_element(&b64)?;
    }

    seq.end()
}
