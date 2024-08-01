#[derive(protobuf_derive::Raw)]
struct Simple {
    #[raw(kind(uint32), raw = u32)]
    simple: u32,
}

impl From<Simple> for RawSimple {
    fn from(_: Simple) -> Self {
        Self { simple: 1 }
    }
}

fn main() {}
