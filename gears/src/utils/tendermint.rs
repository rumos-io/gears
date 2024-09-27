use std::{
    path::{Path, PathBuf},
    process::Child,
    str::FromStr,
};

use anyhow::anyhow;
pub use assert_fs::TempDir;

use run_script::{IoOptions, ScriptOptions};
use tendermint::types::chain_id::ChainId;

use crate::commands::node::run::LogLevel;

pub fn random_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("failed to bind to random addr")
        .local_addr()
        .expect("failed to get addr")
        .port()
}

/// Struct for process which launched from tmp dir
#[derive(Debug)]
pub struct TendermintSubprocess {
    child: Child,
    dir: TempDir,
    pub rpc_port: u16,
    pub p2p_port: u16,
    pub proxy_port: u16,
    pub chain_id: ChainId,
    pub moniker: &'static str,
}

impl TendermintSubprocess {
    pub fn home(&self) -> PathBuf {
        self.dir.join("node")
    }

    pub fn run(path_to_assets: impl AsRef<Path>, level: LogLevel) -> anyhow::Result<Self> {
        const MONIKER: &str = "test";
        const CHAIN_ID: &str = "test-chain";

        let chain_id = ChainId::from_str(CHAIN_ID)?;

        let tmp_dir = TempDir::new()?;
        // let tmp_dir = tmp_dir.into_persistent();

        let options = ScriptOptions {
            runner: None,
            runner_args: None,
            working_directory: Some(tmp_dir.to_path_buf()),
            input_redirection: IoOptions::Null,
            output_redirection: match level {
                LogLevel::Off => IoOptions::Pipe,
                _ => IoOptions::Inherit,
            },
            exit_on_error: false,
            print_commands: false,
            env_vars: None,
        };

        let (p2p_port, rpc_port, proxy_port) = (random_port(), random_port(), random_port());

        let tm_path = path_to_assets
            .as_ref()
            .to_str()
            .ok_or(anyhow!("failed to get path to tmp folder"))?;
        let tmp_dir_path = tmp_dir
            .path()
            .to_str()
            .ok_or(anyhow!("failed to get path to tmp folder"))?;

        let copy_script =
            format!("cp -r {tm_path}/node {tm_path}/tendermint.tar.gz {tmp_dir_path}");

        let (_code, _output, _error) = run_script::run(&copy_script, &vec![], &options)?;

        let (_code, _output, _error) =
            run_script::run(r#"tar -xf tendermint.tar.gz"#, &vec![], &options)?;

        let log_script = match level {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Error | LogLevel::Warn => "error",
            LogLevel::Off => "none",
        };

        let script = format!(
            "./tendermint start --home {} --p2p.laddr=tcp://0.0.0.0:{p2p_port} --rpc.laddr=tcp://127.0.0.1:{rpc_port} --proxy_app=tcp://127.0.0.1:{proxy_port} --log_level={log_script}",
            tmp_dir.join("node")
                .to_str()
                .ok_or(anyhow!("failed to get path to tmp folder"))?
        );

        let child = run_script::spawn(&script, &vec![], &options)?;

        Ok(Self {
            child,
            dir: tmp_dir,
            rpc_port,
            p2p_port,
            proxy_port,
            chain_id,
            moniker: MONIKER,
        })
    }
}

impl Drop for TendermintSubprocess {
    fn drop(&mut self) {
        // Stop child process before deletion of tmp dir
        while let Err(_) = self.child.kill() {
            std::thread::sleep(std::time::Duration::from_millis(100))
        }
    }
}
