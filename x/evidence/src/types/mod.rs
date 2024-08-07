use gears::{
    context::TransactionalContext,
    core::any::google::Any,
    store::{database::Database, StoreKey},
    tendermint::informal::Hash,
};
use prost::Message;

mod query;
mod tx;

//

pub trait Evidence: Message + TryFrom<Any>
where
    <Self as std::convert::TryFrom<Any>>::Error: std::fmt::Debug,
{
    // TODO: uncomment or remove
    // fn route(&self) -> String;
    // Original method is named `type`, replaced as inner interface
    fn kind(&self) -> String;
    fn string(&self) -> String;
    fn hash(&self) -> Hash;
    fn validate_basic(&self) -> anyhow::Result<()>;
    /// Height at which the infraction occurred
    fn height(&self) -> i64;
}

pub trait EvidenceHandler<E: Evidence>
where
    <E as std::convert::TryFrom<Any>>::Error: std::fmt::Debug,
{
    type Error;

    fn handle<CTX: TransactionalContext<DB, SK>, DB: Database, SK: StoreKey>(
        &self,
        ctx: &mut CTX,
        evidence: &E,
    ) -> Result<(), <Self as EvidenceHandler<E>>::Error>;
}
