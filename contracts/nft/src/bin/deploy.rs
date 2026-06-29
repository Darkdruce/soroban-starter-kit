#![allow(clippy::expect_used, clippy::panic)]
//! Deploy helper for soroban-nft-template.
//!
//! Usage:
//!   cargo run --bin deploy -- --network testnet --source alice
//!   cargo run --bin deploy -- --network mainnet --source alice --wasm path/to/nft.wasm

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let has_wasm_flag = args.windows(2).any(|w| w[0] == "--wasm");
    if !has_wasm_flag {
        let status = Command::new("stellar")
            .args(["contract", "build"])
            .status()
            .expect("failed to run `stellar contract build`");
        if !status.success() {
            eprintln!("Build failed.");
            return ExitCode::FAILURE;
        }
    }

    let status = Command::new("stellar")
        .args(["contract", "deploy"])
        .args(&args)
        .status()
        .expect("failed to run `stellar contract deploy`");

    if status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
