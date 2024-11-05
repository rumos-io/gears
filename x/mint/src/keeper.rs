use gears::{
    context::{InfallibleContext, InfallibleContextMut},
    core::Protobuf,
    extensions::{corruption::UnwrapCorrupt, gas::GasResultExt},
    store::{database::Database, StoreKey},
    types::{
        base::{coin::UnsignedCoin, coins::UnsignedCoins},
        decimal256::Decimal256,
        uint::Uint256,
    },
    x::{
        errors::BankKeeperError,
        keepers::mint::{MintingBankKeeper, MintingStakingKeeper},
        module::Module,
    },
};

use crate::types::minter::Minter;

const MINTER_KEY: [u8; 1] = [0x00];

#[derive(Debug, Clone)]
pub struct MintKeeper<SK, BK, STK, M> {
    store_key: SK,
    staking_keeper: STK,
    bank_keeper: BK,
    module: M,
    fee_collector: M,
}

impl<SK, BK, STK, M> MintKeeper<SK, BK, STK, M> {
    pub fn new(
        store_key: SK,
        staking_keeper: STK,
        bank_keeper: BK,
        module: M,
        fee_collector: M,
    ) -> Self {
        Self {
            store_key,
            staking_keeper,
            bank_keeper,
            module,
            fee_collector,
        }
    }
}

impl<SK: StoreKey, BK: MintingBankKeeper<SK, M>, STK: MintingStakingKeeper<SK, M>, M: Module>
    MintKeeper<SK, BK, STK, M>
{
    pub fn minter<CTX: InfallibleContext<DB, SK>, DB: Database>(
        &self,
        ctx: &CTX,
    ) -> Option<Minter> {
        ctx.infallible_store(&self.store_key)
            .get(&MINTER_KEY)
            .map(|this| Minter::decode_vec(&this).unwrap_or_corrupt())
    }

    pub fn minter_set<CTX: InfallibleContextMut<DB, SK>, DB: Database>(
        &self,
        ctx: &mut CTX,
        minter: &Minter,
    ) {
        ctx.infallible_store_mut(&self.store_key)
            .set(MINTER_KEY, minter.encode_vec());
    }

    pub fn staking_token_supply<CTX: InfallibleContext<DB, SK>, DB: Database>(
        &self,
        ctx: &CTX,
    ) -> Option<UnsignedCoin> {
        self.bank_keeper
            .supply(ctx, &self.staking_keeper.staking_denom(ctx).unwrap_gas())
            .unwrap_gas()
    }

    pub fn bonded_ratio<CTX: InfallibleContext<DB, SK>, DB: Database>(
        &self,
        ctx: &CTX,
    ) -> Decimal256 {
        let stake_supply = match self.staking_token_supply(ctx) {
            Some(supply) => supply.amount,
            None => Uint256::zero(),
        };

        match stake_supply > Uint256::zero() {
            true => match self
                .staking_keeper
                .total_bonded_tokens(ctx)
                .unwrap_gas()
                .checked_div(Decimal256::new(stake_supply))
            {
                Ok(div) => div,
                Err(_) => panic!("overflow"),
            },
            false => Decimal256::zero(),
        }
    }

    pub fn mint_coins<CTX: InfallibleContextMut<DB, SK>, DB: Database>(
        &self,
        ctx: &mut CTX,
        amount: UnsignedCoins,
    ) -> Result<(), BankKeeperError> {
        self.bank_keeper.mint_coins(ctx, &self.module, amount)
    }

    pub fn collect_fees<CTX: InfallibleContextMut<DB, SK>, DB: Database>(
        &self,
        ctx: &mut CTX,
        fees: UnsignedCoins,
    ) -> Result<(), BankKeeperError> {
        self.bank_keeper.send_coins_from_module_to_module(
            ctx,
            &self.module,
            &self.fee_collector,
            fees,
        )
    }
}
