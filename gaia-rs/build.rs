use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .map(|this| this.stdout)
        .unwrap_or_default();

    let git_hash = String::from_utf8_lossy(&output);

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}
