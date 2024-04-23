pub trait MeterDescriptor {
    fn name() -> &'static str;
}

#[derive(Debug, Clone)]
pub struct BlockDescriptor;

#[derive(Debug, Clone)]
pub struct TxSizeDescriptor;

#[derive(Debug, Clone)]
pub struct AnteSecp256k1Descriptor;

impl MeterDescriptor for BlockDescriptor {
    fn name() -> &'static str {
        "block gas meter"
    }
}

impl MeterDescriptor for AnteSecp256k1Descriptor {
    fn name() -> &'static str {
        "ante verify: secp256k1"
    }
}

impl MeterDescriptor for TxSizeDescriptor {
    fn name() -> &'static str {
        "txSize"
    }
}
