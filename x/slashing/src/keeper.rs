use crate::{
    keys::{
        addr_pubkey_relation_key, validator_missed_block_bit_array_key, validator_signing_info_key,
    },
    GenesisState, SlashingParamsKeeper, ValidatorSigningInfo,
};
use gears::{
    context::init::InitContext,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    tendermint::types::proto::crypto::PublicKey,
    types::{address::ConsAddress, store::gas::ext::GasResultExt},
    x::{
        keepers::staking::SlashingStakingKeeper, module::Module, types::validator::StakingValidator,
    },
};
use std::marker::PhantomData;

pub(crate) const VALIDATOR_SIGNING_INFO_KEY_PREFIX: [u8; 1] = [0x1];
pub(crate) const VALIDATOR_MISSED_BLOCK_BIT_ARRAY_KEY_PREFIX: [u8; 1] = [0x2];
pub(crate) const ADDR_PUBKEY_RELATION_KEY_PREFIX: [u8; 1] = [0x3];

/// Keeper of the slashing store
#[derive(Debug, Clone)]
pub struct Keeper<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    SSK: SlashingStakingKeeper<SK, M>,
    M: Module,
> {
    store_key: SK,
    slashing_params_keeper: SlashingParamsKeeper<PSK>,
    staking_keeper: SSK,
    _module: PhantomData<M>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, SSK: SlashingStakingKeeper<SK, M>, M: Module>
    Keeper<SK, PSK, SSK, M>
{
    pub fn new(store_key: SK, params_subspace_key: PSK, staking_keeper: SSK) -> Self {
        Self {
            store_key,
            slashing_params_keeper: SlashingParamsKeeper {
                params_subspace_key,
            },
            staking_keeper,
            _module: PhantomData,
        }
    }

    /// init_genesis initializes default parameters
    /// and the keeper's address to pubkey map
    pub fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        genesis: GenesisState,
    ) {
        let pub_keys: Vec<PublicKey> = self
            .staking_keeper
            .validators_iter(ctx)
            .unwrap_gas()
            .map(|validator| {
                let validator = validator.unwrap_gas();
                validator.cons_pub_key().clone()
            })
            .collect();
        for pub_key in pub_keys {
            self.add_pub_key(ctx, &pub_key);
        }

        for info in genesis.signing_infos {
            self.set_validator_signing_info(ctx, &info.address, &info.validator_signing_info);
        }

        for block in genesis.missed_blocks {
            for missed in block.missed_blocks {
                self.set_validator_missed_block_bit_array(
                    ctx,
                    &block.address,
                    missed.index,
                    missed.missed,
                );
            }
        }

        self.slashing_params_keeper.set(ctx, genesis.params);
    }

    /// add_pub_key sets a address-pubkey relation
    pub fn add_pub_key<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        pub_key: &PublicKey,
    ) {
        let mut store = ctx.kv_store_mut(&self.store_key);
        // TODO: check the addr type for genesis
        let addr = ConsAddress::from(pub_key.clone());
        let key = addr_pubkey_relation_key(addr);

        let value = serde_json::to_vec(pub_key).unwrap();
        store.set(key, value)
    }

    /// set_validator_signing_info sets the validator signing info to a consensus address key
    pub fn set_validator_signing_info<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        addr: &ConsAddress,
        signing_info: &ValidatorSigningInfo,
    ) {
        let mut store = ctx.kv_store_mut(&self.store_key);
        let key = validator_signing_info_key(addr.clone());
        let value = serde_json::to_vec(signing_info).unwrap();
        store.set(key, value)
    }

    pub fn set_validator_missed_block_bit_array<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        addr: &ConsAddress,
        index: i64,
        missed: bool,
    ) {
        let mut store = ctx.kv_store_mut(&self.store_key);
        let key = validator_missed_block_bit_array_key(addr.clone(), index);
        // TODO: something like that in sdk
        let value = serde_json::to_vec(&missed).unwrap();
        store.set(key, value)
    }
}
