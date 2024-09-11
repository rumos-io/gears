use std::{net::SocketAddr, path::PathBuf};

use gears::{
    application::handlers::client::{NodeFetcher, TxHandler},
    commands::client::tx::{AccountProvider, ClientTxContext, TxCommand},
    crypto::public::PublicKey,
    types::{
        account::{Account, BaseAccount},
        base::{coin::UnsignedCoin, coins::UnsignedCoins},
        decimal256::Decimal256,
        tx::Messages,
        uint::Uint256,
    },
};
use staking::{CommissionRates, CreateValidator, Description};

use crate::utils::{parse_staking_params_from_genesis, GenesisBalanceIter};

use gears::tendermint::types::proto::crypto::PublicKey as TendermintPublicKey;

#[derive(Debug, Clone)]
pub struct GentxCmd {
    pub pubkey: Option<TendermintPublicKey>,
    pub amount: UnsignedCoin,
    pub moniker: Option<String>,
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

pub fn gentx_cmd<F: NodeFetcher + Clone>(
    cmd: TxCommand<GentxCmd>,
    balance_sk: &'static str,
    staking_sk: &'static str,
    fetcher: &F,
) -> anyhow::Result<()> {
    let gentx_handler = GentxTxHandler::new(cmd.inner.output.clone(), balance_sk, staking_sk)?;

    gears::commands::client::tx::run_tx(cmd, &gentx_handler, fetcher)?;

    Ok(())
}

#[derive(Debug, Clone)]
struct GentxTxHandler {
    output_dir: Option<PathBuf>,
    pub balance_sk: &'static str,
    pub staking_sk: &'static str,
}

impl GentxTxHandler {
    pub fn new(
        output_dir: Option<PathBuf>,
        balance_sk: &'static str,
        staking_sk: &'static str,
    ) -> anyhow::Result<Self> {
        match output_dir {
            Some(output_dir) => {
                if output_dir.exists() && !output_dir.is_dir() {
                    Err(anyhow::anyhow!("use directory"))?
                }

                if !output_dir.exists() {
                    std::fs::create_dir(&output_dir)?;
                }

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

impl TxHandler for GentxTxHandler {
    type Message = CreateValidator;

    type TxCommands = GentxCmd;

    fn account<F: NodeFetcher>(
        &self,
        address: gears::types::address::AccAddress,
        client_tx_context: &mut ClientTxContext,
        _fetcher: &F,
    ) -> anyhow::Result<Option<gears::types::account::Account>> {
        match client_tx_context.account {
                AccountProvider::Offline {
                    sequence,
                    account_number, //TODO: account number should always be 0 see https://github.com/cosmos/cosmos-sdk/blob/2582f0aab7b2cbf66ade066fe570a4622cf0b098/x/auth/ante/sigverify.go#L272
                } => Ok(Some(Account::Base(BaseAccount {
                    address,
                    pub_key: None,
                    account_number,
                    sequence,
                }))),
                AccountProvider::Online => {
                   Err(anyhow::anyhow!("Can't use online mode for gentx account. You need to specify `account-number` and `sequence`"))
                }
            }
    }

    fn prepare_tx(
        &self,
        client_tx_context: &mut ClientTxContext,
        Self::TxCommands {
            pubkey: pub_key,
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
        pubkey: PublicKey, // TODO: this should be a TendermintPublicKey
    ) -> anyhow::Result<gears::types::tx::Messages<Self::Message>> {
        let coins = UnsignedCoins::new([amount.clone()]).expect("hardcoded coin"); // I don't want to comment this code. See: https://github.com/cosmos/cosmos-sdk/blob/d3f09c222243bb3da3464969f0366330dcb977a8/x/genutil/client/cli/gentx.go#L118-L147

        // check that the provided account has a sufficient balance in the set of genesis accounts.
        let txs_iter = GenesisBalanceIter::new(
            self.balance_sk,
            client_tx_context.home.join("config/genesis.json"), // todo: better way to get path to genesis file
        )?
        .into_inner();

        let from_address = pubkey.get_address();

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

        let pub_key = match pub_key {
            Some(var) => PublicKey::from(var),
            None => {
                let file = std::fs::File::open(
                    client_tx_context
                        .home
                        .join("config/priv_validator_key.json"), // TODO: delegate this to tendermint crate
                )?;
                tendermint::get_validator_pub_key(file).unwrap().into()
            }
        };

        client_tx_context.memo = Some(format!(
            "{}@{ip}",
            node_id.unwrap_or(pub_key.get_address().as_hex())
        ));

        let tx = Messages::from(CreateValidator {
            description: Description {
                moniker: moniker.unwrap_or_default(),
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

    fn handle_tx(
        &self,
        mut tx: gears::types::tx::Tx<Self::Message>,
        client_tx_context: &mut ClientTxContext,
    ) -> anyhow::Result<gears::application::handlers::client::TxExecutionResult> {
        if let Some(memo) = client_tx_context.memo.take() {
            tx.body.memo = memo;
        }

        let addr = &tx.body.messages.first().delegator_address;

        let tx_str = serde_json::to_string_pretty(&tx)?;
        match self.output_dir.clone() {
            Some(dir) => {
                let output = dir.join(format!("gentx-{}.json", addr.as_hex()));
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
