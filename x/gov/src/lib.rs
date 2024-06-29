pub mod client;
use gears::{
    context::InfallibleContextMut,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
};

pub mod abci_handler;
pub mod errors;
pub mod genesis;
pub mod keeper;
pub mod msg;
pub mod params;
pub mod query;
pub mod submission;
pub mod types;

pub trait ProposalHandler<PSK: ParamsSubspaceKey, P> {
    fn handle<CTX: InfallibleContextMut<DB, SK>, DB: Database, SK: StoreKey>(
        &self,
        proposal: &P,
        ctx: &mut CTX,
    ) -> anyhow::Result<()>;

    fn check(proposal: &P) -> bool;
}
