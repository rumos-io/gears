use crate::{
    params::DistributionParams,
    types::{
        DelegatorStartingInfoRecord, DelegatorWithdrawInfo, FeePool, ValidatorCurrentRewardsRecord,
        ValidatorHistoricalRewardsRecord, ValidatorOutstandingRewardsRecord,
        ValidatorSlashEventRecord,
    },
    ValidatorAccumulatedCommissionRecord,
};
use serde::{Deserialize, Serialize};

/// GenesisState defines the distribution module's genesis state.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GenesisState {
    /// params defines all the parameters of the module
    pub params: DistributionParams,
    /// fee_pool defines the fee pool at genesis
    pub fee_pool: FeePool,
    /// delegator_withdraw_infos defines the delegator withdraw infos at genesis
    pub delegator_withdraw_infos: Vec<DelegatorWithdrawInfo>,
    /// previous_proposer defines the previous proposer at genesis
    pub previous_proposer: String,
    /// outstanding_rewards defines the outstanding rewards of all validators at genesis
    pub outstanding_rewards: Vec<ValidatorOutstandingRewardsRecord>,
    /// validator_accumulated_commissions defines the accumulated commisions of all validators at genesis
    pub validator_accumulated_commissions: Vec<ValidatorAccumulatedCommissionRecord>,
    /// validator_historical_rewards defines the historical rewards of all validators at genesis.
    pub validator_historical_rewards: Vec<ValidatorHistoricalRewardsRecord>,
    /// validator_current_rewards defines the current rewards of all validators at genesis.
    pub validator_current_rewards: Vec<ValidatorCurrentRewardsRecord>,
    /// delegator_starting_infos defines the delegator starting infos at genesis.
    pub delegator_starting_infos: Vec<DelegatorStartingInfoRecord>,
    /// validator_slash_events defines the validator slash events at genesis.
    pub validator_slash_events: Vec<ValidatorSlashEventRecord>,
}
