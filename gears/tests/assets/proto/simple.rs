#[derive(protobuf_derive::Protobuf, Clone)]
struct Simple {
    simple: u32,
}

#[derive(prost::Message)]
struct RawSimple {
    #[prost(uint32)]
    simple: u32,
}

fn main() {}
