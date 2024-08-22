use std::path::PathBuf;

use gears::{
    application::handlers::client::TxHandler, commands::client::tx::TxCommand, store::StoreKey,
    types::base::coins::UnsignedCoins,
};
use staking::CreateValidator;

use crate::utils::{parse_staking_params_from_genesis, GenesisBalanceIter};

#[derive(Debug, Clone)]
pub struct GentxCmd {
    pub coins: UnsignedCoins,
    pub output: Option<PathBuf>,
}

pub fn gentx_cmd<SK: StoreKey>(
    cmd: TxCommand<GentxCmd>,
    balance_sk: SK,
    staking_sk: SK,
) -> anyhow::Result<()> {
    let gentx_handler = GentxTxHandler::new(cmd.inner.output.clone(), balance_sk, staking_sk)?;

    gears::commands::client::tx::run_tx(cmd, &gentx_handler)?;

    Ok(())
}

#[derive(Debug, Clone)]
struct GentxTxHandler<SK> {
    output_dir: Option<PathBuf>,
    pub balance_sk: SK,
    pub staking_sk: SK,
}

impl<SK> GentxTxHandler<SK> {
    pub fn new(
        output_dir: Option<PathBuf>,
        balance_sk: SK,
        staking_sk: SK,
    ) -> anyhow::Result<Self> {
        match output_dir {
            Some(output_dir) => {
                if output_dir.exists() && !output_dir.is_dir() {
                    Err(anyhow::anyhow!("use directory"))?
                }

                std::fs::create_dir(&output_dir)?;

                Ok(Self {
                    output_dir: Some(output_dir),
                    balance_sk,
                    staking_sk,
                })
            }
            None => Ok(Self {
                output_dir: None,
                balance_sk,
                staking_sk,
            }),
        }
    }
}

impl<SK: StoreKey> TxHandler for GentxTxHandler<SK> {
    type Message = CreateValidator;

    type TxCommands = GentxCmd;

    fn prepare_tx(
        &self,
        client_tx_context: &gears::commands::client::tx::ClientTxContext,
        Self::TxCommands { coins, output: _ }: Self::TxCommands,
        from_address: gears::types::address::AccAddress,
    ) -> anyhow::Result<gears::types::tx::Messages<Self::Message>> {
        // check that the provided account has a sufficient balance in the set of genesis accounts.
        let txs_iter = GenesisBalanceIter::new(
            &self.balance_sk,
            client_tx_context.home.join("config/genesis.json"), // todo: better way to get path to genesis file
        )?
        .into_inner();

        match txs_iter.get(&from_address) {
            Some(acc_coins) => {
                let staking_params = parse_staking_params_from_genesis(
                    &self.staking_sk,
                    "params",
                    client_tx_context.home.join("config/genesis.json"),
                )?;

                let bond_denom = staking_params.bond_denom();

                if coins.amount_of(bond_denom) > acc_coins.amount_of(bond_denom) {
                    Err(anyhow::anyhow!("account {from_address} has a balance in genesis, but it only has {}{bond_denom} available to stake, not {}{bond_denom}", 
                    acc_coins.amount_of(bond_denom), coins.amount_of(bond_denom) ))?
                }
            }
            None => Err(anyhow::anyhow!(
                "account {from_address} does not have a balance in the genesis state"
            ))?,
        }

        // command
        //     .coins
        //     .clone()
        //     .try_into_cmd(from_address)
        //     .map(Into::into)

        todo!()
    }

    fn handle_tx(
        &self,
        tx: gears::types::tx::Tx<Self::Message>,
        _node: url::Url,
    ) -> anyhow::Result<gears::application::handlers::client::TxExecutionResult> {
        let tx_str = serde_json::to_string_pretty(&tx)?;
        match self.output_dir.clone() {
            Some(dir) => {
                let output = dir.join("gentx.json");
                std::fs::write(&output, tx_str)?;
                Ok(gears::application::handlers::client::TxExecutionResult::File(output))
            }
            None => {
                println!("{tx_str}");

                Ok(gears::application::handlers::client::TxExecutionResult::None)
            }
        }
    }
}
