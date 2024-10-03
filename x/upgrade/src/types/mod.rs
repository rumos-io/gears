pub mod plan;
pub mod query;

#[derive(Debug, Clone)]
pub struct Upgrade {
    pub name: String,
    pub block: u32,
}
