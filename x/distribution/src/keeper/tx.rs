use gears::tendermint::types::proto::event::{Event, EventAttribute};

use super::*;
use crate::MsgWithdrawDelegatorReward;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: BankKeeper<SK, M>,
        SSK: SlashingStakingKeeper<SK, M>,
        M: Module,
    > Keeper<SK, PSK, AK, BK, SSK, M>
{
    pub fn withdraw_delegator_reward_and_commission<DB: Database>(
        &self,
        ctx: &mut TxContext<DB, SK>,
        msg: &MsgWithdrawDelegatorReward,
    ) -> Result<(), AppError> {
        self.withdraw_delegation_rewards(ctx, &msg.delegator_address, &msg.validator_address)?;

        ctx.push_event(Event {
            r#type: "message".to_string(),
            attributes: vec![
                EventAttribute {
                    key: "module".into(),
                    value: self.distribution_module.get_name().into(),
                    index: false,
                },
                EventAttribute {
                    key: "sender".into(),
                    value: msg.delegator_address.to_string().into(),
                    index: false,
                },
            ],
        });

        if msg.withdraw_commission {
            self.withdraw_validator_commission(ctx, &msg.validator_address)?;
            ctx.push_event(Event {
                r#type: "message".to_string(),
                attributes: vec![
                    EventAttribute {
                        key: "module".into(),
                        value: self.distribution_module.get_name().into(),
                        index: false,
                    },
                    EventAttribute {
                        key: "sender".into(),
                        value: msg.validator_address.to_string().into(),
                        index: false,
                    },
                ],
            });
        }

        Ok(())
    }
}
