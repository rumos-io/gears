use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use gears::{config::ConfigDirectory, types::tx::metadata::Metadata};

#[derive(Debug, Clone)]
pub struct CoinsMetaGenesisCmd {
    pub home: PathBuf,
    pub metadata: String,
    pub dedup_input: bool,
    pub overwrite_dup: bool,
    pub dry: bool,
}

pub fn add_coins_meta_to_genesis(
    home: impl AsRef<Path>,
    metadata: impl IntoIterator<Item = Metadata>,
    dedup_input: bool,
    overwrite_dup: bool,
    dry: bool,
    in_genesis_path: &str,
) -> anyhow::Result<()> {
    let metadata = {
        let mut metadata = metadata.into_iter().collect::<Vec<_>>();
        let pre_dup_len = metadata.len();

        metadata.dedup();

        if !dedup_input && (pre_dup_len != metadata.len()) {
            Err(anyhow::anyhow!("Found duplicates in new list"))?
        }

        metadata
    };

    let genesis_path = ConfigDirectory::GenesisFile.path_from_home(&home);

    let mut genesis = serde_json::from_slice::<serde_json::Value>(&std::fs::read(&genesis_path)?)?;

    let value = genesis
        .pointer_mut(in_genesis_path)
        .ok_or(anyhow::anyhow!(
            "`{in_genesis_path} not found. Check is genesis file is valid"
        ))?
        .take();

    let mut original_meta = serde_json::from_value::<Vec<Metadata>>(value)?
        .into_iter()
        .map(|this| (this.name.clone(), this))
        .collect::<HashMap<_, _>>();

    for meta in metadata {
        let dup = original_meta.get(&meta.name);

        match dup {
            Some(dup) => {
                if overwrite_dup {
                    original_meta.insert(meta.name.clone(), meta);
                } else {
                    Err(anyhow::anyhow!(
                        "Duplicate meta with name: {}\nNew: {}\nOriginal: {}",
                        dup.name,
                        serde_json::to_string_pretty(&meta).expect("serde encoding"),
                        serde_json::to_string_pretty(&dup).expect("serde encoding"),
                    ))?
                }
            }
            None => {
                original_meta.insert(meta.name.clone(), meta);
            }
        }
    }

    if !dry {
        *genesis
            .pointer_mut(in_genesis_path)
            .expect("we checked that this exists") =
            serde_json::to_value(original_meta.into_values().collect::<Vec<_>>())
                .expect("serde encoding");

        std::fs::write(
            genesis_path,
            serde_json::to_string_pretty(&genesis).expect("serde encoding"),
        )?;
    }

    Ok(())
}
