//! Typed errors for the CLI and their rendering on the terminal.
//!
//! Every failure a user can provoke is a variant of [`Error`] and is printed as a short report: a
//! headline, the output of the external command that failed when there is one, the chain of
//! causes otherwise, and what to do next.
//! Panics are the only failures left to the `color_eyre` handler installed in `main`.
use bumpversion::{
    BumpError,
    command::Error as CommandError,
    config, hooks,
    vcs::git::{self, GitRepository},
};
use colored::Colorize;
use std::path::PathBuf;

/// Errors the CLI reports to the user.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The current working directory could not be determined.
    #[error("could not determine the current working directory")]
    CurrentDir(#[source] std::io::Error),
    /// The working directory does not exist or cannot be resolved.
    #[error("could not resolve directory {}", path.display())]
    ResolveDir {
        /// The directory that was given.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// `--config-file` names a file that does not exist.
    #[error("config file {} does not exist", path.display())]
    ConfigFileNotFound {
        /// The file that was given.
        path: PathBuf,
    },
    /// The named config file holds no bumpversion configuration.
    #[error("no bumpversion configuration found in {}", path.display())]
    ConfigFileEmpty {
        /// The file that was read.
        path: PathBuf,
    },
    /// None of the default config files exists in the working directory.
    #[error("no bumpversion configuration found in {}", dir.display())]
    ConfigNotFound {
        /// The directory that was searched.
        dir: PathBuf,
    },
    /// The working tree has uncommitted changes and `allow_dirty` is off.
    #[error("working directory is not clean")]
    Dirty {
        /// The files with uncommitted changes, relative to the repository.
        files: Vec<PathBuf>,
    },
    /// Neither a subcommand nor a positional argument names the component to bump.
    #[error("missing version component to bump")]
    MissingComponent,
    /// The configuration has no current version.
    #[error("missing current version")]
    MissingCurrentVersion,
    /// The configured current version does not match the parse pattern.
    #[error("could not parse current version {version:?}")]
    InvalidCurrentVersion {
        /// The version string from the configuration.
        version: String,
    },
    /// Invalid command-line arguments.
    #[error(transparent)]
    Options(#[from] crate::options::Error),
    /// Logging could not be set up.
    #[error(transparent)]
    Logging(#[from] crate::logging::Error),
    /// The configuration could not be read or parsed.
    #[error(transparent)]
    Config(#[from] config::Error),
    /// A configured file could not be resolved.
    #[error(transparent)]
    Files(#[from] bumpversion::files::Error),
    /// A git command failed before the bump started.
    #[error(transparent)]
    Git(#[from] git::Error),
    /// The bump or finalization failed.
    #[error(transparent)]
    Bump(#[from] BumpError<GitRepository>),
    /// `show-bump` could not bump the component.
    #[error("failed to bump version")]
    VersionBump(#[from] bumpversion::version::BumpError),
    /// `show-bump` could not serialize the bumped version.
    #[error("failed to serialize version")]
    Serialize(#[from] bumpversion::version::SerializeError),
}

/// The situation after a bump stopped before its commit.
const RECOVER: &str = "The version changes are still in your working tree. \
    Either revert them and start over, or fix the issue and run:";

/// The command that resumes an interrupted bump.
const RECOVER_COMMAND: &str = "bumpversion finalize --allow-dirty";

impl Error {
    /// The external command whose failure caused this error, if there is one.
    fn failed_command(&self) -> Option<&CommandError> {
        match self {
            Self::Bump(
                BumpError::SetupHook(hook)
                | BumpError::PreCommitHook(hook)
                | BumpError::PostCommitHook(hook),
            ) => match hook {
                hooks::Error::Command(command) => Some(command),
                hooks::Error::Shell(_) => None,
            },
            Self::Bump(BumpError::Add(git) | BumpError::Commit(git) | BumpError::Tag(git))
            | Self::Git(git) => match git {
                git::Error::CommandFailed(command) => Some(command),
                _ => None,
            },
            _ => None,
        }
    }

    /// What to do next, when the error leaves the user with a choice.
    fn guidance(&self) -> Option<String> {
        match self {
            Self::Dirty { .. } => Some(
                "Commit or stash these changes, or pass --allow-dirty to bump anyway.".to_string(),
            ),
            Self::ConfigNotFound { dir } => {
                let candidates: Vec<String> = config::config_file_locations(dir)
                    // `Cargo.toml` is a candidate the library does not read yet.
                    .filter(|file| !matches!(file, config::ConfigFile::CargoToml(_)))
                    .filter_map(|file| {
                        let name = file.path().file_name()?;
                        Some(name.to_string_lossy().into_owned())
                    })
                    .collect();
                Some(format!(
                    "bumpversion reads {}. Pass --config-file to use another file.",
                    candidates.join(", ")
                ))
            }
            Self::MissingComponent => Some(
                "Name the component to bump, for example `bumpversion patch`, or pass --new-version."
                    .to_string(),
            ),
            Self::Bump(BumpError::PreCommitHook(_) | BumpError::Add(_) | BumpError::Commit(_)) => {
                Some(format!("{RECOVER}\n\n  {}", RECOVER_COMMAND.bold()))
            }
            Self::Bump(BumpError::Tag(_)) => Some(
                "The release commit was created, but the tag was not. \
                 Fix the issue and create the tag yourself."
                    .to_string(),
            ),
            Self::Bump(BumpError::PostCommitHook(_)) => Some(
                "The release commit and tag were already created. Only the hook failed.".to_string(),
            ),
            Self::Bump(BumpError::MissingPreviousVersion) => Some(
                "finalize takes the previous version from the latest tag, and the repository has none."
                    .to_string(),
            ),
            _ => None,
        }
    }

    /// Whether the error points at a defect in bumpversion rather than at its input.
    fn is_internal(&self) -> bool {
        matches!(
            self,
            Self::Logging(_) | Self::Config(config::Error::Join(_) | config::Error::Diagnostics(_))
        )
    }
}

/// The chain of causes below `error`, outermost first.
fn causes(error: &Error) -> Vec<String> {
    let mut causes = Vec::new();
    let mut source = std::error::Error::source(error);
    while let Some(cause) = source {
        causes.push(cause.to_string());
        source = cause.source();
    }
    causes
}

/// Indent every line of `text` by two spaces.
fn indent(text: &str) -> String {
    text.lines()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The suffix that names how a failed command ended.
fn exit_status(status: std::process::ExitStatus) -> String {
    match status.code() {
        Some(code) => format!(" with exit code {code}"),
        // Unix only: the process was killed by a signal.
        None => " without an exit code".to_string(),
    }
}

/// Render `error` as the report printed to stderr.
///
/// The report ends with a newline.
pub fn render(error: &Error) -> String {
    let command = error.failed_command();

    // A failed git command has no headline beyond its own output, so name it.
    let mut headline = match error {
        Error::Git(git::Error::CommandFailed(CommandError::Failed { .. })) => {
            "git command failed".to_string()
        }
        _ => error.to_string(),
    };
    if let Some(CommandError::Failed { output, .. }) = command {
        headline.push_str(&exit_status(output.status));
    }
    let label = "error:".red().bold();
    let head = match command {
        Some(CommandError::Failed { command, .. }) => format!("{label} {headline}\n  {command}"),
        _ => format!("{label} {headline}"),
    };

    let mut sections = vec![head];

    if let Error::Dirty { files } = error {
        let files: Vec<String> = files
            .iter()
            .map(|file| file.display().to_string().cyan().to_string())
            .collect();
        sections.push(indent(&files.join("\n")));
    }

    if let Some(CommandError::Failed { output, .. }) = command {
        for (label, text) in [("stdout:", &output.stdout), ("stderr:", &output.stderr)] {
            if !text.trim().is_empty() {
                sections.push(format!("{}\n{}", label.dimmed(), indent(text.trim_end())));
            }
        }
    } else {
        let causes = causes(error);
        if !causes.is_empty() {
            sections.push(format!(
                "{}\n{}",
                "Caused by:".dimmed(),
                indent(&causes.join("\n"))
            ));
        }
    }

    if let Some(guidance) = error.guidance() {
        sections.push(guidance);
    }
    if error.is_internal() {
        sections.push(format!(
            "This looks like a bug in bumpversion. Please report it at {}.",
            "https://github.com/romnn/bumpversion/issues".underline()
        ));
    }

    let mut report = sections.join("\n\n");
    report.push('\n');
    report
}

#[cfg(test)]
mod tests {
    use super::{Error, render};
    use indoc::indoc;
    use std::path::PathBuf;

    /// The dirty files are listed relative to the repository, with the way out below them.
    #[test]
    fn dirty_working_directory_lists_files_and_the_way_out() {
        colored::control::set_override(false);
        let error = Error::Dirty {
            files: vec![PathBuf::from("Cargo.lock"), PathBuf::from("docs/ci.md")],
        };
        assert_eq!(
            render(&error),
            indoc! {"
                error: working directory is not clean

                  Cargo.lock
                  docs/ci.md

                Commit or stash these changes, or pass --allow-dirty to bump anyway.
            "}
        );
    }

    /// An error without a command behind it shows its causes, innermost last.
    #[test]
    fn causes_are_listed_below_the_headline() {
        colored::control::set_override(false);
        let error = Error::VersionBump(bumpversion::version::BumpError::InvalidComponent(
            "flavor".to_string(),
        ));
        assert_eq!(
            render(&error),
            indoc! {r#"
                error: failed to bump version

                Caused by:
                  invalid version component "flavor"
            "#}
        );
    }
}
