// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::{fs::read_dir, io::Error, path::Path, process::Command};

fn main() -> Result<(), Error> {
    let env = Env::new();

    for feature in read_dir("features")? {
        let path = feature?.path();
        if let Some(name) = path.file_stem() {
            println!("cargo:rerun-if-changed={}", path.display());
            if env.check(&path)? {
                supports(name.to_str().expect("valid feature name"));
            }
        }
    }

    match env.target_os.as_str() {
        "linux" => {
            supports("gso");
            supports("pktinfo");
            supports("tos");
        }
        "macos" => {
            supports("pktinfo");
            supports("tos");
        }
        _ => {
            // TODO others
        }
    }

    Ok(())
}

fn supports(name: &str) {
    println!("cargo:rustc-cfg=s2n_quic_platform_{}", name);
}

struct Env {
    rustc: String,
    out_dir: String,
    target: String,
    target_os: String,
}

impl Env {
    fn new() -> Self {
        // See https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts
        Self {
            rustc: env("RUSTC"),
            out_dir: env("OUT_DIR"),
            target: env("TARGET"),
            target_os: env("CARGO_CFG_TARGET_OS"),
        }
    }

    // Tries to compile the program and returns if it was successful
    fn check(&self, path: &Path) -> Result<bool, Error> {
        let mut command = Command::new(&self.rustc);

        command
            .arg("--out-dir")
            .arg(&self.out_dir)
            .arg("--target")
            .arg(&self.target)
            .arg("--crate-type")
            .arg("bin")
            .arg("--codegen")
            .arg("opt-level=0")
            .arg(path);

        for (key, _) in std::env::vars() {
            const CARGO_FEATURE: &str = "CARGO_FEATURE_";
            if key.starts_with(CARGO_FEATURE) {
                command.arg("--cfg").arg(format!(
                    "feature=\"{}\"",
                    key.trim_start_matches(CARGO_FEATURE)
                        .to_lowercase()
                        .replace('_', "-")
                ));
            }
        }

        Ok(command.spawn()?.wait()?.success())
    }
}

fn env(name: &str) -> String {
    println!("cargo:rerun-if-env-changed={}", name);
    std::env::var(name)
        .unwrap_or_else(|_| panic!("build script missing {:?} environment variable", name))
}
