use clap::{arg, value_parser, ArgAction, Command};
use tendermint_informal::block::Height;

pub fn get_query_command(sub_commands: Vec<Command>) -> Command {
    let mut cli = Command::new("query")
        .about("Querying subcommands")
        .subcommand_required(true)
        .arg(
            arg!(--node)
                .help("<host>:<port> to Tendermint RPC interface for this chain")
                .default_value("http://localhost:26657")
                .action(ArgAction::Set)
                .global(true),
        )
        .arg(
            arg!(--height)
                .help("Use a specific height to query state at (this can error if the node is pruning state)")
                .default_value("0")
                .value_parser(value_parser!(Height))
                .action(ArgAction::Set)
                .global(true),
        );

    for sub_command in sub_commands {
        cli = cli.subcommand(sub_command);
    }

    cli
}
