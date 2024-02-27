use clap::Args;
use gears::types::context::query_context::QueryContext;

#[derive(Args, Debug, Clone)]
pub struct CliClientParams;

#[allow(dead_code)]
pub(super) fn query_command_handler<DB, SK>(
    _ctx: &QueryContext<'_, DB, SK>,
    _msg: &CliClientParams,
) -> anyhow::Result<String> {
    Ok(String::new())
}
