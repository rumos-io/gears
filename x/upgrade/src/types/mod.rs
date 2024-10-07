pub mod plan;
pub mod query;

#[derive(Debug, Clone)]
pub struct Upgrade {
    pub name: String,
    pub block: u32,
}

impl Upgrade {
    pub fn try_new(block_bytes: impl AsRef<[u8]>, name_bytes: impl AsRef<[u8]>) -> Option<Self> {
        let block = u32::from_be_bytes(block_bytes.as_ref().try_into().ok()?);
        let name = String::from_utf8(name_bytes.as_ref()[1..].to_vec()).ok()?;

        Some(Self { name, block })
    }
}

#[derive(Debug, Clone, gears::derive::Protobuf, serde::Serialize, serde::Deserialize)]
#[proto(raw = "ibc_proto::cosmos::upgrade::v1beta1::ModuleVersion")]
pub struct ModuleVersion {
    pub name: String,
    pub version: u64,
}
