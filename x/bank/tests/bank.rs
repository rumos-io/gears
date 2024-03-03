use gears::baseapp::Genesis;
use serde::{Deserialize, Serialize};
use utils::testing::TmpChild;

#[test]
fn test() -> anyhow::Result<()>
{
    let _tendermint = TmpChild::start_tendermint::<_, MockConfig>("./tests/assets", &GenesisMock::default() )?;

    Ok(())
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct GenesisMock( pub bank::GenesisState );

impl Genesis for GenesisMock {
    fn add_genesis_account(
        &mut self,
        address: proto_types::AccAddress,
        coins: proto_messages::cosmos::base::v1beta1::SendCoins,
    ) -> Result<(), gears::error::AppError> {
        Ok(self.0.add_genesis_account(address, coins) )
    }
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct MockConfig {
    pub example: u32,
}

