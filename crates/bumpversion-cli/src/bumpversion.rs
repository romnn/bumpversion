//! `bumpversion` CLI binary entrypoint.

#![forbid(unsafe_code)]

mod common;
mod logging;
mod options;
mod verbose;

use clap::Parser;
use color_eyre::eyre;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let result: eyre::Result<()> = async {
        color_eyre::install()?;

        let mut options = options::Options::parse();
        options::fix(&mut options);
        common::bumpversion(options).await
    }
    .await;
    common::report_result(result)
}
