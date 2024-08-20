use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use gears::{
    store::StoreKey,
    types::{address::AccAddress, base::coins::UnsignedCoins, tx::Tx},
};
use serde::{Deserialize, Serialize};
use staking::CreateValidator;

use crate::{balances_iter::GenesisBalanceIter, errors::SERDE_JSON_CONVERSION};

#[derive(Debug, Clone)]
pub struct CollectGentxCmd {
    pub(crate) gentx_dir: PathBuf,
    pub(crate) home: PathBuf,
    pub moniker: String,
}

pub fn gen_app_state_from_config<SK: StoreKey>(
    CollectGentxCmd {
        gentx_dir,
        home,
        moniker,
    }: CollectGentxCmd,
    balance_sk: &SK,
    genutil_sk: &SK,
) -> anyhow::Result<(Peers, String)> {
    let txs_iter = GenesisBalanceIter::new(balance_sk, home.join("config/genesis.json"))?; // todo: better way to get path to genesis file

    let (persistent_peers, app_gen_txs) = collect_txs(gentx_dir, moniker, txs_iter)?;

    if app_gen_txs.is_empty() {
        return Err(anyhow::anyhow!("there must be at least one genesis tx"));
    }

    let mut genesis: serde_json::Value =
        serde_json::from_reader(std::fs::File::open(home.join("config/genesis.json"))?)?;

    #[derive(Serialize, Deserialize, Default)]
    struct GenutilGenesis {
        gen_txs: Vec<Tx<CreateValidator>>,
    }

    let genesis_unparsed = genesis
        .as_object_mut()
        .ok_or(anyhow::anyhow!("failed to read json as object"))?;

    let mut existed_gen_txs = match genesis_unparsed.get_mut(genutil_sk.name()) {
        Some(genesis) => serde_json::from_value(genesis.take()).expect(SERDE_JSON_CONVERSION),
        None => GenutilGenesis::default(),
    };

    for tx in app_gen_txs {
        if existed_gen_txs.gen_txs.contains(&tx) {
            continue;
        }

        existed_gen_txs.gen_txs.push(tx)
    }

    let _ = genesis_unparsed.insert(
        genutil_sk.name().to_owned(),
        serde_json::to_value(existed_gen_txs).expect(SERDE_JSON_CONVERSION),
    );

    Ok((
        persistent_peers,
        serde_json::to_string_pretty(genesis_unparsed).expect(SERDE_JSON_CONVERSION),
    ))
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

            if !Path::new(&dir.file_name()).ends_with(".json") {
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
