use crate::{
    types::{Evidence, Handler, Router},
    GenesisState, RouterAlreadyExistsError,
};
use gears::{
    context::{init::InitContext, QueryableContext, TransactionalContext},
    store::{
        database::{ext::UnwrapCorrupt, Database},
        StoreKey,
    },
    tendermint::informal::Hash,
    types::store::gas::{errors::GasStoreErrors, ext::GasResultExt},
    x::{
        keepers::{slashing::EvidenceSlashingKeeper, staking::SlashingStakingKeeper},
        module::Module,
    },
};
use std::{collections::HashMap, marker::PhantomData};

const KEY_PREFIX_EVIDENCE: [u8; 1] = [0x0];

/// Keeper of the evidence store
#[derive(Debug, Clone)]
pub struct Keeper<
    'a,
    SK: StoreKey,
    StkK: SlashingStakingKeeper<SK, M>,
    SlsK: EvidenceSlashingKeeper<SK, M>,
    DB: Database,
    E: Evidence,
    M: Module,
> {
    store_key: SK,
    // TODO
    #[allow(dead_code)]
    staking_keeper: StkK,
    #[allow(dead_code)]
    slashing_keeper: SlsK,
    router: Option<Router<'a, DB, SK, E>>,
    _module: PhantomData<M>,
}

impl<
        'a,
        SK: StoreKey,
        StkK: SlashingStakingKeeper<SK, M>,
        SlsK: EvidenceSlashingKeeper<SK, M>,
        DB: Database,
        E: Evidence + Default,
        M: Module,
    > Keeper<'a, SK, StkK, SlsK, DB, E, M>
{
    pub fn new(
        store_key: SK,
        staking_keeper: StkK,
        slashing_keeper: SlsK,
        routes: Option<HashMap<String, Handler<'a, DB, SK, E>>>,
    ) -> Self {
        Self {
            store_key,
            staking_keeper,
            slashing_keeper,
            router: routes.map(|routes| Router::new(routes)),
            _module: PhantomData,
        }
    }

    /// set_router sets the Evidence Handler router for the x/evidence module. Note,
    /// we allow the ability to set the router after the Keeper is constructed as a
    /// given Handler may need access the Keeper before being constructed. The router
    /// may only be set once.
    pub fn set_router(
        &mut self,
        router: Router<'a, DB, SK, E>,
    ) -> Result<(), RouterAlreadyExistsError> {
        if self.router.is_some() {
            Err(RouterAlreadyExistsError)
        } else {
            self.router = Some(router);
            Ok(())
        }
    }

    /// genesis initializes the evidence module's state from a provided genesis
    /// state.
    pub fn init_genesis(&self, ctx: &mut InitContext<'_, DB, SK>, genesis: GenesisState) {
        if let Err(e) = genesis.validate::<E>() {
            panic!("failed to validate evidence genesis state: {e}");
        }

        for e in genesis.evidence {
            let evidence = E::decode(e.value.as_slice()).expect("validation of types is passed");
            if self.evidence(ctx, evidence.hash()).unwrap_gas().is_some() {
                panic!("evidence with hash {} already exists", evidence.hash());
            }

            self.set_evidence(ctx, &evidence).unwrap_gas();
        }
    }

    /// evidence gets Evidence by hash in the module's KVStore.
    pub fn evidence<CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        evidence_hash: Hash,
    ) -> Result<Option<E>, GasStoreErrors> {
        let store = ctx.kv_store(&self.store_key);
        let store = store.prefix_store(KEY_PREFIX_EVIDENCE);
        Ok(store
            .get(evidence_hash.as_bytes())?
            .map(|bytes| E::decode(bytes.as_slice()).unwrap_or_corrupt()))
    }

    /// set_evidence sets Evidence by hash in the module's KVStore.
    pub fn set_evidence<CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        evidence: &E,
    ) -> Result<(), GasStoreErrors> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut store = store.prefix_store_mut(KEY_PREFIX_EVIDENCE);
        store.set(
            evidence.hash().as_bytes().to_vec(),
            evidence.encode_to_vec(),
        )
    }
}
