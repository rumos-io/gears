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

pub trait Evidence: Message + TryFrom<Any> {
    type Error;
    // TODO: uncomment or remove
    // fn route(&self) -> String;
    // Original method is named `type`, replaced as inner interface
    fn kind(&self) -> String;
    fn string(&self) -> String;
    fn hash(&self) -> Hash;
    /// Height at which the infraction occurred
    fn height(&self) -> i64;
    fn handle<CTX: TransactionalContext<DB, SK>, DB: Database, SK: StoreKey>(
        ctx: &mut CTX,
        evidence: &Self,
    ) -> Result<(), <Self as Evidence>::Error>;
}
