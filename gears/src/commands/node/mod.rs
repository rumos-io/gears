pub mod genesis;
pub mod init;
pub mod run;

#[derive(Debug, Clone)]
pub enum AppCommands<AUX> {
    Init(init::InitCommand),
    Run(run::RunCommand),
    GenesisAdd(genesis::GenesisCommand),
    Aux(AUX),
}
