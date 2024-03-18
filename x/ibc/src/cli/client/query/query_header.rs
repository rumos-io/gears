use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct CliClientParams;

pub(super) fn query_command_handler(_args: CliClientParams) -> anyhow::Result<String> {
    Ok(String::new())
}
