//! `cargo-bumpversion` subcommand integration for bumpversion CLI.
//!
//! Accepts both invocation styles — `cargo bumpversion <args…>` and a direct
//! `cargo-bumpversion <args…>` — and delegates to the same logic as the
//! `bumpversion` binary.
#![forbid(unsafe_code)]

mod common;
mod logging;
mod options;
mod verbose;

use clap::Parser;
use color_eyre::eyre;
use std::ffi::{OsStr, OsString};

/// The subcommand name cargo injects, derived from this binary's own name so it
/// stays correct if the binary is renamed.
fn cargo_subcommand_name() -> &'static str {
    const BIN: &str = env!("CARGO_BIN_NAME");
    BIN.strip_prefix("cargo-").unwrap_or(BIN)
}

/// Normalize `argv` so clap sees the same arguments for both invocation styles.
///
/// Cargo runs an external subcommand as `cargo-<name> <name> <args…>`, so the
/// name appears twice. Exactly one occurrence is removed, and only in the single
/// position cargo puts it — immediately after the executable. Every other
/// argument is passed through untouched, which is what keeps a direct
/// `cargo-bumpversion patch` working identically.
///
/// `argv[0]` is deliberately preserved. [`Parser::parse_from`] treats the first
/// element as the program name and discards it, so dropping it here would make
/// clap silently swallow the first *real* argument — the defect that turned
/// read-only invocations such as `cargo bumpversion show-bump major` into actual
/// version bumps, and made `cargo-bumpversion --help` bump instead of printing
/// help.
fn normalize_args<I>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args: Vec<OsString> = args.into_iter().collect();
    let subcommand = OsStr::new(cargo_subcommand_name());
    if args.get(1).is_some_and(|arg| arg == subcommand) {
        args.remove(1);
    }
    args
}

/// Main entry point for `cargo-bumpversion`.
#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let mut options = options::Options::parse_from(normalize_args(std::env::args_os()));
    options::fix(&mut options);
    common::bumpversion(options).await
}

#[cfg(test)]
mod tests {
    use super::normalize_args;
    use std::ffi::OsString;

    fn normalize(args: &[&str]) -> Vec<String> {
        normalize_args(args.iter().map(OsString::from))
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    /// `cargo bumpversion patch` reaches this binary with the name repeated.
    #[test]
    fn strips_the_injected_subcommand_name() {
        assert_eq!(
            normalize(&["cargo-bumpversion", "bumpversion", "patch"]),
            ["cargo-bumpversion", "patch"]
        );
    }

    /// A direct invocation has no name to strip, and must keep every argument.
    #[test]
    fn leaves_a_direct_invocation_alone() {
        assert_eq!(
            normalize(&["cargo-bumpversion", "patch"]),
            ["cargo-bumpversion", "patch"]
        );
    }

    /// The regression that caused real bumps: `argv[0]` must survive, or clap
    /// consumes `show-bump` as the program name and `major` becomes the bump.
    #[test]
    fn preserves_argv0_so_the_first_real_argument_is_not_eaten() {
        assert_eq!(
            normalize(&["cargo-bumpversion", "bumpversion", "show-bump", "major"]),
            ["cargo-bumpversion", "show-bump", "major"]
        );
        assert_eq!(
            normalize(&["cargo-bumpversion", "--help"]),
            ["cargo-bumpversion", "--help"]
        );
    }

    /// Only the position cargo uses is stripped, and only once — a later
    /// occurrence is a real argument (a path, say) and must be left in place.
    #[test]
    fn strips_only_the_first_position_and_only_once() {
        assert_eq!(
            normalize(&["cargo-bumpversion", "bumpversion", "bumpversion"]),
            ["cargo-bumpversion", "bumpversion"]
        );
        assert_eq!(
            normalize(&["cargo-bumpversion", "patch", "bumpversion"]),
            ["cargo-bumpversion", "patch", "bumpversion"]
        );
    }

    /// Nothing to strip, and nothing to panic over.
    #[test]
    fn handles_argv_without_arguments() {
        assert_eq!(normalize(&["cargo-bumpversion"]), ["cargo-bumpversion"]);
        assert!(normalize(&[]).is_empty());
    }
}
