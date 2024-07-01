use crate::{
    keys::{
        addr_pubkey_relation_key, validator_missed_block_bit_array_key,
        validator_missed_block_bit_array_prefix_key, validator_signing_info_key,
    },
    GenesisState, SlashingParamsKeeper, ValidatorSigningInfo,
};
use gears::{
    context::{
        block::BlockContext, init::InitContext, InfallibleContextMut, QueryableContext,
        TransactionalContext,
    },
    error::{AppError, IBC_ENCODE_UNWRAP},
    params::ParamsSubspaceKey,
    store::{
        database::{ext::UnwrapCorrupt, Database},
        StoreKey,
    },
    tendermint::types::{
        proto::{
            crypto::PublicKey,
            event::{Event, EventAttribute},
            Protobuf,
        },
        time::Timestamp,
    },
    types::{address::ConsAddress, store::gas::ext::GasResultExt},
    x::{
        keepers::staking::{SlashingStakingKeeper, VALIDATOR_UPDATE_DELAY},
        module::Module,
        types::validator::StakingValidator,
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
        pub_keys
            .into_iter()
            .for_each(|pub_key| self.add_pub_key(ctx, &pub_key));

        genesis.signing_infos.into_iter().for_each(|info| {
            self.set_validator_signing_info(ctx, &info.address, &info.validator_signing_info)
        });

        genesis.missed_blocks.into_iter().for_each(|block| {
            block.missed_blocks.into_iter().for_each(|missed| {
                self.set_validator_missed_block_bit_array(
                    ctx,
                    &block.address,
                    missed.index,
                    missed.missed,
                )
            });
        });

        self.slashing_params_keeper.set(ctx, genesis.params);
    }

    pub fn handle_validator_signature<DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, SK>,
        addr_bytes: prost::bytes::Bytes,
        power: u32,
        signed: bool,
    ) -> anyhow::Result<()> {
        let height = ctx.height();

        // fetch the validator public key
        let cons_addr = ConsAddress::try_from(Vec::from(addr_bytes))?;
        self.get_pub_key(ctx, &cons_addr).ok_or(AppError::Custom(
            "validator consensus address not found".to_string(),
        ))?;

        // fetch signing info
        let mut sign_info =
            self.get_validator_signing_info(ctx, &cons_addr)
                .ok_or(AppError::Custom(
                    "Expected signing info for validator but it is not found".to_string(),
                ))?;

        // this is a relative index, so it counts blocks the validator *should* have signed
        // will use the 0-value default signing info if not present, except for start height
        let params = self.slashing_params_keeper.get(ctx);
        let index = sign_info.index_offset % params.signed_blocks_window as u32;
        sign_info.index_offset += 1;

        // Update signed block bit array & counter
        // This counter just tracks the sum of the bit array
        // That way we avoid needing to read/write the whole array each time
        let previous = self.get_validator_missed_block_bit_array(ctx, &cons_addr, index);

        match (previous, signed) {
            (false, false) => {
                // Array value has changed from not missed to missed, increment counter
                self.set_validator_missed_block_bit_array(ctx, &cons_addr, index, true);
                sign_info.missed_blocks_counter += 1;
            }
            (true, true) => {
                // Array value has changed from missed to not missed, decrement counter
                self.set_validator_missed_block_bit_array(ctx, &cons_addr, index, false);
                sign_info.missed_blocks_counter -= 1;
            }
            _ => {
                // Array value at this index has not changed, no need to update counter
            }
        }

        let min_signed_per_window = params.min_signed_per_window_u32()?;

        if !signed {
            ctx.append_events(vec![Event {
                r#type: "liveness".to_string(),
                attributes: vec![
                    EventAttribute {
                        key: "address".into(),
                        value: cons_addr.clone().to_string().into(),
                        index: false,
                    },
                    EventAttribute {
                        key: "missed_blocks".into(),
                        value: sign_info.missed_blocks_counter.to_string().into(),
                        index: false,
                    },
                    EventAttribute {
                        key: "height".into(),
                        value: height.to_string().into(),
                        index: false,
                    },
                ],
            }]);

            // TODO: how do we log?
            tracing::debug!(
                name: "absent validator",
                // TODO: what is target?
                target: "module::slashing",
                ?height,
                validator = cons_addr.to_string(),
                missed = sign_info.missed_blocks_counter,
                treshold = min_signed_per_window,
            );
            // logger.Debug(
            //     "absent validator",
            //     "height", height,
            //     "validator", consAddr.String(),
            //     "missed", signInfo.MissedBlocksCounter,
            //     "threshold", minSignedPerWindow,
            // )
        }

        let min_height = sign_info.start_height + params.signed_blocks_window as u32;
        let max_missed = params.signed_blocks_window as u32 - min_signed_per_window;

        // if we are past the minimum height and the validator has missed too many blocks, punish them
        if height > min_height && sign_info.missed_blocks_counter > max_missed {
            let validator_is_jailed = self
                .staking_keeper
                .validator_by_cons_addr(ctx, &cons_addr)
                .unwrap_gas()
                .map(|v| v.is_jailed())
                .unwrap_or_default();

            if validator_is_jailed {
                // Downtime confirmed: slash and jail the validator
                // We need to retrieve the stake distribution which signed the block, so we subtract ValidatorUpdateDelay from the evidence height,
                // and subtract an additional 1 since this is the LastCommit.
                let distribution_height = height
                    .saturating_sub(VALIDATOR_UPDATE_DELAY)
                    .saturating_sub(1);

                ctx.append_events(vec![Event {
                    r#type: "slash".to_string(),
                    attributes: vec![
                        EventAttribute {
                            key: "address".into(),
                            value: cons_addr.clone().to_string().into(),
                            index: false,
                        },
                        EventAttribute {
                            key: "power".into(),
                            value: format!("\"{}\"", power).into(),
                            index: false,
                        },
                        EventAttribute {
                            key: "reason".into(),
                            value: "missing_signature".to_string().into(),
                            index: false,
                        },
                        EventAttribute {
                            key: "jailed".into(),
                            value: cons_addr.clone().to_string().into(),
                            index: false,
                        },
                    ],
                }]);

                self.staking_keeper
                    .slash(
                        ctx,
                        &cons_addr,
                        distribution_height,
                        power,
                        params.slash_fraction_downtime,
                    )
                    .unwrap_gas();
                self.staking_keeper.jail(ctx, &cons_addr).unwrap_gas();

                let time = ctx.get_time();
                // TODO: consider to move the DateTime type and work with timestamps into Gears
                // The timestamp is provided by context and conversion won't fail.
                let time =
                    chrono::DateTime::from_timestamp(time.seconds, time.nanos as u32).unwrap();
                let delta = chrono::TimeDelta::nanoseconds(params.downtime_jail_duration);
                let jailed_until = time + delta;
                sign_info.jailed_until = Timestamp {
                    seconds: jailed_until.timestamp(),
                    nanos: jailed_until.timestamp_subsec_nanos() as i32,
                };
                // We need to reset the counter & array so that the validator won't be immediately slashed for downtime upon rebonding.
                sign_info.missed_blocks_counter = 0;
                sign_info.index_offset = 0;
                self.clear_validator_missed_block_bit_array(ctx, &cons_addr);

                // TODO: how do we log?
                tracing::info!(
                    name: "slashing and jailing validator due to liveness fault",
                    // TODO: what is target?
                    target: "module::slashing",
                    ?height,
                    validator = cons_addr.to_string(),
                    ?min_height,
                    treshold = min_signed_per_window,
                    slashed = params.slash_fraction_downtime.to_string(),
                    // TODO: what is better way to print time?
                    jailed_until = jailed_until.format("%Y-%m-%d %H:%M:%S").to_string(),
                );
            } else {
                // TODO: how do we log?
                tracing::info!(
                    name: "validator would have been slashed for downtime, but was either not found in store or already jailed",
                    // TODO: what is target?
                    target: "module::slashing",
                    validator = cons_addr.to_string(),
                );
            }
        }

        // Set the updated signing info
        self.set_validator_signing_info(ctx, &cons_addr, &sign_info);

        Ok(())
    }

    /// get_pub_key returns the pubkey from the adddress-pubkey relation
    pub fn get_pub_key<DB: Database>(
        &self,
        ctx: &BlockContext<'_, DB, SK>,
        addr: &ConsAddress,
    ) -> Option<PublicKey> {
        let store = ctx.kv_store(&self.store_key);
        let key = addr_pubkey_relation_key(addr.clone());
        store
            .get(&key)
            .map(|bytes| serde_json::from_slice(&bytes).unwrap_or_corrupt())
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

        // TODO: add Protobuf for PublicKey
        let value = serde_json::to_vec(pub_key).unwrap();
        store.set(key, value)
    }

    /// set_validator_signing_info sets the validator signing info to a consensus address key
    pub fn get_validator_signing_info<DB: Database>(
        &self,
        ctx: &BlockContext<'_, DB, SK>,
        addr: &ConsAddress,
    ) -> Option<ValidatorSigningInfo> {
        let store = ctx.kv_store(&self.store_key);
        let key = validator_signing_info_key(addr.clone());
        store
            .get(&key)
            .map(|bytes| serde_json::from_slice(&bytes).unwrap_or_corrupt())
    }

    /// set_validator_signing_info sets the validator signing info to a consensus address key
    pub fn set_validator_signing_info<DB: Database, CTX: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut CTX,
        addr: &ConsAddress,
        signing_info: &ValidatorSigningInfo,
    ) {
        let mut store = ctx.infallible_store_mut(&self.store_key);
        let key = validator_signing_info_key(addr.clone());
        let value = signing_info.encode_vec().expect(IBC_ENCODE_UNWRAP);
        store.set(key, value)
    }

    pub fn get_validator_missed_block_bit_array<DB: Database>(
        &self,
        ctx: &BlockContext<'_, DB, SK>,
        addr: &ConsAddress,
        index: u32,
    ) -> bool {
        let store = ctx.kv_store(&self.store_key);
        let key = validator_missed_block_bit_array_key(addr.clone(), index);
        store
            .get(&key)
            .map(|bytes| serde_json::from_slice(&bytes).unwrap_or_corrupt())
            .unwrap_or_default()
    }

    pub fn set_validator_missed_block_bit_array<DB: Database, CTX: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut CTX,
        addr: &ConsAddress,
        index: u32,
        missed: bool,
    ) {
        let mut store = ctx.infallible_store_mut(&self.store_key);
        let key = validator_missed_block_bit_array_key(addr.clone(), index);
        // TODO: something like that in sdk
        let value = serde_json::to_vec(&missed).unwrap();
        store.set(key, value)
    }

    /// clear_validator_missed_block_bit_array deletes every instance of ValidatorMissedBlockBitArray in the store
    pub fn clear_validator_missed_block_bit_array<
        DB: Database,
        CTX: InfallibleContextMut<DB, SK>,
    >(
        &self,
        ctx: &mut CTX,
        addr: &ConsAddress,
    ) {
        let store = ctx.infallible_store(&self.store_key);
        let prefix = validator_missed_block_bit_array_prefix_key(addr.clone());
        let keys = store
            .prefix_store(prefix.clone())
            .into_range(..)
            .map(|(k, _v)| k.to_vec())
            .collect::<Vec<_>>();

        let store = ctx.infallible_store_mut(&self.store_key);
        let mut store = store.prefix_store_mut(prefix);
        keys.iter().for_each(|k| {
            store.delete(k);
        });
    }
}
