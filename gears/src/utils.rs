use std::{path::Path, process::Child, str::FromStr};

use crate::{
    baseapp::genesis::Genesis,
    commands::node::init::{init, InitCommand},
    types::address::AccAddress,
};
use anyhow::anyhow;
pub use assert_fs::TempDir;

use rand::{prelude::Distribution, rngs::ThreadRng};
use run_script::{IoOptions, ScriptOptions};
use tendermint::types::chain_id::ChainId;

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
    pub fn run_tendermint<G: Genesis, AC: crate::config::ApplicationConfig>(
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

        let opt: InitCommand = InitCommand::former()
            .home(tmp_dir.to_path_buf())
            .chain_id(ChainId::from_str("test-chain")?)
            .moniker("test".to_owned())
            .form();

        init::<_, AC>(opt, genesis)?;

        let (_code, _output, _error) = run_script::run(
            r#"
                tar -xf tendermint.tar.gz
                "#,
            &vec![],
            &options,
        )?; // TODO: make it work for windows too?

        let script = format!(
            "./tendermint start --home {}",
            tmp_dir
                .to_str()
                .ok_or(anyhow!("failed to get path to tmp folder"))?
        );

        let child = run_script::spawn(&script, &vec![], &options)?;

        Ok(Self(child, tmp_dir))
    }
}

pub fn random_address() -> AccAddress {
    rand::distributions::Alphanumeric
        .sample_iter(&mut ThreadRng::default())
        .take(100)
        .collect::<Vec<u8>>()
        .try_into()
        .expect("generated range is smaller than 255")
}
