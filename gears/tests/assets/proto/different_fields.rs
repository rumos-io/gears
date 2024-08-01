#[derive(protobuf_derive::Protobuf, Clone)]
struct DifferentFields {
    #[proto(name = "other_simple")]
    simple: u32,
}

#[derive(prost::Message)]
struct RawDifferentFields {
    #[prost(uint32)]
    other_simple: u32,
}

fn main() {}
