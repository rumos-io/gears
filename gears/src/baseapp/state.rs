// use database::Database;
// use store_crate::{StoreKey};
//
// #[derive(Debug, Clone)]
// pub struct State< 'a, 'b, T: Database, SK : StoreKey >
// {
//   ms : Arc<RwLock<MultiStore<T, SK>>>,
//   ctx : Context<'a , 'b, T, SK >,
// }
//
#[derive(Debug, Clone, PartialEq)]
pub enum StateEnum /*< T: Database, SK : StoreKey>*/ {
    None,
    CheckState(()),
    PrepareProposalState(()),
    ProcessProposalState(()),
    FinalizeBlockState(()),
}
