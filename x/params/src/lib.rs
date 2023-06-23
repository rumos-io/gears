mod keeper;

pub use keeper::*;

use std::marker::PhantomData;

use database::Database;
use gears::types::context_v2::Context;
use store::StoreKey;

pub struct SubSpace<SK: StoreKey, PK, P> {
    params_keeper: SK,
    subspace_key: PK,
    s: PhantomData<SK>,
    p: PhantomData<P>,
}

// impl<SK: StoreKey, PK, P> SubSpace<SK, PK, P> {
//     fn get<DB: Database>(&self, ctx: &Context<DB, SK>) -> P {
//         let params_store = ctx.get_mutable_kv_store(&self.params_store_key);

//         let subspace_store = params_store.get_immutable_prefix_store(vec![]); // TODO: need to use subspace key here
//     }
// }
