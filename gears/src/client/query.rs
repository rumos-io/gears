use clap::{arg, ArgAction, Command};

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
        );

    for sub_command in sub_commands {
        cli = cli.subcommand(sub_command);
    }

    cli
}
