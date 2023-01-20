use bytes::Bytes;
use prost::Message;

use proto_types::AccAddress;

use crate::error::AppError;

use super::proto::{MsgSend, Tx};

// TODO:
// 1. Many more checks are needed on DecodedTx::from_bytes see https://github.com/cosmos/cosmos-sdk/blob/2582f0aab7b2cbf66ade066fe570a4622cf0b098/x/auth/tx/decoder.go#L16

pub enum Msg {
    Send(MsgSend),
    Test,
}

impl Msg {
    pub fn get_signers(&self) -> &AccAddress {
        match &self {
            Msg::Send(msg) => return &msg.from_address,
            Msg::Test => todo!(),
        }
    }
}

pub struct DecodedTx {
    messages: Vec<Msg>,
}

impl DecodedTx {
    pub fn from_bytes(raw: Bytes) -> Result<DecodedTx, AppError> {
        let tx = Tx::decode(raw)?;
        let mut messages: Vec<Msg> = vec![];

        for msg in tx.body.messages {
            match msg.type_url.as_str() {
                "/cosmos.bank.v1beta1.MsgSend" => {
                    let msg = MsgSend::decode::<Bytes>(msg.value.into())?;
                    messages.push(Msg::Send(msg));
                }
                _ => return Err(AppError::TxParseError), // If any message is not recognized then reject the entire Tx
            };
        }

        Ok(DecodedTx { messages })
    }

    pub fn get_msgs(&self) -> &Vec<Msg> {
        return &self.messages;
    }
}
