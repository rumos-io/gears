use clap::Args;
use tendermint::informal::block::Height;

#[derive(Args, Debug, Clone)]
pub struct CliClientParams;

pub(super) fn query_command_handler(
    _args: CliClientParams,
    _node: &str,
    _height: Option<Height>,
) -> anyhow::Result<String> {
    Ok(String::new())
}
