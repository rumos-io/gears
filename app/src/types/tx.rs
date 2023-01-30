use std::collections::HashSet;

use bytes::Bytes;
use ibc_proto::{cosmos::tx::v1beta1::SignerInfo, protobuf::Protobuf};
use prost::Message;

use proto_messages::cosmos::tx::v1beta1::{AuthInfo, Tx};
use proto_types::AccAddress;

use crate::{crypto::PubKey, error::AppError};

use super::proto::{self, MsgSend};

// TODO:
// 1. Many more checks are needed on DecodedTx::from_bytes see https://github.com/cosmos/cosmos-sdk/blob/2582f0aab7b2cbf66ade066fe570a4622cf0b098/x/auth/tx/decoder.go#L16
// 2. Implement equality on AccAddress to avoid conversion to string in get_signers()
pub enum Msg {
    Send(MsgSend),
    Test,
}

impl Msg {
    pub fn get_signers(&self) -> Vec<&AccAddress> {
        match &self {
            Msg::Send(msg) => return vec![&msg.from_address],
            Msg::Test => todo!(),
        }
    }

    pub fn validate_basic(&self) -> Result<(), AppError> {
        match &self {
            Msg::Send(msg) => proto::validate_coins(&msg.amount),
            Msg::Test => todo!(),
        }
    }
}

pub struct DecodedTx {
    messages: Vec<Msg>,
    auth_info: AuthInfo,
    signatures: Vec<Vec<u8>>,
}

impl DecodedTx {
    pub fn from_bytes(raw: Bytes) -> Result<DecodedTx, AppError> {
        let tx = Tx::decode(raw).map_err(|e| AppError::TxParseError(e.to_string()))?;
        let mut messages: Vec<Msg> = vec![];

        for msg in tx.body.messages {
            match msg.type_url.as_str() {
                "/cosmos.bank.v1beta1.MsgSend" => {
                    let msg = MsgSend::decode::<Bytes>(msg.value.into())?;
                    messages.push(Msg::Send(msg));
                }
                _ => return Err(AppError::TxParseError("message type not recognized".into())), // If any message is not recognized then reject the entire Tx
            };
        }

        Ok(DecodedTx {
            messages,
            auth_info: tx.auth_info,
            signatures: tx.signatures,
        })
    }

    pub fn get_msgs(&self) -> &Vec<Msg> {
        return &self.messages;
    }

    pub fn get_signers(&self) -> Vec<&AccAddress> {
        let mut signers = vec![];
        let mut seen = HashSet::new();

        for msg in &self.messages {
            for addr in msg.get_signers() {
                if seen.insert(addr.to_string()) {
                    signers.push(addr);
                }
            }
        }

        // ensure any specified fee payer is included in the required signers (at the end)
        let fee_payer = &self.auth_info.fee.payer;

        if let Some(addr) = fee_payer {
            if seen.insert(addr.to_string()) {
                signers.push(addr);
            }
        }

        return signers;
    }

    pub fn get_signatures(&self) -> &Vec<Vec<u8>> {
        return &self.signatures;
    }
}

#[cfg(test)]
mod tests {

    use proto_messages::cosmos::{
        base::v1beta1::{Coin, SendCoins},
        tx::v1beta1::Fee,
    };

    use super::*;

    #[test]
    fn get_signers_works() {
        let from_addr_1_3 =
            AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".into())
                .unwrap();

        let from_addr_2 =
            AccAddress::from_bech32("cosmos1l7hypmqk2yc334vc6vmdwzp5sdefygj2cs28wl".into())
                .unwrap();

        let to_addr =
            AccAddress::from_bech32("cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut".into())
                .unwrap();

        let fee_addr =
            AccAddress::from_bech32("cosmos1ryt87gjvnn8ph0lqac8k2x2kdek0sgh8uckq6u".into())
                .unwrap();

        // No fee address
        let tx = DecodedTx {
            messages: vec![
                Msg::Send(MsgSend {
                    from_address: from_addr_1_3.clone(),
                    to_address: to_addr.clone(),
                    amount: vec![],
                }),
                Msg::Send(MsgSend {
                    from_address: from_addr_2.clone(),
                    to_address: to_addr.clone(),
                    amount: vec![],
                }),
                Msg::Send(MsgSend {
                    from_address: from_addr_1_3.clone(),
                    to_address: to_addr.clone(),
                    amount: vec![],
                }),
            ],
            auth_info: AuthInfo {
                signer_infos: vec![],
                fee: Fee {
                    amount: Some(
                        SendCoins::new(vec![Coin {
                            denom: String::from("atom").try_into().unwrap(),
                            amount: cosmwasm_std::Uint256::one(),
                        }])
                        .unwrap(),
                    ),
                    gas_limit: 0,
                    payer: None,
                    granter: "granter".into(),
                },
            },
            signatures: vec![],
        };
        let signers = tx.get_signers();
        let expected: Vec<&AccAddress> = vec![&from_addr_1_3, &from_addr_2];
        assert_eq!(signers, expected);

        // Includes different fee address
        let tx = DecodedTx {
            messages: vec![
                Msg::Send(MsgSend {
                    from_address: from_addr_1_3.clone(),
                    to_address: to_addr.clone(),
                    amount: vec![],
                }),
                Msg::Send(MsgSend {
                    from_address: from_addr_2.clone(),
                    to_address: to_addr.clone(),
                    amount: vec![],
                }),
                Msg::Send(MsgSend {
                    from_address: from_addr_1_3.clone(),
                    to_address: to_addr.clone(),
                    amount: vec![],
                }),
            ],
            auth_info: AuthInfo {
                signer_infos: vec![],
                fee: Fee {
                    amount: None,
                    gas_limit: 0,
                    payer: Some(fee_addr.clone()),
                    granter: "granter".into(),
                },
            },
            signatures: vec![],
        };
        let signers = tx.get_signers();
        let expected: Vec<&AccAddress> = vec![&from_addr_1_3, &from_addr_2, &fee_addr];
        assert_eq!(signers, expected);

        // Includes duplicate fee address
        let tx = DecodedTx {
            messages: vec![
                Msg::Send(MsgSend {
                    from_address: from_addr_1_3.clone(),
                    to_address: to_addr.clone(),
                    amount: vec![],
                }),
                Msg::Send(MsgSend {
                    from_address: from_addr_2.clone(),
                    to_address: to_addr.clone(),
                    amount: vec![],
                }),
                Msg::Send(MsgSend {
                    from_address: from_addr_1_3.clone(),
                    to_address: to_addr.clone(),
                    amount: vec![],
                }),
            ],
            auth_info: AuthInfo {
                signer_infos: vec![],
                fee: Fee {
                    amount: None,
                    gas_limit: 0,
                    payer: Some(from_addr_2.clone()),
                    granter: "granter".into(),
                },
            },
            signatures: vec![],
        };
        let signers = tx.get_signers();
        let expected: Vec<&AccAddress> = vec![&from_addr_1_3, &from_addr_2];
        assert_eq!(signers, expected);
    }
}
