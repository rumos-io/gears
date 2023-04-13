pub fn serialize_number_to_string<T, S>(x: &T, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: std::string::ToString,
{
    s.serialize_str(&x.to_string())
}
