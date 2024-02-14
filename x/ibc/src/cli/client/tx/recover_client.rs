use clap::Args;
pub use ibc::core::client::types::proto::v1::MsgRecoverClient as RawMsgRecoverClient;

#[derive(Args, Debug)]
pub struct MsgRecoverClient {
    pub subject_client_id: String,
    pub substitute_client_id: String,
    pub signer: String,
}

pub(super) fn tx_command_handler(
    _msg: MsgRecoverClient,
) -> anyhow::Result<crate::message::Message> {
    unimplemented!()
}
