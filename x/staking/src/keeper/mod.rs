use crate::{
    consts::{error::SERDE_ENCODING_DOMAIN_TYPE, keeper::*},
    traits::*,
    BondStatus, Delegation, DvPair, DvvTriplet, GenesisState, LastValidatorPower, Pool,
    Redelegation, StakingParamsKeeper, UnbondingDelegation, Validator,
};
use chrono::Utc;
use gears::{
    context::{
        block::BlockContext, init::InitContext, InfallibleContext, QueryableContext,
        TransactionalContext,
    },
    error::AppError,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    tendermint::types::{
        proto::{
            event::{Event, EventAttribute},
            validator::ValidatorUpdate,
        },
        time::Timestamp,
    },
    types::{
        address::{AccAddress, ValAddress},
        base::{coin::Coin, send::SendCoins},
        decimal256::Decimal256,
        store::gas::{errors::GasStoreErrors, ext::GasResultExt},
        uint::Uint256,
    },
    x::{keepers::auth::AuthKeeper, module::Module},
};
use prost::bytes::BufMut;
use std::{cmp::Ordering, collections::HashMap, u64};

// Each module contains methods of keeper with logic related to its name. It can be delegation and
// validator types.

const CTX_NO_GAS_UNWRAP: &str = "Context doesn't have any gas";

mod bonded;
mod delegation;
mod historical_info;
mod hooks;
mod mock_hook_keeper;
mod query;
mod redelegation;
mod tx;
mod unbonded;
mod unbonding;
mod validator;
mod validators_and_total_power;

pub use mock_hook_keeper::*;

#[derive(Debug, Clone)]
pub struct Keeper<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    AK: AuthKeeper<SK, M>,
    BK: BankKeeper<SK, M>,
    KH: KeeperHooks<SK, AK, M>,
    M: Module,
> {
    store_key: SK,
    auth_keeper: AK,
    bank_keeper: BK,
    staking_params_keeper: StakingParamsKeeper<PSK>,
    hooks_keeper: Option<KH>,
    bonded_module: M,
    not_bonded_module: M,
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: BankKeeper<SK, M>,
        KH: KeeperHooks<SK, AK, M>,
        M: Module,
    > Keeper<SK, PSK, AK, BK, KH, M>
{
    pub fn new(
        store_key: SK,
        params_subspace_key: PSK,
        auth_keeper: AK,
        bank_keeper: BK,
        hooks_keeper: Option<KH>,
        bonded_module: M,
        not_bonded_module: M,
    ) -> Self {
        let staking_params_keeper = StakingParamsKeeper {
            params_subspace_key,
        };

        Keeper {
            store_key,
            auth_keeper,
            bank_keeper,
            staking_params_keeper,
            hooks_keeper,
            bonded_module,
            not_bonded_module,
        }
    }

    pub fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        genesis: GenesisState,
    ) -> Vec<ValidatorUpdate> {
        let mut bonded_tokens = Uint256::zero();
        let mut not_bonded_tokens = Uint256::zero();

        // TODO
        // ctx = ctx.WithBlockHeight(1 - sdk.ValidatorUpdateDelay)

        self.set_pool(ctx, genesis.pool);
        self.set_last_total_power(ctx, genesis.last_total_power)
            .expect(CTX_NO_GAS_UNWRAP);
        self.staking_params_keeper.set(ctx, genesis.params.clone());

        for validator in genesis.validators {
            self.set_validator(ctx, &validator)
                .expect(CTX_NO_GAS_UNWRAP);
            // Manually set indices for the first time
            self.set_validator_by_cons_addr(ctx, &validator)
                .expect(CTX_NO_GAS_UNWRAP);
            self.set_validator_by_power_index(ctx, &validator)
                .expect(CTX_NO_GAS_UNWRAP);

            if !genesis.exported {
                self.after_validator_created(ctx, &validator);
            }

            if validator.status == BondStatus::Unbonding {
                self.insert_unbonding_validator_queue(ctx, &validator)
                    .expect(CTX_NO_GAS_UNWRAP);
            }

            match validator.status {
                BondStatus::Bonded => {
                    bonded_tokens += validator.tokens;
                }
                BondStatus::Unbonding | BondStatus::Unbonded => {
                    not_bonded_tokens += validator.tokens;
                }
            }
        }

        for delegation in genesis.delegations {
            if !genesis.exported {
                self.before_delegation_created(
                    ctx,
                    &delegation.delegator_address,
                    &delegation.validator_address,
                );
            }

            self.set_delegation(ctx, &delegation)
                .expect(CTX_NO_GAS_UNWRAP);

            if !genesis.exported {
                self.after_delegation_modified(
                    ctx,
                    &delegation.delegator_address,
                    &delegation.validator_address,
                );
            }
        }

        for unbonding_delegation in genesis.unbonding_delegations {
            self.set_unbonding_delegation(ctx, &unbonding_delegation);
            for entry in unbonding_delegation.entries.as_slice() {
                self.insert_ubd_queue(ctx, &unbonding_delegation, entry.completion_time.clone());
            }
        }

        for redelegation in genesis.redelegations {
            self.set_redelegation(ctx, &redelegation)
                .expect("setting of redelegation won't fail because used infallable context");
            for entry in &redelegation.entries {
                self.insert_redelegation_queue(ctx, &redelegation, entry.completion_time.clone())
                    .expect("setting of redelegation won't fail because used infallable context");
            }
        }

        let bonded_coins = if !bonded_tokens.is_zero() {
            vec![Coin {
                denom: genesis.params.bond_denom.clone(),
                amount: bonded_tokens,
            }]
        } else {
            vec![]
        };
        let not_bonded_coins = if !not_bonded_tokens.is_zero() {
            vec![Coin {
                denom: genesis.params.bond_denom,
                amount: not_bonded_tokens,
            }]
        } else {
            vec![]
        };

        let bonded_balance = self
            .bank_keeper
            .get_all_balances::<DB, InitContext<'_, DB, SK>>(ctx, self.bonded_module.get_address())
            .unwrap_gas();
        if bonded_balance
            .clone()
            .into_iter()
            .any(|e| e.amount.is_zero())
        {
            self.auth_keeper
                .check_create_new_module_account(ctx, &self.bonded_module)
                .unwrap_gas();
        }
        // if balance is different from bonded coins panic because genesis is most likely malformed
        assert_eq!(
            bonded_balance, bonded_coins,
            "bonded pool balance is different from bonded coins: {:?} <-> {:?}",
            bonded_balance, bonded_coins
        );

        let not_bonded_balance = self
            .bank_keeper
            .get_all_balances::<DB, InitContext<'_, DB, SK>>(
                ctx,
                self.not_bonded_module.get_address(),
            )
            .unwrap_gas();
        if not_bonded_balance
            .clone()
            .into_iter()
            .any(|e| e.amount.is_zero())
        {
            self.auth_keeper
                .check_create_new_module_account(ctx, &self.not_bonded_module)
                .unwrap_gas();
        }

        // if balance is different from non bonded coins panic because genesis is most likely malformed
        assert_eq!(
            not_bonded_balance, not_bonded_coins,
            "not bonded pool balance is different from not bonded coins: {:?} <-> {:?}",
            not_bonded_balance, not_bonded_coins,
        );

        let mut res = vec![];
        // don't need to run Tendermint updates if we exported
        if genesis.exported {
            for last_validator in genesis.last_validator_powers {
                self.set_last_validator_power(ctx, &last_validator)
                    .expect(CTX_NO_GAS_UNWRAP);
                let validator = self
                    .validator(ctx, &last_validator.address)
                    .expect("Init ctx doesn't have any gas")
                    .expect("validator in the store was not found");
                let mut update = validator.abci_validator_update(self.power_reduction(ctx));
                update.power = last_validator.power;
                res.push(update);
            }
        } else {
            // TODO: exit in sdk
            res = self.apply_and_return_validator_set_updates(ctx).unwrap();
        }
        res
    }

    pub fn set_pool<DB: Database>(&self, ctx: &mut InitContext<'_, DB, SK>, pool: Pool) {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut pool_store = store.prefix_store_mut(POOL_KEY);
        let pool = serde_json::to_vec(&pool).expect(SERDE_ENCODING_DOMAIN_TYPE);
        pool_store.set(pool.clone(), pool);
    }

    /// BlockValidatorUpdates calculates the ValidatorUpdates for the current block
    /// Called in each EndBlock
    pub fn block_validator_updates<DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, SK>,
    ) -> Vec<ValidatorUpdate> {
        // Calculate validator set changes.

        // NOTE: ApplyAndReturnValidatorSetUpdates has to come before
        // UnbondAllMatureValidatorQueue.
        // This fixes a bug when the unbonding period is instant (is the case in
        // some of the tests). The test expected the validator to be completely
        // unbonded after the Endblocker (go from Bonded -> Unbonding during
        // ApplyAndReturnValidatorSetUpdates and then Unbonding -> Unbonded during
        // UnbondAllMatureValidatorQueue).
        let validator_updates = match self.apply_and_return_validator_set_updates(ctx) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        };

        // unbond all mature validators from the unbonding queue
        self.unbond_all_mature_validators(ctx)
            .expect(CTX_NO_GAS_UNWRAP);

        // Remove all mature unbonding delegations from the ubd queue.
        let time = ctx.get_time();
        // TODO: consider to move the DataTime type and work with timestamps into Gears
        // The timestamp is provided by context and conversion won't fail.
        let mature_unbonds = self.dequeue_all_mature_ubd_queue(ctx, time.clone());
        for dv_pair in mature_unbonds {
            let val_addr = dv_pair.val_addr;
            let val_addr_str = val_addr.to_string();
            let del_addr = dv_pair.del_addr;
            let del_addr_str = del_addr.to_string();
            let balances = if let Ok(balances) = self.complete_unbonding(ctx, val_addr, del_addr) {
                balances
            } else {
                continue;
            };

            ctx.push_event(Event {
                r#type: EVENT_TYPE_COMPLETE_UNBONDING.to_string(),
                attributes: vec![
                    EventAttribute {
                        key: ATTRIBUTE_KEY_AMOUNT.as_bytes().into(),
                        value: serde_json::to_vec(&balances)
                            .expect(SERDE_ENCODING_DOMAIN_TYPE)
                            .into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_VALIDATOR.as_bytes().into(),
                        value: val_addr_str.as_bytes().to_vec().into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_DELEGATOR.as_bytes().into(),
                        value: del_addr_str.as_bytes().to_vec().into(),
                        index: false,
                    },
                ],
            });
        }
        // Remove all mature redelegations from the red queue.
        let mature_redelegations = self.dequeue_all_mature_redelegation_queue(ctx, time);
        for dvv_triplet in mature_redelegations {
            let val_src_addr = dvv_triplet.val_src_addr;
            let val_src_addr_str = val_src_addr.to_string();
            let val_dst_addr = dvv_triplet.val_dst_addr;
            let val_dst_addr_str = val_dst_addr.to_string();
            let del_addr = dvv_triplet.del_addr;
            let del_addr_str = del_addr.to_string();
            let balances = if let Ok(balances) =
                self.complete_redelegation(ctx, del_addr, val_src_addr, val_dst_addr)
            {
                balances
            } else {
                continue;
            };

            ctx.push_event(Event {
                r#type: EVENT_TYPE_COMPLETE_REDELEGATION.to_string(),
                attributes: vec![
                    EventAttribute {
                        key: ATTRIBUTE_KEY_AMOUNT.as_bytes().into(),
                        value: serde_json::to_vec(&balances)
                            .expect(SERDE_ENCODING_DOMAIN_TYPE)
                            .into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_DELEGATOR.as_bytes().into(),
                        value: del_addr_str.as_bytes().to_vec().into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_VALIDATOR.as_bytes().into(),
                        value: val_src_addr_str.as_bytes().to_vec().into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_VALIDATOR.as_bytes().into(),
                        value: val_dst_addr_str.as_bytes().to_vec().into(),
                        index: false,
                    },
                ],
            });
        }
        validator_updates
    }

    /// ApplyAndReturnValidatorSetUpdates applies and return accumulated updates to the bonded validator set. Also,
    /// * Updates the active valset as keyed by LastValidatorPowerKey.
    /// * Updates the total power as keyed by LastTotalPowerKey.
    /// * Updates validator status' according to updated powers.
    /// * Updates the fee pool bonded vs not-bonded tokens.
    /// * Updates relevant indices.
    /// It gets called once after genesis, another time maybe after genesis transactions,
    /// then once at every EndBlock.
    ///
    /// CONTRACT: Only validators with non-zero power or zero-power that were bonded
    /// at the previous block height or were removed from the validator set entirely
    /// are returned to Tendermint.
    pub fn apply_and_return_validator_set_updates<
        DB: Database,
        CTX: TransactionalContext<DB, SK> + InfallibleContext<DB, SK>,
    >(
        &self,
        ctx: &mut CTX,
    ) -> anyhow::Result<Vec<ValidatorUpdate>> {
        let params = self.staking_params_keeper.try_get(ctx)?;
        let max_validators = params.max_validators;
        let power_reduction = self.power_reduction(ctx);
        let mut total_power = 0;
        let mut amt_from_bonded_to_not_bonded = Uint256::zero();
        let amt_from_not_bonded_to_bonded = Uint256::zero();

        let mut last = self.last_validators_by_addr(ctx);
        let validators_map = self.validators_power_store_vals_map(ctx)?;

        let mut updates = vec![];

        for (_k, val_addr) in validators_map.iter().take(max_validators as usize) {
            // everything that is iterated in this loop is becoming or already a
            // part of the bonded validator set
            let mut validator: Validator = self
                .validator(ctx, val_addr)?
                .expect("validator should be presented in store");

            if validator.jailed {
                return Err(AppError::Custom(
                    "should never retrieve a jailed validator from the power store".to_string(),
                )
                .into());
            }
            // if we get to a zero-power validator (which we don't bond),
            // there are no more possible bonded validators
            if validator.tokens_to_consensus_power(self.power_reduction(ctx)) == 0 {
                break;
            }

            // apply the appropriate state change if necessary
            match validator.status {
                BondStatus::Unbonded => {
                    self.unbonded_to_bonded(ctx, &mut validator)?;
                    amt_from_bonded_to_not_bonded =
                        amt_from_not_bonded_to_bonded + validator.tokens;
                }
                BondStatus::Unbonding => {
                    self.unbonding_to_bonded(ctx, &mut validator)?;
                    amt_from_bonded_to_not_bonded =
                        amt_from_not_bonded_to_bonded + validator.tokens;
                }
                BondStatus::Bonded => {}
            }

            // fetch the old power bytes
            let val_addr_str = val_addr.to_string();
            let old_power_bytes = last.get(&val_addr_str);
            let new_power = validator.consensus_power(power_reduction);
            let new_power_bytes = new_power.to_be_bytes();
            // update the validator set if power has changed
            if old_power_bytes.is_none()
                || old_power_bytes.map(|v| v.as_slice()) != Some(&new_power_bytes)
            {
                updates.push(validator.abci_validator_update(power_reduction));

                self.set_last_validator_power(
                    ctx,
                    &LastValidatorPower {
                        address: val_addr.clone(),
                        power: new_power,
                    },
                )?;
            }

            last.remove(&val_addr_str);

            total_power += new_power;
        }

        let no_longer_bonded = sort_no_longer_bonded(&last)?;

        for val_addr in no_longer_bonded {
            let mut validator = self
                .validator(
                    ctx,
                    &ValAddress::from_bech32(&val_addr)
                        .expect("Expected correct validator address"),
                )?
                .expect("validator should be presented in store");
            self.bonded_to_unbonding(ctx, &mut validator)?;
            amt_from_bonded_to_not_bonded = amt_from_not_bonded_to_bonded + validator.tokens;
            self.delete_last_validator_power(ctx, &validator.operator_address)?;
            updates.push(validator.abci_validator_update_zero());
        }

        // Update the pools based on the recent updates in the validator set:
        // - The tokens from the non-bonded candidates that enter the new validator set need to be transferred
        // to the Bonded pool.
        // - The tokens from the bonded validators that are being kicked out from the validator set
        // need to be transferred to the NotBonded pool.
        // Compare and subtract the respective amounts to only perform one transfer.
        // This is done in order to avoid doing multiple updates inside each iterator/loop.
        match amt_from_bonded_to_not_bonded.cmp(&amt_from_not_bonded_to_bonded) {
            Ordering::Greater => {
                self.not_bonded_tokens_to_bonded(
                    ctx,
                    amt_from_bonded_to_not_bonded - amt_from_not_bonded_to_bonded,
                )?;
            }
            Ordering::Less => {
                self.bonded_tokens_to_not_bonded(
                    ctx,
                    amt_from_bonded_to_not_bonded - amt_from_not_bonded_to_bonded,
                )?;
            }
            Ordering::Equal => {}
        }

        // set total power on lookup index if there are any updates
        if !updates.is_empty() {
            self.set_last_total_power(ctx, Uint256::from_u128(total_power as u128))?;
        }
        Ok(updates)
    }

    pub fn power_reduction<DB: Database, CTX: QueryableContext<DB, SK>>(&self, _ctx: &CTX) -> i64 {
        // TODO: sdk constant in cosmos
        1_000_000
    }

    pub fn not_bonded_tokens_to_bonded<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        amount: Uint256,
    ) -> Result<(), GasStoreErrors> {
        // TODO: original routine is infallible, it means that the amount is a valid number.
        // The method is called from failable methods. Consider to provide correct solution taking
        // into account additional analisis.
        let params = self.staking_params_keeper.try_get(ctx)?;
        let coins = SendCoins::new(vec![Coin {
            denom: params.bond_denom,
            amount,
        }])
        .unwrap();

        // TODO: check and maybe remove unwrap
        self.bank_keeper
            .send_coins_from_module_to_module::<DB, CTX>(
                ctx,
                &self.not_bonded_module,
                &self.bonded_module,
                coins,
            )
            .unwrap();

        Ok(())
    }

    /// begin_info returns the completion time and height of a redelegation, along
    /// with a boolean signaling if the redelegation is complete based on the source
    /// validator.
    pub fn begin_info<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        val_addr: &ValAddress,
    ) -> Result<(Timestamp, u64, bool), GasStoreErrors> {
        // TODO: When would the validator not be found?
        let validator = self.validator(ctx, val_addr)?;
        let validator_status = validator
            .as_ref()
            .map(|v| v.status.clone())
            .unwrap_or(BondStatus::Bonded);

        match validator_status {
            BondStatus::Bonded => {
                // the longest wait - just unbonding period from now
                let params = self.staking_params_keeper.try_get(ctx)?;
                // TODO: consider to work with time in Gears
                let duration = chrono::TimeDelta::new(
                    params.unbonding_time.seconds,
                    params.unbonding_time.nanos as u32,
                )
                .unwrap();
                let time = ctx.get_time();
                // TODO: consider to work with time in Gears
                let time =
                    chrono::DateTime::from_timestamp(time.seconds, time.nanos as u32).unwrap();
                let completion_time = time + duration;
                let height = ctx.height();
                // TODO: consider to work with time in Gears
                let completion_time = Timestamp {
                    seconds: completion_time.timestamp(),
                    nanos: completion_time.timestamp_subsec_nanos() as i32,
                };
                Ok((completion_time, height, false))
            }
            BondStatus::Unbonded => Ok((
                Timestamp {
                    seconds: 0,
                    nanos: 0,
                },
                0,
                true,
            )),
            BondStatus::Unbonding => {
                let validator = validator.unwrap();
                Ok((validator.unbonding_time, validator.unbonding_height, false))
            }
        }
    }
}

/// given a map of remaining validators to previous bonded power
/// returns the list of validators to be unbonded, sorted by operator address
fn sort_no_longer_bonded(last: &HashMap<String, Vec<u8>>) -> anyhow::Result<Vec<String>> {
    let mut no_longer_bonded = last.iter().map(|(k, _v)| k.clone()).collect::<Vec<_>>();
    // sorted by address - order doesn't matter
    no_longer_bonded.sort();
    Ok(no_longer_bonded)
}
