use std::{net::SocketAddr, path::PathBuf};

use gears::{
    application::handlers::client::TxHandler,
    commands::client::tx::{ClientTxContext, TxCommand},
    crypto::{ed25519::Ed25519PubKey, public::PublicKey},
    types::{
        base::{coin::UnsignedCoin, coins::UnsignedCoins},
        decimal256::Decimal256,
        tx::Messages,
        uint::Uint256,
    },
};
use staking::{CommissionRates, CreateValidator, Description};

use crate::utils::{parse_staking_params_from_genesis, GenesisAccounts, GenesisBalanceIter};

use gears::tendermint::types::proto::crypto::PublicKey as TendermintPublicKey;

#[derive(Debug, Clone)]
pub struct GentxCmd {
    pub pubkey: Option<TendermintPublicKey>,
    pub amount: UnsignedCoin,
    pub moniker: String,
    pub identity: String,
    pub website: String,
    pub security_contact: String,
    pub details: String,
    pub commission_rate: Decimal256,
    pub commission_max_rate: Decimal256,
    pub commission_max_change_rate: Decimal256,
    pub min_self_delegation: Uint256,

    pub output: Option<PathBuf>,
    pub ip: SocketAddr,
    pub node_id: Option<String>,
}

pub fn gentx_cmd(
    cmd: TxCommand<GentxCmd>,
    balance_sk: &'static str,
    staking_sk: &'static str,
    auth_sk: &'static str,
) -> anyhow::Result<()> {
    let gentx_handler =
        GentxTxHandler::new(cmd.inner.output.clone(), balance_sk, staking_sk, auth_sk)?;

    gears::commands::client::tx::run_tx(cmd, &gentx_handler)?;

    Ok(())
}

#[derive(Debug, Clone)]
struct GentxTxHandler {
    output_dir: Option<PathBuf>,
    pub balance_sk: &'static str,
    pub staking_sk: &'static str,
    pub auth_sk: &'static str,
}

impl GentxTxHandler {
    pub fn new(
        output_dir: Option<PathBuf>,
        balance_sk: &'static str,
        staking_sk: &'static str,
        auth_sk: &'static str,
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
                    auth_sk,
                })
            }
            None => Ok(Self {
                output_dir: None,
                balance_sk,
                staking_sk,
                auth_sk,
            }),
        }
    }
}

impl TxHandler for GentxTxHandler {
    type Message = CreateValidator;

    type TxCommands = GentxCmd;

    fn prepare_tx(
        &self,
        client_tx_context: &mut ClientTxContext,
        Self::TxCommands {
            pubkey,
            amount,
            moniker,
            identity,
            website,
            security_contact,
            details,
            commission_rate,
            commission_max_rate,
            commission_max_change_rate,
            min_self_delegation,
            output: _,
            ip,
            node_id,
        }: Self::TxCommands,
        from_address: gears::types::address::AccAddress,
    ) -> anyhow::Result<gears::types::tx::Messages<Self::Message>> {
        let coins = UnsignedCoins::new([amount.clone()]).expect("hardcoded coin"); // I don't want to comment this code. See: https://github.com/cosmos/cosmos-sdk/blob/d3f09c222243bb3da3464969f0366330dcb977a8/x/genutil/client/cli/gentx.go#L118-L147

        // check that the provided account has a sufficient balance in the set of genesis accounts.
        let txs_iter = GenesisBalanceIter::new(
            self.balance_sk,
            client_tx_context.home.join("config/genesis.json"), // todo: better way to get path to genesis file
        )?
        .into_inner();

        match txs_iter.get(&from_address) {
            Some(acc_coins) => {
                let staking_params = parse_staking_params_from_genesis(
                    self.staking_sk,
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

        let pub_key = match pubkey {
            Some(var) => PublicKey::from(var),
            None => {
                #[derive(serde::Deserialize)]
                struct NodeKeyJson {
                    pub priv_key: gears::tendermint::crypto::PrivateKey,
                }

                let key: NodeKeyJson = serde_json::from_reader(std::fs::File::open(
                    client_tx_context.home.join("config/node_key.json"),
                )?)?;

                

                match key.priv_key {
                    gears::tendermint::crypto::PrivateKey::Ed25519(var) => PublicKey::Ed25519(
                        Ed25519PubKey::try_from(var.verification_key().as_bytes().to_vec())?,
                    ),
                    _ => unreachable!(),
                }
            }
        };

        client_tx_context.memo = Some(format!(
            "{}@{ip}",
            node_id.unwrap_or(pub_key.get_address().as_hex())
        ));

        let tx = Messages::from(CreateValidator {
            description: Description {
                moniker,
                identity,
                website,
                security_contact,
                details,
            },
            commission: CommissionRates::new(
                commission_rate,
                commission_max_rate,
                commission_max_change_rate,
            )?,
            min_self_delegation,
            delegator_address: from_address.clone(),
            validator_address: from_address.into(),
            pubkey: pub_key.into(),
            value: amount,
        });

        Ok(tx)
    }

    fn account(
        &self,
        address: gears::types::address::AccAddress,
        client_tx_context: &mut ClientTxContext,
    ) -> anyhow::Result<Option<gears::types::account::Account>> {
        GenesisAccounts::new(
            self.auth_sk,
            client_tx_context.home.join("config/genesis.json"),
        )
        .map(|this| this.into_inner().remove(&address))
    }

    fn handle_tx(
        &self,
        mut tx: gears::types::tx::Tx<Self::Message>,
        client_tx_context: &mut ClientTxContext,
    ) -> anyhow::Result<gears::application::handlers::client::TxExecutionResult> {
        if let Some(memo) = client_tx_context.memo.take() {
            tx.body.memo = memo;
        }

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
