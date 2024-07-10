use crate::{params::DistributionParamsKeeper, GenesisState};
use anyhow::anyhow;
pub use gears::{
    context::init::InitContext,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    x::{
        keepers::{
            auth::AuthKeeper, bank::StakingBankKeeper as BankKeeper, staking::SlashingStakingKeeper,
        },
        module::Module,
    },
};
use gears::{
    context::TransactionalContext,
    types::{
        address::ConsAddress,
        base::coins::Coins,
        store::gas::{errors::GasStoreErrors, ext::GasResultExt},
    },
};
use std::collections::HashMap;

mod store;

/// Keeper of the slashing store
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Keeper<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    // TODO: check/replace
    AK: AuthKeeper<SK, M>,
    // TODO: check/replace
    BK: BankKeeper<SK, M>,
    // TODO: check/replace
    SSK: SlashingStakingKeeper<SK, M>,
    M: Module,
> {
    store_key: SK,
    auth_keeper: AK,
    bank_keeper: BK,
    params_keeper: DistributionParamsKeeper<PSK>,
    staking_keeper: SSK,
    fee_collector_module: M,
    distribution_module: M,
    blocked_addrs: HashMap<String, bool>,
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: BankKeeper<SK, M>,
        SSK: SlashingStakingKeeper<SK, M>,
        M: Module,
    > Keeper<SK, PSK, AK, BK, SSK, M>
{
    pub fn new(
        store_key: SK,
        params_subspace_key: PSK,
        auth_keeper: AK,
        bank_keeper: BK,
        staking_keeper: SSK,
        fee_collector_module: M,
        distribution_module: M,
        blocked_addrs: HashMap<String, bool>,
    ) -> Self {
        Self {
            store_key,
            params_keeper: DistributionParamsKeeper {
                params_subspace_key,
            },
            auth_keeper,
            bank_keeper,
            staking_keeper,
            fee_collector_module,
            distribution_module,
            blocked_addrs,
        }
    }

    pub fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        genesis: GenesisState,
    ) -> anyhow::Result<()> {
        self.set_fee_pool(ctx, &genesis.fee_pool);
        self.params_keeper.set(ctx, genesis.params);

        genesis.delegator_withdraw_infos.iter().for_each(|dwi| {
            self.set_delegator_withdraw_addr(ctx, &dwi.delegator_address, &dwi.withdraw_address)
        });

        let previous_proposer = if !genesis.previous_proposer.is_empty() {
            ConsAddress::from_bech32(&genesis.previous_proposer)?
        } else {
            todo!("the sdk doesn't have this branch. It may use a default value");
        };

        self.set_previous_proposer_cons_addr(ctx, &previous_proposer);

        let mut module_holdings = vec![];
        for rew in genesis.outstanding_rewards {
            self.set_validator_outstanding_rewards(
                ctx,
                &rew.validator_address,
                &rew.outstanding_rewards,
            );
            module_holdings.push(rew.outstanding_rewards.rewards);
        }
        let start = module_holdings[0].clone();
        let module_holdings = module_holdings
            .into_iter()
            .take(1)
            .try_fold(start, |acc, holdings| acc.checked_add(holdings))?;

        genesis
            .validator_accumulated_commissions
            .iter()
            .for_each(|vac| {
                self.set_validator_accumulated_commission(
                    ctx,
                    &vac.validator_address,
                    &vac.accumulated,
                )
            });

        genesis.validator_historical_rewards.iter().for_each(|vhr| {
            self.set_validator_historical_rewards(
                ctx,
                &vhr.validator_address,
                vhr.period,
                &vhr.rewards,
            )
        });

        genesis.validator_current_rewards.iter().for_each(|vcr| {
            self.set_validator_current_rewards(ctx, &vcr.validator_address, &vcr.rewards)
        });

        genesis.delegator_starting_infos.iter().for_each(|dsi| {
            self.set_delegator_starting_info(
                ctx,
                &dsi.validator_address,
                &dsi.delegator_address,
                &dsi.starting_info,
            )
        });

        genesis.validator_slash_events.iter().for_each(|vse| {
            self.set_validator_slash_event(
                ctx,
                &vse.validator_address,
                vse.height,
                vse.period,
                &vse.validator_slash_event,
            )
        });

        let module_holdings = module_holdings.checked_add(genesis.fee_pool.community_pool)?;
        let (module_holdings_int, _) = module_holdings.truncate_decimal();

        // check if the module account exists

        self.check_set_distribution_account(ctx).unwrap_gas();
        let balances = self
            .bank_keeper
            .get_all_balances(ctx, self.distribution_module.get_address())
            .unwrap_gas();

        if module_holdings_int != Coins::new(balances)? {
            return Err(anyhow!(
                "distribution module balance does not match the module holdings".to_string(),
            ));
        }

        Ok(())
    }

    /// check_set_distribution_account creates module account for current module
    pub fn check_set_distribution_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
    ) -> Result<(), GasStoreErrors> {
        // TODO: here is fallible call with module as self
        self.auth_keeper
            .check_create_new_module_account(ctx, &self.distribution_module)
    }
}
