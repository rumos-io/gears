use tendermint::informal::block::Height;

pub trait QueryHandler {
    type QueryCommands;

    fn prepare_query(
        &self,
        command: Self::QueryCommands,
        node: &str,
        height: Option<Height>,
    ) -> anyhow::Result<()>;

    fn handle_query() -> anyhow::Result<()>;
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
