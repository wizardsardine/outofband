//! Bakes the commit being built into the binary so the deployed page can say
//! which revision it is running.

use std::process::Command;

fn main() {
    println!("cargo:rustc-env=OUTOFBAND_COMMIT={}", commit());
    println!("cargo:rerun-if-env-changed=OUTOFBAND_COMMIT");
    // A new commit changes HEAD; a new commit on the current branch changes
    // the ref it points at. Without both, the value sticks at whatever the
    // first build saw.
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/refs/heads");
}

/// The environment wins over git: the deploy scripts exclude `.git` from the
/// rsync, so on a server there is no repository to ask and the value has to
/// be passed in.
fn commit() -> String {
    if let Ok(from_env) = std::env::var("OUTOFBAND_COMMIT") {
        let from_env = from_env.trim();
        if !from_env.is_empty() {
            return from_env.to_string();
        }
    }

    let Some(hash) = git(&["rev-parse", "--short", "HEAD"]) else {
        return "unknown".to_string();
    };
    // A hash alone would claim the page matches that commit, which is false
    // while the tree has edits in it.
    match git(&["status", "--porcelain"]) {
        Some(status) if !status.is_empty() => format!("{hash}-dirty"),
        _ => hash,
    }
}

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8(output.stdout).ok()?.trim().to_string())
}
