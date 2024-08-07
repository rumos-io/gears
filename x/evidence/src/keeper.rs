use crate::{
    types::{Evidence, EvidenceHandler},
    GenesisState,
};
use gears::{
    context::{init::InitContext, QueryableContext, TransactionalContext},
    core::any::google::Any,
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
use std::marker::PhantomData;

const KEY_PREFIX_EVIDENCE: [u8; 1] = [0x0];

/// Keeper of the evidence store
#[derive(Debug, Clone)]
pub struct Keeper<
    SK: StoreKey,
    StkK: SlashingStakingKeeper<SK, M>,
    SlsK: EvidenceSlashingKeeper<SK, M>,
    E: Evidence + Default,
    EH: EvidenceHandler<E>,
    M: Module,
> where
    <E as std::convert::TryFrom<Any>>::Error: std::fmt::Debug,
{
    store_key: SK,
    // TODO
    #[allow(dead_code)]
    staking_keeper: StkK,
    #[allow(dead_code)]
    slashing_keeper: SlsK,
    #[allow(dead_code)]
    evidence_handler: Option<EH>,
    _module: PhantomData<(M, E)>,
}

impl<
        SK: StoreKey,
        StkK: SlashingStakingKeeper<SK, M>,
        SlsK: EvidenceSlashingKeeper<SK, M>,
        E: Evidence + Default,
        EH: EvidenceHandler<E>,
        M: Module,
    > Keeper<SK, StkK, SlsK, E, EH, M>
where
    <E as std::convert::TryFrom<Any>>::Error: std::fmt::Debug,
{
    pub fn new(
        store_key: SK,
        staking_keeper: StkK,
        slashing_keeper: SlsK,
        evidence_handler: Option<EH>,
    ) -> Self {
        Self {
            store_key,
            staking_keeper,
            slashing_keeper,
            evidence_handler,
            _module: PhantomData,
        }
    }

    /// genesis initializes the evidence module's state from a provided genesis
    /// state.
    pub fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        genesis: GenesisState,
    ) {
        if let Err(e) = genesis.validate::<E>() {
            panic!("failed to validate evidence genesis state: {e}");
        }

        for e in genesis.evidence {
            let evidence = E::try_from(e).expect("validation of types is passed");
            if self.evidence(ctx, evidence.hash()).unwrap_gas().is_some() {
                panic!("evidence with hash {} already exists", evidence.hash());
            }

            self.set_evidence(ctx, &evidence).unwrap_gas();
        }
    }

    /// evidence gets Evidence by hash in the module's KVStore.
    pub fn evidence<CTX: QueryableContext<DB, SK>, DB: Database>(
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
    pub fn set_evidence<CTX: TransactionalContext<DB, SK>, DB: Database>(
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
