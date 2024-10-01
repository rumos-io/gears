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
    pub ignore_dup: bool,
    pub overwrite_same: bool,
}

pub fn add_coins_meta_to_genesis(
    home: impl AsRef<Path>,
    metadata: impl IntoIterator<Item = Metadata>,
    dedup_input: bool,
    ignore_dup: bool,
    overwrite_same: bool,
) -> anyhow::Result<()> {
    let metadata = {
        let mut metadata = metadata.into_iter().collect::<Vec<_>>();
        let pre_dup_len = metadata.len();

        metadata.dedup();

        if !dedup_input && ((pre_dup_len != metadata.len()) == true) {
            Err(anyhow::anyhow!("Found duplicates in new list"))?
        }

        metadata
    };

    let genesis_path = ConfigDirectory::GenesisFile.path_from_hone(&home);

    let mut genesis = serde_json::from_slice::<serde_json::Value>(&std::fs::read(&genesis_path)?)?;

    let value = genesis
        .get_mut("app_state")
        .ok_or(anyhow::anyhow!("missing `app_state`"))?
        .get_mut("bank")
        .ok_or(anyhow::anyhow!("`bank` module is not found"))?
        .get_mut("denom_metadata")
        .ok_or(anyhow::anyhow!("`denom_metadata` is not found"))?;

    let owned_value = value.take();

    let mut original_meta = serde_json::from_value::<Vec<Metadata>>(owned_value)?
        .into_iter()
        .map(|this| (this.name.clone(), this))
        .collect::<HashMap<_, _>>();

    for meta in metadata {
        let dup = original_meta.get(&meta.name);

        match dup {
            Some(dup) => {
                if !ignore_dup {
                    Err(anyhow::anyhow!("Duplicate meta: {}", dup.name))?
                }

                if !overwrite_same && ((dup == &meta) == true) {
                    Err(anyhow::anyhow!(
                        "Similar meta with name: {}\nNew: {:#?}\nOriginal: {:#?}",
                        dup.name,
                        meta,
                        dup
                    ))?
                } else {
                    original_meta.insert(meta.name.clone(), meta);
                }
            }
            None => {
                original_meta.insert(meta.name.clone(), meta);
            }
        }
    }

    *value = serde_json::to_value(
        original_meta
            .into_iter()
            .map(|(_, this)| this)
            .collect::<Vec<_>>(),
    )
    .expect("serde encoding");

    std::fs::write(
        genesis_path,
        serde_json::to_string_pretty(value).expect("serde encoding"),
    )?;

    Ok(())
}
