use std::collections::{HashMap, HashSet};

use gears::{
    context::{InfallibleContext, InfallibleContextMut},
    core::Protobuf,
    extensions::corruption::UnwrapCorrupt,
    store::{database::Database, StoreKey},
    x::module::Module,
};
use prost::bytes::Bytes;

use crate::{
    handler::UpgradeHandler,
    types::{plan::Plan, Upgrade},
};

pub use downgrade_flag::*;

/// specifies the Byte under which a pending upgrade plan is stored in the store
const PLAN_PREFIX: [u8; 1] = [0x0];
/// is a prefix for to look up completed upgrade plan by name
const DONE_PREFIX: [u8; 1] = [0x1];
/// is a prefix to look up module names (key) and versions (value)
const _VERSION_MAP_PREFIX: [u8; 1] = [0x2];
/// is a prefix to look up Protocol Version
const _PROTOCOL_VERSION_BYTE_PREFIX: [u8; 1] = [0x3];

/// is the key under which upgraded ibc state is stored in the upgrade store
const UPGRADED_IBC_STATE_KEY: &[u8] = "upgradedIBCState".as_bytes();
/// is the sub-key under which upgraded client state will be stored
const UPGRADED_CLIENT_KEY: &[u8] = "upgradedClient".as_bytes();
/// is the sub-key under which upgraded consensus state will be stored
const UPGRADED_CONS_STATE_KEY: &[u8] = "upgradedConsState".as_bytes();

fn upgraded_client_key(height: u32) -> Vec<u8> {
    [
        UPGRADED_IBC_STATE_KEY,
        height.to_be_bytes().as_slice(), // TODO: Unsure in this
        UPGRADED_CLIENT_KEY,
    ]
    .concat()
}

fn upgraded_const_state_key(height: u32) -> Vec<u8> {
    [
        UPGRADED_IBC_STATE_KEY,
        height.to_be_bytes().as_slice(), // TODO: Unsure in this
        UPGRADED_CONS_STATE_KEY,
    ]
    .concat()
}

#[derive(Debug, Clone)]
pub struct UpgradeKeeper<SK, M, UH> {
    store_key: SK,
    upgrade_handlers: HashMap<String, UH>,
    skip_heights: HashSet<u32>, // TODO: source https://github.com/cosmos/gaia/blob/189b57be735d64d0dbf0945717b49017a1beb11e/cmd/gaiad/cmd/root.go#L192-L195
    _upgrade_mod: M,
}

impl<SK, M, UH> UpgradeKeeper<SK, M, UH> {
    pub fn new() {}
}

impl<SK: StoreKey, M: Module, UH: UpgradeHandler> UpgradeKeeper<SK, M, UH> {
    pub fn apply_upgrade<DB: Database, CTX: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut CTX,
        plan: &Plan,
    ) -> anyhow::Result<()> {
        let handler = self
            .upgrade_handlers
            .get(&plan.name)
            .ok_or(anyhow::anyhow!(
                "Upgrade should never be called without first checking HasHandler"
            ))?;

        // let updated = handler.handle(ctx, plan, [()])?;

        Ok(())
    }

    pub fn upgrade_plan<DB: Database, CTX: InfallibleContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Option<Plan> {
        let store = ctx.infallible_store(&self.store_key);

        store
            .get(&PLAN_PREFIX)
            .map(|this| Protobuf::decode::<Bytes>(this.into()).unwrap_or_corrupt())
    }

    pub fn delete_upgrade_plan<DB: Database, CTX: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut CTX,
    ) -> bool {
        let old_plan = self.upgrade_plan(ctx);
        if let Some(old_plan) = old_plan {
            self.clear_ibc_state(ctx, old_plan.height);
        }

        ctx.infallible_store_mut(&self.store_key)
            .delete(&PLAN_PREFIX)
            .is_some()
    }

    fn clear_ibc_state<DB: Database, CTX: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut CTX,
        last_height: u32,
    ) {
        let mut store = ctx.infallible_store_mut(&self.store_key);
        store.delete(&upgraded_client_key(last_height));
        store.delete(&upgraded_const_state_key(last_height));
    }

    pub fn last_completed_upgrade<DB: Database, CTX: InfallibleContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Option<Upgrade> {
        // TODO: When revertable iterator will be available use it
        let upgrade_bytes = ctx
            .infallible_store(&self.store_key)
            .prefix_store(DONE_PREFIX)
            .into_range(..);

        let mut found = false;
        let mut last_upgrade = Option::None;
        for (key, value) in upgrade_bytes {
            let upgrade = Upgrade::try_new(key.as_slice(), value.as_slice()).unwrap_or_corrupt();

            if !found
                || upgrade.block
                    >= last_upgrade
                        .as_ref()
                        .map(|this: &Upgrade| this.block)
                        .unwrap_or_default()
            {
                found = true;
                last_upgrade = Some(upgrade)
            }
        }

        last_upgrade
    }

    pub fn is_skip_height(&self, height: u32) -> bool {
        self.skip_heights.contains(&height)
    }

    pub fn has_handler(&self, name: impl AsRef<str>) -> bool {
        self.upgrade_handlers.contains_key(name.as_ref())
    }
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
