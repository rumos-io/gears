use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    marker::PhantomData,
};

use gears::{
    context::{InfallibleContext, InfallibleContextMut},
    core::Protobuf,
    extensions::corruption::UnwrapCorrupt,
    store::{database::Database, StoreKey},
};
use prost::bytes::Bytes;

use crate::{
    handler::UpgradeHandler,
    types::{plan::Plan, Upgrade},
    Module,
};

pub use downgrade_flag::*;

/// specifies the Byte under which a pending upgrade plan is stored in the store
const PLAN_PREFIX: [u8; 1] = [0x0];
/// is a prefix for to look up completed upgrade plan by name
const DONE_PREFIX: [u8; 1] = [0x1];
/// is a prefix to look up module names (key) and versions (value)
const VERSION_MAP_PREFIX: [u8; 1] = [0x2];
/// is a prefix to look up Protocol Version
const PROTOCOL_VERSION_BYTE_PREFIX: [u8; 1] = [0x3];

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
    upgrade_handlers: HashMap<&'static str, UH>,
    skip_heights: HashSet<u32>, // TODO: source https://github.com/cosmos/gaia/blob/189b57be735d64d0dbf0945717b49017a1beb11e/cmd/gaiad/cmd/root.go#L192-L195
    _modules_marker: PhantomData<M>,
}

impl<SK, M, UH: strum::IntoEnumIterator + UpgradeHandler> UpgradeKeeper<SK, M, UH> {
    pub fn new(store_key: SK, skip_heights: impl IntoIterator<Item = u32>) -> Self {
        Self {
            store_key,
            upgrade_handlers: UH::iter().map(|this| (this.name(), this)).collect(),
            skip_heights: skip_heights.into_iter().collect(),
            _modules_marker: PhantomData,
        }
    }
}

impl<SK, M, UH: UpgradeHandler> UpgradeKeeper<SK, M, UH> {
    pub fn new_unverified(
        store_key: SK,
        upgrade_handlers: impl IntoIterator<Item = (&'static str, UH)>,
        skip_heights: impl IntoIterator<Item = u32>,
    ) -> Self {
        Self {
            store_key,
            upgrade_handlers: upgrade_handlers.into_iter().collect(),
            skip_heights: skip_heights.into_iter().collect(),
            _modules_marker: PhantomData,
        }
    }
}

impl<SK: StoreKey, M: Module, UH: UpgradeHandler> UpgradeKeeper<SK, M, UH>
where
    <M as TryFrom<Vec<u8>>>::Error: Display + Debug,
{
    pub fn apply_upgrade<DB: Database, CTX: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut CTX,
        plan: Plan,
    ) -> anyhow::Result<()> {
        let handler = self
            .upgrade_handlers
            .get(plan.name.as_str())
            .ok_or(anyhow::anyhow!(
                "Upgrade should never be called without first checking HasHandler"
            ))?;

        let versions = self.modules_version(ctx);

        let updated = handler.handle(ctx, &plan, versions)?;

        self.set_modules_version(ctx, updated);
        self.set_protocol_version(ctx, self.protocol_version(ctx) + 1);

        // TODO: protocol setter for baseapp https://github.com/cosmos/cosmos-sdk/blob/d3f09c222243bb3da3464969f0366330dcb977a8/x/upgrade/keeper/keeper.go#L350-L353

        self.clear_ibc_state(ctx, plan.height);
        self.delete_upgrade_plan(ctx);
        self.set_done(ctx, plan);

        Ok(())
    }

    pub fn done_height<DB: Database, CTX: InfallibleContext<DB, SK>>(
        &self,
        ctx: &CTX,
        name: impl AsRef<str>,
    ) -> Option<u32> {
        let height = ctx
            .infallible_store(&self.store_key)
            .get(name.as_ref().as_bytes())?;
        let height = u32::from_be_bytes(height.try_into().ok().unwrap_or_corrupt());

        Some(height)
    }

    fn set_done<DB: Database, CTX: InfallibleContextMut<DB, SK>>(&self, ctx: &mut CTX, plan: Plan) {
        let height = ctx.height();

        ctx.infallible_store_mut(&self.store_key)
            .prefix_store_mut(DONE_PREFIX)
            .set(plan.name.into_bytes(), height.to_be_bytes());
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

    pub fn modules_version<DB: Database, CTX: InfallibleContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> HashMap<M, u64> {
        ctx.infallible_store(&self.store_key)
            .prefix_store(VERSION_MAP_PREFIX)
            .into_range(..)
            .map(|(key, value)| {
                (
                    M::try_from(key.as_slice().to_vec()).expect("unknown module version saved"),
                    u64::from_be_bytes(value.as_slice().try_into().ok().unwrap_or_corrupt()),
                )
            })
            .collect::<HashMap<_, _>>()
    }

    fn set_modules_version<DB: Database, CTX: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut CTX,
        modules: impl IntoIterator<Item = (M, u64)>,
    ) {
        let modules = modules.into_iter().collect::<HashMap<_, _>>();

        if modules.is_empty() {
            let mut store = ctx
                .infallible_store_mut(&self.store_key)
                .prefix_store_mut(VERSION_MAP_PREFIX);

            for (module, version) in modules {
                store.set(
                    module.name().as_bytes().to_owned(),
                    version.to_be_bytes().to_vec(),
                );
            }
        }
    }

    fn protocol_version<DB: Database, CTX: InfallibleContext<DB, SK>>(&self, ctx: &CTX) -> u64 {
        ctx.infallible_store(&self.store_key)
            .get(&PROTOCOL_VERSION_BYTE_PREFIX)
            .map(|this| u64::from_be_bytes(this.as_slice().try_into().unwrap_or_corrupt()))
            .unwrap_or_default()
    }

    fn set_protocol_version<DB: Database, CTX: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut CTX,
        version: u64,
    ) {
        ctx.infallible_store_mut(&self.store_key)
            .set(PROTOCOL_VERSION_BYTE_PREFIX, version.to_be_bytes());
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
