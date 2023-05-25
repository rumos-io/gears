use std::collections::HashSet;

use bytes::Bytes;
use ibc_proto::{cosmos::tx::v1beta1::TxRaw, protobuf::Protobuf};

use prost::Message;
use proto_messages::cosmos::{
    base::v1beta1::SendCoins,
    tx::v1beta1::{AuthInfo, Msg, PublicKey, Tx, TxBody},
};
use proto_types::AccAddress;

use crate::error::AppError;

//use super::proto::{self};

// TODO:
// 0. Move all of DecodeTx functionality into proto_messages::cosmos::tx::v1beta1::tx
// 1. Many more checks are needed on DecodedTx::from_bytes see https://github.com/cosmos/cosmos-sdk/blob/2582f0aab7b2cbf66ade066fe570a4622cf0b098/x/auth/tx/decoder.go#L16
// 2. Implement equality on AccAddress to avoid conversion to string in get_signers()
// 3. Consider removing the "seen" hashset in get_signers()

pub struct SignatureData {
    pub signature: Vec<u8>,
    pub sequence: u64,
}

pub struct DecodedTx {
    messages: Vec<Msg>,
    auth_info: AuthInfo,
    signatures: Vec<Vec<u8>>,
    body: TxBody,
    signatures_data: Vec<SignatureData>,
    pub tx_raw: TxRaw,
}

impl DecodedTx {
    pub fn from_bytes(raw: Bytes) -> Result<DecodedTx, AppError> {
        let tx = Tx::decode(raw.clone()).map_err(|e| AppError::TxParseError(e.to_string()))?;
        let tx_raw = TxRaw::decode(raw).map_err(|e| AppError::TxParseError(e.to_string()))?;

        // extract signatures data when decoding - this isn't done in the SDK
        if tx.signatures.len() != tx.auth_info.signer_infos.len() {
            return Err(AppError::TxValidation("signature list is empty".into()));
        }
        let mut signatures_data = Vec::with_capacity(tx.signatures.len());
        for (i, signature) in tx.signatures.iter().enumerate() {
            signatures_data.push(SignatureData {
                signature: signature.clone(),
                // the check above, tx.signatures.len() != tx.auth_info.signer_infos.len(), ensures that this indexing is safe
                sequence: tx.auth_info.signer_infos[i].sequence,
            })
        }

        Ok(DecodedTx {
            messages: tx.body.messages.clone(),
            auth_info: tx.auth_info,
            signatures: tx.signatures,
            body: tx.body,
            signatures_data,
            tx_raw,
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

    pub fn get_signatures_data(&self) -> &Vec<SignatureData> {
        &self.signatures_data
    }

    pub fn get_timeout_height(&self) -> u64 {
        self.body.timeout_height
    }

    pub fn get_memo(&self) -> &str {
        &self.body.memo
    }

    pub fn get_fee(&self) -> &Option<SendCoins> {
        return &self.auth_info.fee.amount;
    }

    pub fn get_fee_payer(&self) -> &AccAddress {
        if let Some(payer) = &self.auth_info.fee.payer {
            return payer;
        } else {
            // At least one signer exists due to Ante::validate_basic_ante_handler()
            return self.get_signers()[0];
        }
    }

    pub fn get_public_keys(&self) -> Vec<&Option<PublicKey>> {
        self.auth_info
            .signer_infos
            .iter()
            .map(|si| &si.public_key)
            .collect()
    }
}

#[cfg(test)]
pub mod tests {

    use cosmwasm_std::Uint256;
    use ibc_proto::cosmos::tx::v1beta1::SignDoc;
    use ibc_relayer::keyring::{Secp256k1KeyPair, SigningKeyPair};
    use proto_messages::cosmos::{
        bank::v1beta1::MsgSend,
        base::v1beta1::{Coin, SendCoins},
        tx::v1beta1::{Fee, SignerInfo},
    };

    use super::*;

    /// Generates a signed transaction. This is used by other modules.
    pub fn get_signed_tx() -> DecodedTx {
        let from_addr_1 =
            AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".into())
                .unwrap();

        let to_addr =
            AccAddress::from_bech32("cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut".into())
                .unwrap();

        let key_pair: Secp256k1KeyPair = serde_json::from_str(
            r#"{
            "private_key": "f6fdd0e88e3988cc108690e28184508471f48eba283eeb61fce858f7b7a9642f",
            "public_key": "02f504b051dbb2be349d34a65a1ec25984591c6c1fe1ca512ed2656913b8540a2a",
            "address": [
              129,
              58,
              194,
              42,
              97,
              73,
              22,
              85,
              226,
              120,
              106,
              224,
              209,
              39,
              214,
              153,
              11,
              251,
              251,
              222
            ],
            "address_type": "Cosmos",
            "account": "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux"
          }"#,
        )
        .unwrap();

        let auth_info = AuthInfo {
            signer_infos: vec![SignerInfo {
                public_key: Some(PublicKey::Secp256k1(
                    key_pair.public_key.serialize().to_vec().try_into().unwrap(),
                )),
                mode_info: None,
                sequence: 1,
            }],
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
            tip: None,
        };

        let messages = vec![Msg::Send(MsgSend {
            from_address: from_addr_1.clone(),
            to_address: to_addr.clone(),
            amount: SendCoins::new(vec![Coin {
                denom: String::from("atom").try_into().unwrap(),
                amount: Uint256::one(),
            }])
            .unwrap(),
        })];

        let tx_body = TxBody {
            messages: messages.clone(),
            memo: "".into(),
            timeout_height: 0,
            extension_options: vec![],
            non_critical_extension_options: vec![],
        };

        let sign_doc = SignDoc {
            body_bytes: tx_body.encode_vec(),
            auth_info_bytes: auth_info.encode_vec(),
            chain_id: "unit-testing".into(),
            account_number: 1,
        };

        let signature = key_pair.sign(&sign_doc.encode_to_vec()).unwrap();

        DecodedTx {
            messages,
            auth_info: auth_info.clone(),
            signatures: vec![signature.clone()],
            body: tx_body.clone(),
            signatures_data: vec![SignatureData {
                signature: signature.clone(),
                sequence: 1,
            }],
            tx_raw: TxRaw {
                body_bytes: tx_body.encode_vec(),
                auth_info_bytes: auth_info.encode_vec(),
                signatures: vec![signature],
            },
        }
    }

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
                    amount: SendCoins::new(vec![Coin {
                        denom: String::from("atom").try_into().unwrap(),
                        amount: Uint256::one(),
                    }])
                    .unwrap(),
                }),
                Msg::Send(MsgSend {
                    from_address: from_addr_2.clone(),
                    to_address: to_addr.clone(),
                    amount: SendCoins::new(vec![Coin {
                        denom: String::from("atom").try_into().unwrap(),
                        amount: Uint256::one(),
                    }])
                    .unwrap(),
                }),
                Msg::Send(MsgSend {
                    from_address: from_addr_1_3.clone(),
                    to_address: to_addr.clone(),
                    amount: SendCoins::new(vec![Coin {
                        denom: String::from("atom").try_into().unwrap(),
                        amount: Uint256::one(),
                    }])
                    .unwrap(),
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
                tip: None,
            },
            signatures: vec![],
            body: TxBody {
                messages: vec![],
                memo: "".into(),
                timeout_height: 0,
                extension_options: vec![],
                non_critical_extension_options: vec![],
            },
            signatures_data: vec![],
            tx_raw: TxRaw {
                body_bytes: vec![],
                auth_info_bytes: vec![],
                signatures: vec![],
            },
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
                    amount: SendCoins::new(vec![Coin {
                        denom: String::from("atom").try_into().unwrap(),
                        amount: Uint256::one(),
                    }])
                    .unwrap(),
                }),
                Msg::Send(MsgSend {
                    from_address: from_addr_2.clone(),
                    to_address: to_addr.clone(),
                    amount: SendCoins::new(vec![Coin {
                        denom: String::from("atom").try_into().unwrap(),
                        amount: Uint256::one(),
                    }])
                    .unwrap(),
                }),
                Msg::Send(MsgSend {
                    from_address: from_addr_1_3.clone(),
                    to_address: to_addr.clone(),
                    amount: SendCoins::new(vec![Coin {
                        denom: String::from("atom").try_into().unwrap(),
                        amount: Uint256::one(),
                    }])
                    .unwrap(),
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
                tip: None,
            },
            signatures: vec![],
            body: TxBody {
                messages: vec![],
                memo: "".into(),
                timeout_height: 0,
                extension_options: vec![],
                non_critical_extension_options: vec![],
            },
            signatures_data: vec![],
            tx_raw: TxRaw {
                body_bytes: vec![],
                auth_info_bytes: vec![],
                signatures: vec![],
            },
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
                    amount: SendCoins::new(vec![Coin {
                        denom: String::from("atom").try_into().unwrap(),
                        amount: Uint256::one(),
                    }])
                    .unwrap(),
                }),
                Msg::Send(MsgSend {
                    from_address: from_addr_2.clone(),
                    to_address: to_addr.clone(),
                    amount: SendCoins::new(vec![Coin {
                        denom: String::from("atom").try_into().unwrap(),
                        amount: Uint256::one(),
                    }])
                    .unwrap(),
                }),
                Msg::Send(MsgSend {
                    from_address: from_addr_1_3.clone(),
                    to_address: to_addr.clone(),
                    amount: SendCoins::new(vec![Coin {
                        denom: String::from("atom").try_into().unwrap(),
                        amount: Uint256::one(),
                    }])
                    .unwrap(),
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
                tip: None,
            },
            signatures: vec![],
            body: TxBody {
                messages: vec![],
                memo: "".into(),
                timeout_height: 0,
                extension_options: vec![],
                non_critical_extension_options: vec![],
            },
            signatures_data: vec![],
            tx_raw: TxRaw {
                body_bytes: vec![],
                auth_info_bytes: vec![],
                signatures: vec![],
            },
        };
        let signers = tx.get_signers();
        let expected: Vec<&AccAddress> = vec![&from_addr_1_3, &from_addr_2];
        assert_eq!(signers, expected);
    }
}
