//! `bumpversion` CLI binary entrypoint.

#![forbid(unsafe_code)]

mod common;
mod error;
mod logging;
mod options;
mod verbose;

use clap::Parser;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    // Every expected failure is a typed error that `report_result` renders; the
    // eyre handler is installed for panics alone.
    if let Err(error) = color_eyre::install() {
        eprintln!("error: could not install the panic handler: {error}");
        return ExitCode::FAILURE;
    }

    let mut options = options::Options::parse();
    options::fix(&mut options);
    common::report_result(common::bumpversion(options).await)
}
