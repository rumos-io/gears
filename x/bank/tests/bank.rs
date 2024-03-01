// mod utils {
//     use std::{borrow::Cow, path::PathBuf, process::Child};

//     use assert_fs::TempDir;
//     use run_script::{IoOptions, ScriptOptions};
//     use serde::Serialize;
//     use tendermint::informal::chain::Id;

//     /// Struct for process which lauched from tmp dir
//     #[derive(Debug)]
//     pub struct TmpChild(pub Child, pub TempDir);

//     impl Drop for TmpChild {
//         fn drop(&mut self) {
//             // Stop child process before deletion of tmp dir
//             while let Err(_) = self.0.kill() {}
//         }
//     }

//     impl TmpChild {
//         pub fn start_tendermint() -> anyhow::Result<Self> {
//             let tmp_dir = assert_fs::TempDir::new()?;

//             let options = ScriptOptions {
//                 runner: None,
//                 runner_args: None,
//                 working_directory: Some(tmp_dir.to_path_buf()),
//                 input_redirection: IoOptions::Inherit,
//                 output_redirection: IoOptions::Pipe,
//                 exit_on_error: false,
//                 print_commands: false,
//                 env_vars: None,
//             };

//             // TODO: Copy local file downloaded once
//             let (_code, _output, _error) = run_script::run(
//                 &r#"
//             curl -L https://github.com/tendermint/tendermint/releases/download/v0.34.21/tendermint_0.34.21_linux_amd64.tar.gz > tendermint.tar.gz

//             tar -xf tendermint.tar.gz
//             "#,
//                 &vec![],
//                 &options,
//             )?;

//             let child = run_script::spawn(
//                 r#"
//             ./tendermint init --home ./tm_config
//             ./tendermint start --home ./tm_config
//             "#,
//                 &vec![],
//                 &options,
//             )?;

//             dbg!(&child);

//             Ok(Self(child, tmp_dir))
//         }
//     }

//     pub fn init_sh() -> anyhow::Result<Child> {
//         let options = ScriptOptions::new();

//         let child = run_script::spawn(
//             r#"
//             set -eux

//             rm -rf ~/.gaia-rs
//             cargo run -- init test

//             cargo run -- add-genesis-account cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux 34uatom
//             "#,
//             &vec![],
//             &options,
//         )?;

//         Ok(child)
//     }
// }
