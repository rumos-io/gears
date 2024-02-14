use clap::Subcommand;

// IBC-GO: modules/core/02-client/client/cli/cli.go
/// IBC client transaction subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    ClientCreate,
    ClientUpdate,
    ClientUpgrade,
    Misbehavior,
    RecoverClientProposal,
    IBCUpgradeProposal,
}

// IBC-GO: modules/core/02-client/client/cli/cli.go
/// IBC client query subcommands
#[derive(Subcommand, Debug)]
pub enum QueryCommands {
    State,
    States,
    Status,
    Params,
    Header,
    ConsensusState,
    ConsensusStates,
    ConsensusStateHeights,
    SelfConsensusState,
}