use std::{path::Path, process::Child, str::FromStr, time::Duration};

use anyhow::anyhow;
use assert_fs::TempDir;
use gears::{baseapp::Genesis, cli::ApplicationCli};
use run_script::{IoOptions, ScriptOptions};
use tendermint::informal::chain::Id;

/// Struct for process which lauched from tmp dir
#[derive(Debug)]
pub struct TmpChild(pub Child, pub TempDir);

impl Drop for TmpChild {
    fn drop(&mut self) {
        // Stop child process before deletion of tmp dir
        while let Err(_) = self.0.kill() {
            std::thread::sleep(Duration::from_millis(100))
        }
    }
}

impl TmpChild {
    pub fn start_tendermint<G: Genesis, AC: gears::config::ApplicationConfig, CL : ApplicationCli + Clone>(
        path_to_tendermint: &(impl AsRef<Path> + ?Sized),
        genesis: &G,
    ) -> anyhow::Result<Self> {
        let tmp_dir = TempDir::new()?;

        dircpy::CopyBuilder::new(path_to_tendermint, &tmp_dir)
            .overwrite(true)
            .run()?;

        let opt: gears::client::init::InitCommand<CL> = gears::client::init::InitOptionsBuilder::default()
            .chain_id(Id::from_str("test-chain")?)
            .moniker("test".to_owned())
            .build()?;

        gears::client::init::init::<_, AC, _>(opt, genesis)?;

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
            &r#"
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
