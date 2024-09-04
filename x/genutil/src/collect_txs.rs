use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use gears::types::{address::AccAddress, base::coins::UnsignedCoins, tx::Tx};
use staking::CreateValidator;

use crate::{errors::SERDE_JSON_CONVERSION, genesis::GenutilGenesis, utils::GenesisBalanceIter};

#[derive(Debug, Clone)]
pub enum CollectMode {
    File(bool),
    Display,
}

#[derive(Debug, Clone)]
pub struct CollectGentxCmd {
    pub(crate) gentx_dir: PathBuf,
    pub(crate) home: PathBuf,
    pub mode: CollectMode,
}

pub fn gen_app_state_from_config(
    CollectGentxCmd {
        gentx_dir,
        home,
        mode,
    }: CollectGentxCmd,
    balance_sk: &str,
    genutil: &str,
) -> anyhow::Result<()> {
    let genesis_file = home.join("config/genesis.json");

    let txs_iter = GenesisBalanceIter::new(balance_sk, &genesis_file)?; // todo: better way to get path to genesis file

    let (persistent_peers, app_gen_txs) =
        collect_txs(gentx_dir, read_moniker_cfg(&home)?, txs_iter)?;

    if app_gen_txs.is_empty() {
        return Err(anyhow::anyhow!("there must be at least one genesis tx"));
    }

    let mut genesis: serde_json::Value =
        serde_json::from_reader(std::fs::File::open(&genesis_file)?)?;

    let mut existed_gen_txs = match genesis.pointer_mut("genutil/gen_txs") {
        Some(val) => serde_json::from_value(val.take()).expect(SERDE_JSON_CONVERSION),
        None => GenutilGenesis::default(),
    };

    for tx in app_gen_txs {
        if existed_gen_txs.gen_txs.contains(&tx) {
            continue;
        }

        existed_gen_txs.gen_txs.push(tx)
    }

    match genesis
        .pointer_mut("/app_state")
        .ok_or(anyhow::anyhow!("Failed to read `app_state` from genesis"))?
        .pointer_mut(&format!("/{genutil}"))
    {
        Some(genesis) => match genesis.pointer_mut("/gen_txs") {
            Some(genesis) => {
                *genesis =
                    serde_json::to_value(existed_gen_txs.gen_txs).expect(SERDE_JSON_CONVERSION)
            }
            None => {
                let _ = genesis
                    .as_object_mut()
                    .ok_or(anyhow::anyhow!(
                        "Failed to read `gen_txs` as object. Probably invalid genesis file"
                    ))?
                    .insert(
                        "gen_txs".to_owned(),
                        serde_json::to_value(existed_gen_txs.gen_txs).expect(SERDE_JSON_CONVERSION),
                    );
            }
        },
        None => {
            let _ = genesis
                .as_object_mut()
                .ok_or(anyhow::anyhow!(
                    "Failed to read `{genutil}` as object. Probably invalid genesis file"
                ))?
                .insert(
                    genutil.to_owned(),
                    serde_json::to_value(existed_gen_txs).expect(SERDE_JSON_CONVERSION),
                );
        }
    }

    match mode {
        CollectMode::File(backup) => {
            let config_file = home.join("config/config.toml");

            if backup {
                std::fs::copy(&genesis_file, home.join("config/genesis.old.json"))?;
                std::fs::copy(&config_file, home.join("config/config.old.toml"))?;
            }

            let genesis_output =
                serde_json::to_string_pretty(&genesis).expect(SERDE_JSON_CONVERSION);
            let config_output = add_peers_to_tm_toml_config(&home, persistent_peers)?;

            std::fs::write(&genesis_file, genesis_output)?;
            std::fs::write(&config_file, config_output.to_string())?;
        }
        CollectMode::Display => {
            let genesis_output =
                serde_json::to_string_pretty(&genesis).expect(SERDE_JSON_CONVERSION);

            println!("# genesis.json\n{}", genesis_output);

            let config_output = add_peers_to_tm_toml_config(&home, persistent_peers)?;

            println!("# config.toml\n{}", config_output.to_string());
        }
    }

    Ok(())
}

fn read_moniker_cfg(home: impl AsRef<Path>) -> anyhow::Result<String> {
    let tendermint_config: toml_edit::DocumentMut =
        std::fs::read_to_string(home.as_ref().join("config/config.toml"))?.parse()?;

    let moniker = tendermint_config
        .get("moniker")
        .ok_or(anyhow::anyhow!(
            "Failed to find `moniker` in tendermint config"
        ))?
        .as_str()
        .ok_or(anyhow::anyhow!(
            "Failed to read `moniker` in tendermint config"
        ))?
        .to_owned();

    Ok(moniker)
}

fn add_peers_to_tm_toml_config(
    home: impl AsRef<Path>,
    peers: Peers,
) -> anyhow::Result<toml_edit::DocumentMut> {
    let mut tendermint_config: toml_edit::DocumentMut =
        std::fs::read_to_string(home.as_ref().join("config/config.toml"))?.parse()?;

    let peers_str = peers.to_string();

    match tendermint_config.get_mut("p2p") {
        Some(config) => match config.get_mut("persistent_peers") {
            Some(peers) => {
                *peers = toml_edit::value(peers_str);
            }
            None => {
                config
                    .as_table_mut()
                    .ok_or(anyhow::anyhow!(
                        "invalid config. Can't read `[p2p]` as table"
                    ))?
                    .insert("persistent_peers", toml_edit::value(peers_str));
            }
        },
        None => {
            let mut table = toml_edit::Table::default();
            table["persistent_peers"] = toml_edit::value(peers_str);

            tendermint_config
                .as_table_mut()
                .insert("p2p", toml_edit::Item::Table(table));
        }
    }

    Ok(tendermint_config)
}

fn collect_txs(
    dir: impl AsRef<Path>,
    moniker: String,
    balance: impl IntoIterator<Item = (AccAddress, UnsignedCoins)>,
) -> anyhow::Result<(Peers, Vec<Tx<CreateValidator>>)> {
    let balance = balance.into_iter().collect::<HashMap<_, _>>();

    let items = if dir.as_ref().is_dir() {
        let mut entities = Vec::new();

        for dir in dir.as_ref().read_dir()? {
            let dir = dir?;

            if dir.file_type()?.is_dir() {
                continue;
            }

            if !dir.file_name().to_string_lossy().ends_with(".json") {
                continue;
            }

            let file_content = std::fs::read_to_string(dir.path())?;
            let tx: Tx<CreateValidator> = serde_json::from_str(&file_content)?;

            entities.push(tx);
        }

        entities
    } else {
        let file_content = std::fs::read_to_string(dir.as_ref())?;
        let tx: Tx<CreateValidator> = serde_json::from_str(&file_content)?;

        vec![tx]
    };

    let mut addresses_ip = Vec::with_capacity(items.len());
    for tx in &items {
        let msg = tx.get_msgs();
        if msg.len() != 1 {
            Err(anyhow::anyhow!(
                "genesis transactions must be single-message"
            ))?
        }

        // the memo flag is used to store
        // the ip and node-id, for example this may be:
        // "528fd3df22b31f4969b05652bfe8f0fe921321d5@192.168.2.37:26656"

        let node_id_addr = tx.get_memo();
        if node_id_addr.is_empty() {
            Err(anyhow::anyhow!(
                "failed to find node's address and IP due empty memo"
            ))?
        }

        let msg = msg.first();
        let delegator_balance = balance
            .get(&msg.delegator_address)
            .ok_or(anyhow::anyhow!("account balance not in genesis state"))?;

        let _ = balance
            .get(&msg.validator_address.clone().into())
            .ok_or(anyhow::anyhow!("account balance not in genesis state"))?;

        if delegator_balance.amount_of(&msg.value.denom) < msg.value.amount {
            Err(anyhow::anyhow!("insufficient fund for delegation"))?
        }

        // exclude itself from persistent peers
        if msg.description.moniker != moniker {
            addresses_ip.push(node_id_addr.to_owned())
        }
    }

    addresses_ip.sort();

    Ok((Peers(addresses_ip), items))
}

#[derive(Debug, Clone)]
pub struct Peers(pub Vec<String>);

impl std::fmt::Display for Peers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join(","))
    }
}
