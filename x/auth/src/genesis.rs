use gears::{error::AppError, x::auth::Params};
use proto_messages::cosmos::auth::v1beta1::BaseAccount;
use proto_types::AccAddress;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenesisState {
    pub accounts: Vec<BaseAccount>,
    pub params: Params,
}

impl Default for GenesisState {
    fn default() -> Self {
        Self {
            accounts: vec![],
            params: Params {
                max_memo_characters: 256,
                tx_sig_limit: 7,
                tx_size_cost_per_byte: 10,
                sig_verify_cost_ed25519: 590,
                sig_verify_cost_secp256k1: 1000,
            },
        }
    }
}

impl GenesisState {
    pub fn add_genesis_account(&mut self, address: AccAddress) -> Result<(), AppError> {
        let mut contains = false;
        for acct in &self.accounts {
            if acct.address == address {
                contains = true;
                break;
            }
        }

        if !contains {
            self.accounts.push(BaseAccount {
                address,
                pub_key: None,
                account_number: 0, // This is ignored when initializing from genesis
                sequence: 0,       //TODO: make a BaseAccount constructor
            });
            Ok(())
        } else {
            Err(AppError::Genesis(format!(
                "cannot add account at existing address {}",
                address
            )))
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn add_genesis_account_works() {
        let mut genesis_state = GenesisState::default();
        let address = "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux"
            .parse()
            .unwrap();
        genesis_state
            .add_genesis_account(address)
            .expect("will succeed because address is not in genesis_state.accounts");

        assert_eq!(genesis_state.accounts.len(), 1);
        assert!(matches!(
                &genesis_state.accounts[0],
                BaseAccount {
                    address,
                    pub_key: None,
                    account_number: 0,
                    sequence: 0,
                }
             if address == &AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux").unwrap()),);
    }
}
