use gears::tendermint::informal::Hash;
use prost::Message;

mod query;
mod router;
mod tx;
pub use router::*;

//

pub trait Evidence: Message {
    fn route(&self) -> String;
    fn r#type(&self) -> String;
    fn string(&self) -> String;
    fn hash(&self) -> Hash;
    fn validate_basic(&self) -> anyhow::Result<()>;
    /// Height at which the infraction occurred
    fn height(&self) -> i64;
}
