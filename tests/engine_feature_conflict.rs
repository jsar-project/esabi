#![cfg(not(target_arch = "wasm32"))]

use std::{env, path::PathBuf, process::Command};

#[test]
fn enabling_multiple_engines_fails() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("engine-conflict")
        .join("Cargo.toml");

    let cargo = env::var("CARGO").unwrap_or_else(|_| String::from("cargo"));
    let output = Command::new(cargo)
        .arg("check")
        .arg("--manifest-path")
        .arg(&manifest)
        .env("CARGO_TERM_COLOR", "never")
        .output()
        .expect("failed to run cargo check for the engine conflict fixture");

    assert!(
        !output.status.success(),
        "expected cargo check to fail when both engines are enabled"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Only one JavaScript engine can be enabled at a time"),
        "unexpected stderr:\n{stderr}"
    );
}
