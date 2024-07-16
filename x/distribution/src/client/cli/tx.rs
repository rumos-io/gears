use crate::{Message, MsgFundCommunityPool, MsgSetWithdrawAddr, MsgWithdrawDelegatorReward};
use anyhow::{Ok, Result};
use clap::{Args, Subcommand};
use gears::types::{
    address::{AccAddress, ValAddress},
    base::{coin::UnsignedCoin, coins::UnsignedCoins},
};

#[derive(Args, Debug, Clone)]
pub struct DistributionTxCli {
    #[command(subcommand)]
    pub command: DistributionCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum DistributionCommands {
    /// Withdraw rewards from a given delegation address, and optionally withdraw validator commission if the delegation address given is a validator operator
    WithdrawRewards {
        validator_address: ValAddress,
        /// Withdraw the validator's commission in addition to the rewards
        #[arg(long, default_value_t = false)]
        commission: bool,
    },
    /// Change the default withdraw address for rewards associated with an address
    SetWithdrawAddr { withdraw_address: AccAddress },
    /// Funds the community pool with the specified amount
    FundCommunityPool { amount: UnsignedCoin },
}

pub fn run_staking_tx_command(
    args: DistributionTxCli,
    from_address: AccAddress,
) -> Result<Message> {
    match &args.command {
        DistributionCommands::WithdrawRewards {
            validator_address,
            commission,
        } => Ok(Message::WithdrawRewards(MsgWithdrawDelegatorReward {
            validator_address: validator_address.clone(),
            delegator_address: from_address.clone(),
            withdraw_commission: *commission,
        })),
        DistributionCommands::SetWithdrawAddr { withdraw_address } => {
            Ok(Message::SetWithdrawAddr(MsgSetWithdrawAddr {
                delegator_address: from_address.clone(),
                withdraw_address: withdraw_address.clone(),
            }))
        }
        DistributionCommands::FundCommunityPool { amount } => {
            let amount = UnsignedCoins::new(vec![amount.clone()])?;
            Ok(Message::FundCommunityPool(MsgFundCommunityPool {
                amount,
                depositor: from_address.clone(),
            }))
        }
    }
}
