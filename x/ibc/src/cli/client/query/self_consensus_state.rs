use clap::Args;
use gears::types::context::query_context::QueryContext;
use tendermint::informal::block::Height;

#[derive(Args, Debug, Clone)]
pub struct CliClientParams;

#[allow(dead_code)]
pub(super) fn query_command_handler<DB, SK>(
    _ctx: &QueryContext<'_, DB, SK>,
    _args: CliClientParams,
    _node: &str,
    _height: Option<Height>,
) -> anyhow::Result<String> {
    Ok(String::new())
}
