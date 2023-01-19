use bytes::Bytes;
use prost::Message;

use ibc_proto::cosmos::tx::v1beta1::Tx;
use proto_types::AccAddress;

use super::proto::MsgSend;

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
    pub fn from_bytes(raw: Bytes) -> DecodedTx {
        let tx = Tx::decode(raw).unwrap();
        let body = tx.body.unwrap();
        let url = body.messages[0].clone().type_url;

        let mut messages: Vec<Msg> = vec![];

        match url.as_str() {
            "/cosmos.bank.v1beta1.MsgSend" => {
                let msg = MsgSend::decode::<Bytes>(body.clone().messages[0].clone().value.into())
                    .unwrap();

                messages.push(Msg::Send(msg));
            }
            _ => (),
        };

        DecodedTx { messages }
    }

    pub fn get_msgs(&self) -> &Vec<Msg> {
        return &self.messages;
    }
}
