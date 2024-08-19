use std::{collections::HashMap, path::Path};

use gears::types::{address::AccAddress, base::coins::UnsignedCoins, tx::Tx};
use staking::CreateValidator;

pub fn collect_txs(
    dir: impl AsRef<Path>,
    moniker: String,
    balance: impl IntoIterator<Item = (AccAddress, UnsignedCoins)>,
) -> anyhow::Result<(String, Vec<Tx<CreateValidator>>)> {
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
            addresses_ip.push(format!("{node_id_addr},"))
        }
    }

    addresses_ip.sort();

    let lenght = addresses_ip
        .iter()
        .fold(0, |accumulator, this| accumulator + this.len());

    let peers =
        addresses_ip
            .into_iter()
            .fold(String::with_capacity(lenght), |mut accumulator, this| {
                accumulator.extend(this.chars());
                accumulator
            });

    Ok((peers, items))
}
