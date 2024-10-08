pub mod abci_handler;
pub mod client;
pub mod errors;
pub mod genesis;
pub mod keeper;
pub mod msg;
pub mod params;
pub mod query;
pub mod submission;
pub mod types;

// pub trait ProposalHandler<P> {
//     fn handle<CTX: InfallibleContextMut<DB, SK>, DB: Database, SK: StoreKey>(
//         &self,
//         proposal: &P,
//         ctx: &mut CTX,
//     ) -> Result<(), SubmissionHandlingError>;

//     fn check(proposal: &P) -> bool;
// }
