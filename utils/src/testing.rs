use std::{path::Path, process::Child, str::FromStr, time::Duration};

use assert_fs::{fixture::PathCopy, TempDir};
use gears::baseapp::Genesis;
use run_script::{IoOptions, ScriptOptions};
use tendermint::informal::chain::Id;

use crate::tendermint::InitOptionsBuilder;

/// Struct for process which lauched from tmp dir
#[derive(Debug)]
pub struct TmpChild(pub Child, pub TempDir);

impl Drop for TmpChild {
    fn drop(&mut self) {
        // Stop child process before deletion of tmp dir
        while let Err(_) = self.0.kill() { std::thread::sleep(Duration::from_millis(100))}
    }
}

impl TmpChild {
    pub fn start_tendermint< G : Genesis, AC: gears::config::ApplicationConfig>( path_to_tendermint : &impl AsRef<Path>, genesis : &G ) -> anyhow::Result<Self> {
        let tmp_dir = TempDir::new()?;
        tmp_dir.copy_from(path_to_tendermint, &["*"] )?;

        let opt = InitOptionsBuilder::default()
        .chain_id( Id::from_str( "test-chain")? )
        .app_genesis_state( &genesis )
        .moniker( "test".to_owned() )
        .build()?;

        crate::tendermint::init_tendermint::<_, AC>(opt)?;

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

        let script = format!( "./tendermint start --home {:?}", tmp_dir.path() );
        let child = run_script::spawn(
            &script,
            &vec![],
            &options,
        )?;

        Ok(Self(child, tmp_dir))
    }
}
