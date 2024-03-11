use clap::Args;
use gears::client::query::run_query;
use proto_messages::cosmos::ibc::types::tendermint::types::{Header, ProtoHeader};
use tendermint::informal::block::Height;

#[derive(Args, Debug, Clone)]
pub struct CliClientParams;

pub(super) async fn query_command_handler(
    _args: CliClientParams,
    node: &str,
    height: Option<Height>,
) -> anyhow::Result<String> {
    let result = run_query::<Header, ProtoHeader>(
        Vec::new(), // TODO:
        "TODO".to_owned(),
        node,
        height,
    )
    .await?;

    let result = serde_json::to_string_pretty(&result)?;

    Ok(result)
}
