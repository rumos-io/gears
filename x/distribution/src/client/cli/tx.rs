use crate::{
    Message, MsgFundCommunityPool, MsgSetWithdrawAddr, MsgWithdrawDelegatorReward,
    QueryWithdrawAllRewardsRequest, QueryWithdrawAllRewardsResponse,
    QueryWithdrawAllRewardsResponseRaw,
};
use anyhow::{Ok, Result};
use clap::{Args, Subcommand};
use gears::{
    commands::client::tx::ClientTxContext,
    core::Protobuf,
    types::{
        address::{AccAddress, ValAddress},
        base::{coin::UnsignedCoin, coins::UnsignedCoins},
        tx::Messages,
    },
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
    /// Withdraw all delegations rewards for a delegator
    WithdrawAllRewards,
    /// Change the default withdraw address for rewards associated with an address
    SetWithdrawAddr { withdraw_address: AccAddress },
    /// Funds the community pool with the specified amount
    FundCommunityPool { amount: UnsignedCoin },
}

pub fn run_staking_tx_command(
    ctx: &ClientTxContext,
    args: DistributionTxCli,
    from_address: AccAddress,
) -> Result<Messages<Message>> {
    match &args.command {
        DistributionCommands::WithdrawRewards {
            validator_address,
            commission,
        } => Ok(Message::WithdrawRewards(MsgWithdrawDelegatorReward {
            validator_address: validator_address.clone(),
            delegator_address: from_address.clone(),
            withdraw_commission: *commission,
        })
        .into()),
        DistributionCommands::WithdrawAllRewards => {
            let query = QueryWithdrawAllRewardsRequest {
                delegator_address: from_address.clone(),
            };
            let res = ctx
                .query::<QueryWithdrawAllRewardsResponse, QueryWithdrawAllRewardsResponseRaw>(
                    "/cosmos.distribution.v1beta1.Query/DelegatorValidators".to_string(),
                    query.encode_vec(),
                )?;

            let mut msgs = vec![];
            for addr in res.validators {
                let validator_address = ValAddress::from_bech32(&addr)?;
                msgs.push(Message::WithdrawRewards(MsgWithdrawDelegatorReward {
                    validator_address,
                    delegator_address: from_address.clone(),
                    withdraw_commission: false,
                }))
            }

            Ok(msgs.try_into()?)
        }
        DistributionCommands::SetWithdrawAddr { withdraw_address } => {
            Ok(Message::SetWithdrawAddr(MsgSetWithdrawAddr {
                delegator_address: from_address.clone(),
                withdraw_address: withdraw_address.clone(),
            })
            .into())
        }
        DistributionCommands::FundCommunityPool { amount } => {
            let amount = UnsignedCoins::new(vec![amount.clone()])?;
            Ok(Message::FundCommunityPool(MsgFundCommunityPool {
                amount,
                depositor: from_address.clone(),
            })
            .into())
        }
    }
}
