use clap::Args;
use gears::{client::query::run_query, types::context::query_context::QueryContext};
use proto_messages::cosmos::ibc::types::tendermint::types::{Header, ProtoHeader};
use tendermint::informal::block::Height;

#[derive(Args, Debug, Clone)]
pub struct CliClientParams;

#[allow(dead_code)]
pub(super) fn query_command_handler<DB, SK>(
    _ctx: &QueryContext<'_, DB, SK>,
    _args: CliClientParams,
    node: &str,
    height: Option<Height>,
) -> anyhow::Result<String> {
    let result = run_query::<Header, ProtoHeader>(
        Vec::new(), // TODO:
        "TODO".to_owned(),
        node,
        height,
    )?;

    let result = serde_json::to_string_pretty(&result)?;

    Ok(result)
}
