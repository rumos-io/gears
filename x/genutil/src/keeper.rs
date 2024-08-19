use std::path::Path;

use gears::types::tx::Tx;
use staking::CreateValidator;

pub fn collect_txs(dir: impl AsRef<Path>) -> anyhow::Result<()> {
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

    for tx in items {
        if tx.get_msgs().len() != 1 {
            Err(anyhow::anyhow!(
                "genesis transactions must be single-message"
            ))?
        }

        let node_id_addr = tx.get_memo();
        if node_id_addr.is_empty() {
            Err(anyhow::anyhow!(
                "failed to find node's address and IP due empty memo"
            ))?
        }
    }

    Ok(())
}
