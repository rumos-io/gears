use std::{str::FromStr, time::Duration};

use gears::types::{base::coin::Coin, decimal256::Decimal256};

const DEFAULT_PERIOD: Duration = Duration::from_secs(172800); // 2 days

#[derive(Debug)]
pub struct DepositParams {
    pub min_deposit: Vec<Coin>,       // SendCoins?
    pub max_deposit_period: Duration, // ?
}

impl Default for DepositParams {
    fn default() -> Self {
        Self {
            min_deposit: vec![Coin::from_str("10000000stake").expect("default is valid")],
            max_deposit_period: DEFAULT_PERIOD,
        }
    }
}

pub struct VotingParams {
    pub voting_period: Duration,
}

impl Default for VotingParams {
    fn default() -> Self {
        Self {
            voting_period: DEFAULT_PERIOD,
        }
    }
}

pub struct TallyParams {
    pub quorum: Decimal256,
    pub threshold: Decimal256,
    pub veto_threshold: Decimal256,
}

impl Default for TallyParams {
    fn default() -> Self {
        Self {
            quorum: Decimal256::from_atomics(334_u16, 3).expect("Default should be valid"),
            threshold: Decimal256::from_atomics(5_u8, 1).expect("Default should be valid"),
            veto_threshold: Decimal256::from_atomics(334_u16, 3).expect("Default should be valid"),
        }
    }
}
