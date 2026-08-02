//! Utilities for running and checking external commands.
use async_process::{Command, ExitStatus};
use std::borrow::Cow;
use std::ffi::OsStr;

/// The captured output of a child process.
///
/// Contains `stdout`, `stderr`, and the exit `status`.
/// Captured output of a child process execution.
#[derive(Debug, Clone, PartialEq)]
pub struct Output {
    /// Standard output of the command.
    pub stdout: String,
    /// Standard error of the command.
    pub stderr: String,
    /// Exit status of the process.
    pub status: ExitStatus,
}

impl From<async_process::Output> for Output {
    fn from(output: async_process::Output) -> Self {
        Self {
            stdout: String::from_utf8_lossy(&output.stdout).into(),
            stderr: String::from_utf8_lossy(&output.stderr).into(),
            status: output.status,
        }
    }
}

/// Errors that can occur when running an external process.
/// Errors that can occur when running external commands.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// I/O error while spawning or capturing the process.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    // TODO: into eyre here!
    /// The process exited with a non-zero status code.
    #[error(
        "`{}` failed with code {}:\n\n--- Stdout:\n {}\n--- Stderr:\n {}",
        command,
        output.status.code().unwrap_or(1),
        output.stdout,
        output.stderr
    )]
    Failed {
        /// The command that was run, rendered without its environment.
        command: String,
        /// Captured output including status, stdout, stderr.
        output: Output,
    },
}

/// Renders a command as a shell-like invocation, without its environment.
///
/// The result names the failing command in [`Error::Failed`], so it reaches the
/// user.
///
/// [`Command`]'s `Debug` is not an alternative: it prints every environment
/// variable set on the command, which may hold credentials.
fn display_command(cmd: &Command) -> String {
    fn quote(value: &OsStr) -> String {
        let value = value.to_string_lossy();
        shlex::try_quote(&value).map_or_else(|_| value.to_string(), Cow::into_owned)
    }

    let command: Vec<String> = std::iter::once(quote(cmd.get_program()))
        .chain(cmd.get_args().map(quote))
        .collect();
    let command = command.join(" ");
    match cmd.get_current_dir() {
        Some(dir) => format!("cd {} && {command}", quote(dir.as_os_str())),
        None => command,
    }
}

/// Check that the process exited successfully, returning an error otherwise.
///
/// # Errors
/// Returns `Error::Failed` if the exit status indicates failure.
/// Check that a process exited successfully, returning an error otherwise.
///
/// # Errors
/// Returns `Error::Failed` if the exit status indicates failure.
pub fn check_exit_status(cmd: &Command, output: &async_process::Output) -> Result<(), Error> {
    if output.status.success() {
        Ok(())
    } else {
        Err(Error::Failed {
            command: display_command(cmd),
            output: output.clone().into(),
        })
    }
}

/// Execute the given command, capturing output and checking exit status.
///
/// # Errors
/// Returns `Error::Io` for I/O errors or `Error::Failed` if the process exits with non-zero status.
/// Execute the given command, capturing stdout/stderr and checking exit code.
///
/// # Errors
/// Returns `Error::Io` for I/O failures or `Error::Failed` for non-zero exits.
pub async fn run_command(cmd: &mut Command) -> Result<Output, Error> {
    let output = cmd.output().await?;
    check_exit_status(cmd, &output)?;
    Ok(output.into())
}

#[cfg(test)]
mod tests {
    use similar_asserts::assert_eq as sim_assert_eq;

    /// Environment variables set on a command stay out of its rendered form,
    /// which is what a failing hook shows the user.
    #[test]
    fn display_command_omits_the_environment() {
        let mut cmd = async_process::Command::new("sh");
        cmd.args(["-c", "cargo update --offline"]);
        cmd.env("API_TOKEN", "s3cr3t");
        cmd.current_dir("/home/user/my repo");

        let rendered = super::display_command(&cmd);
        sim_assert_eq!(
            rendered,
            "cd '/home/user/my repo' && sh -c 'cargo update --offline'"
        );
    }
}
