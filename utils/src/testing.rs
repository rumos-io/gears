use std::{path::Path, process::Child, str::FromStr};

use anyhow::anyhow;
pub use assert_fs::TempDir;
use gears::{baseapp::Genesis, client::genesis_account::GenesisCommand};
use proto_messages::cosmos::base::v1beta1::{Coin, SendCoins};
use proto_types::{AccAddress, Denom};
use run_script::{IoOptions, ScriptOptions};

/// Struct for process which lauched from tmp dir
#[derive(Debug)]
pub struct TmpChild(pub Child, pub TempDir);

impl Drop for TmpChild {
    fn drop(&mut self) {
        // Stop child process before deletion of tmp dir
        while let Err(_) = self.0.kill() {
            std::thread::sleep(std::time::Duration::from_millis(100))
        }
    }
}

impl TmpChild {
    pub fn run_tendermint<G: Genesis, AC: gears::config::ApplicationConfig>(
        tmp_dir: TempDir,
        path_to_tendermint: &(impl AsRef<Path> + ?Sized),
        genesis: &G,
    ) -> anyhow::Result<Self> {
        dircpy::CopyBuilder::new(path_to_tendermint, &tmp_dir)
            .overwrite(true)
            .run()?;

        let options = ScriptOptions {
            runner: None,
            runner_args: None,
            working_directory: Some(tmp_dir.to_path_buf()),
            input_redirection: IoOptions::Inherit,
            output_redirection: IoOptions::Pipe,
            exit_on_error: false,
            print_commands: false,
            env_vars: None,
        };

        let opt: gears::client::init::InitCommand =
            gears::client::init::InitCommandBuilder::default()
                .home(tmp_dir.to_path_buf())
                .chain_id(tendermint::informal::chain::Id::from_str("test-chain")?)
                .moniker("test".to_owned())
                .build()?;

        gears::client::init::init::<_, AC>(opt, genesis)?;

        let genesis_account_cmd = GenesisCommand {
            home: tmp_dir.to_path_buf(),
            address: AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux")
                .expect("Default account should be valid"),
            coins: SendCoins::new(vec![Coin {
                denom: Denom::from_str("uatom").expect("default denom should be valid"),
                amount: 34_u32.into(),
            }])
            .expect("not empty"),
        };

        gears::client::genesis_account::genesis_account_add::<G>(genesis_account_cmd)?;

        let (_code, _output, _error) = run_script::run(
            r#"
                tar -xf tendermint.tar.gz
                "#,
            &vec![],
            &options,
        )?; // TODO: make it work for windows too?

        let child = run_script::spawn(
            "./tendermint start",
            &vec![
                "--home".to_owned(),
                tmp_dir
                    .to_str()
                    .ok_or(anyhow!("failed to get path to tmp folder"))?
                    .to_owned(),
            ],
            &options,
        )?;

        Ok(Self(child, tmp_dir))
    }

    pub fn start_tendermint(
        path_to_tendermint: &(impl AsRef<Path> + ?Sized),
    ) -> anyhow::Result<Self> {
        let tmp_dir = TempDir::new()?;

        dircpy::CopyBuilder::new(path_to_tendermint, &tmp_dir)
            .overwrite(true)
            .run()?;

        let options = ScriptOptions {
            runner: None,
            runner_args: None,
            working_directory: Some(tmp_dir.to_path_buf()),
            input_redirection: IoOptions::Inherit,
            output_redirection: IoOptions::Pipe,
            exit_on_error: false,
            print_commands: false,
            env_vars: None,
        };

        let (_code, _output, _error) = run_script::run(
            r#"
            tar -xf tendermint.tar.gz
            "#,
            &vec![],
            &options,
        )?; // TODO: make it work for windows too?

        let child = run_script::spawn(
            "./tendermint start",
            &vec![
                "--home".to_owned(),
                tmp_dir
                    .to_str()
                    .ok_or(anyhow!("failed to get path to tmp folder"))?
                    .to_owned(),
            ],
            &options,
        )?;

        Ok(Self(child, tmp_dir))
    }
}
