use gears::types::base::coin::UnsignedCoin;
use gears::types::decimal256::CosmosDecimalProtoString;
use gears::types::uint::Uint256;
use gears::{
    derive::{Protobuf, Raw},
    types::decimal256::Decimal256,
};

use crate::params::MintingParams;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Raw, Protobuf)]
pub struct Minter {
    #[raw(kind(string), raw = String)]
    #[proto(
        from = "Decimal256::from_cosmos_proto_string",
        from_ref,
        into = "Decimal256::to_cosmos_proto_string",
        into_ref
    )]
    pub inflation: Decimal256,
    #[raw(kind(string), raw = String)]
    #[proto(
        from = "Decimal256::from_cosmos_proto_string",
        from_ref,
        into = "Decimal256::to_cosmos_proto_string",
        into_ref
    )]
    pub annual_provisions: Decimal256,
}

impl Default for Minter {
    fn default() -> Self {
        Self {
            inflation: Decimal256::from_atomics(13_u8, 2).expect("default is valid"),
            annual_provisions: Decimal256::zero(),
        }
    }
}

impl Minter {
    pub fn next_inflation_rate(
        &self,
        params: &MintingParams,
        bonded_ratio: Decimal256,
    ) -> Option<Decimal256> {
        let inflation_change_per_year = Decimal256::one()
            .checked_sub(bonded_ratio.checked_div(params.goal_bonded).ok()?)
            .ok()?
            .checked_mul(params.inflation_rate_change)
            .ok()?;

        let inflation_rate_change = inflation_change_per_year
            .checked_div(Decimal256::new(Uint256::from(params.blocks_per_year)))
            .ok()?;

        let mut inflation = self.inflation.checked_add(inflation_rate_change).ok()?; // note inflationRateChange may be negative
        if inflation > params.inflation_max {
            inflation = params.inflation_max;
        }
        if inflation < params.inflation_min {
            inflation = params.inflation_min;
        }

        Some(inflation)
    }

    pub fn next_annual_provision(&self, total_supply: Decimal256) -> Option<Decimal256> {
        self.inflation.checked_mul(total_supply).ok()
    }

    pub fn block_provision(&self, params: &MintingParams) -> Option<UnsignedCoin> {
        let provision_amount = self
            .annual_provisions
            .checked_div(Decimal256::new(Uint256::from(params.blocks_per_year)))
            .ok()?;

        Some(UnsignedCoin {
            denom: params.mint_denom.clone(),
            amount: provision_amount.to_uint_floor(),
        })
    }
}
