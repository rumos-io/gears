use proto_messages::cosmos::tx::v1beta1::message::Message;
use proto_types::AccAddress;
use tendermint::informal::block::Height;

pub trait TxHandler {
    type Message: Message;
    type TxCommands;

    fn handle_tx_command(
        &self,
        command: Self::TxCommands,
        from_address: AccAddress,
    ) -> anyhow::Result<Self::Message>;
}

pub trait QueryHandler {
    type QueryCommands;

    fn handle_query_command(
        &self,
        command: Self::QueryCommands,
        node: &str,
        height: Option<Height>,
    ) -> anyhow::Result<()>;
}

/// Name aux stands for `auxiliary`. In terms of implementation this is more like user extension to CLI.
/// It's reason exists to add user specific commands which doesn't supports usually.
pub trait AuxHandler {
    type AuxCommands; // TODO: use NilAuxCommand as default if/when associated type defaults land https://github.com/rust-lang/rust/issues/29661

    #[allow(unused_variables)]
    fn handle_aux_commands(&self, command: Self::AuxCommands) -> anyhow::Result<()> {
        Ok(())
    }
}
