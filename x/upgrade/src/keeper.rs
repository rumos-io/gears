use gears::{
    context::QueryableContext,
    core::Protobuf,
    extensions::corruption::UnwrapCorrupt,
    store::{database::Database, StoreKey},
    types::store::gas::errors::GasStoreErrors,
    x::module::Module,
};
use prost::bytes::Bytes;

use crate::types::{plan::Plan, Upgrade};

pub use downgrade_flag::*;

const PLAN_KEY: [u8; 1] = [0x0];
const DONE_KEY: [u8; 1] = [0x1];

#[derive(Debug, Clone)]
pub struct UpgradeKeeper<SK, M> {
    store_key: SK,
    _upgrade_mod: M,
}

impl<SK, M> UpgradeKeeper<SK, M> {
    pub fn new() {}
}

impl<SK: StoreKey, M: Module> UpgradeKeeper<SK, M> {
    pub fn upgrade_plan<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<Option<Plan>, GasStoreErrors> {
        let store = ctx.kv_store(&self.store_key);

        Ok(store
            .get(&PLAN_KEY)?
            .map(|this| Protobuf::decode::<Bytes>(this.into()).unwrap_or_corrupt()))
    }

    // pub fn last_completed_upgrade<DB: Database, CTX: QueryableContext<DB, SK>>(
    //     &self,
    //     ctx: &CTX,
    // ) -> Result<Option<Upgrade>, GasStoreErrors> {
    // }
}

mod downgrade_flag {
    use std::sync::atomic::AtomicBool;

    /// tells if we've already sanity checked that this binary version isn't being used against an old state.
    static DOWNGRADE_VERIFIED: AtomicBool = AtomicBool::new(false);

    pub fn downgrade_verified() -> bool {
        DOWNGRADE_VERIFIED.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn set_downgrade_verified(val: bool) -> bool {
        DOWNGRADE_VERIFIED.swap(val, std::sync::atomic::Ordering::SeqCst)
    }
}
