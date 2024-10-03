use gears::types::tx::metadata::Metadata;
use metadata::CoinsMetaGenesisCmd;

pub mod cli;

mod metadata;

#[derive(Debug, Clone)]
pub enum BankAuxCmd {
    Genesis(CoinsMetaGenesisCmd),
}

pub fn handle_aux_cmd(cmd: BankAuxCmd, in_genesis_path: &str) -> anyhow::Result<()> {
    match cmd {
        BankAuxCmd::Genesis(CoinsMetaGenesisCmd {
            home,
            metadata,
            dedup_input,
            overwrite_dup,
            dry,
        }) => {
            let metadata = match serde_json::from_str(&metadata) {
                Ok(metadata) => metadata,
                Err(_) => match serde_json::from_slice::<Vec<Metadata>>(&std::fs::read(&metadata)?)
                {
                    Ok(metadata) => metadata,
                    Err(_) => Err(anyhow::anyhow!(
                        "Failed to read `metadata` as json or path to json file"
                    ))?,
                },
            };

            metadata::add_coins_meta_to_genesis(
                home,
                metadata,
                dedup_input,
                overwrite_dup,
                dry,
                in_genesis_path,
            )
        }
    }
}
