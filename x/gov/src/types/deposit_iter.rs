// use gears::{store::database::Database, types::store::{kv::Store, range::StoreRange}};

// use crate::msg::deposit::MsgDeposit;

// #[derive(Debug)]
// pub struct DepositIterator<'a, DB>(StoreRange<'a, DB>);

// impl<'a, DB: Database> DepositIterator<'a, DB>
// {
//     pub fn new( store: Store<'a, DB>,) -> DepositIterator<'a, DB>
//     {
//       let prefix = store.prefix_store(MsgDeposit::KEY_PREFIX);

//       let range = prefix.range(..); // omit details

//       DepositIterator(range)
//     }
// }
