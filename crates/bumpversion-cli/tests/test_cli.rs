//! Integration tests for the `bumpversion` CLI binary.

use assert_cmd::Command;
use color_eyre::eyre;
use indoc::indoc;
use predicates::prelude::*;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

/// The usage line clap prints for `subcommand`.
///
/// clap derives the usage line from the file name of `argv[0]`, which carries a
/// `.exe` suffix on Windows.
fn usage(subcommand: &str) -> String {
    let bin = Path::new(env!("CARGO_BIN_EXE_bumpversion"))
        .file_name()
        .unwrap_or_else(|| OsStr::new("bumpversion"))
        .to_string_lossy();
    format!("Usage: {bin} {subcommand}")
}

/// A `.bumpversion.toml` at `1.2.3` with `commit` and `tag` enabled, in a fresh
/// repository — the shape that made the `cargo-bumpversion` argument bug
/// destructive rather than merely wrong.
fn armed_repo() -> eyre::Result<tempfile::TempDir> {
    let temp = tempfile::tempdir()?;
    fs::write(
        temp.path().join(".bumpversion.toml"),
        r#"
[tool.bumpversion]
current_version = "1.2.3"
commit = true
tag = true

[[tool.bumpversion.files]]
filename = "VERSION"
"#,
    )?;
    fs::write(temp.path().join("VERSION"), "1.2.3")?;
    git_init(temp.path())?;
    Ok(temp)
}

/// Assert the repository is exactly as `armed_repo` left it.
fn assert_untouched(dir: &Path) -> eyre::Result<()> {
    assert_eq!(fs::read_to_string(dir.join("VERSION"))?, "1.2.3");
    assert!(
        fs::read_to_string(dir.join(".bumpversion.toml"))?.contains(r#"current_version = "1.2.3""#),
        "config version must not have changed"
    );
    let tags = std::process::Command::new("git")
        .args(["tag", "-l"])
        .current_dir(dir)
        .output()?;
    assert!(
        tags.stdout.is_empty(),
        "no tag may have been created, found: {}",
        String::from_utf8_lossy(&tags.stdout)
    );
    Ok(())
}

fn git_init(dir: &Path) -> eyre::Result<()> {
    let output = std::process::Command::new("git")
        .arg("init")
        .current_dir(dir)
        .output()?;
    eyre::ensure!(output.status.success(), "failed to init git repo");
    Ok(())
}

#[test]
fn test_show_help() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.arg("show").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(usage("show")));
}

#[test]
fn test_show_bump_help() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.arg("show-bump").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(usage("show-bump")));
}

/// `cargo-bumpversion --help` must print help. It used to fall through to the
/// bump path, and in a repository with `commit`/`tag` enabled that committed and
/// tagged a real release.
#[test]
fn test_cargo_bumpversion_help_does_not_bump() -> eyre::Result<()> {
    let temp = armed_repo()?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cargo-bumpversion"));
    cmd.current_dir(temp.path()).arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));

    assert_untouched(temp.path())
}

/// The exact invocation that caused the incident: cargo execs this binary as
/// `cargo-bumpversion bumpversion show-bump major`. `show-bump` is read-only and
/// must stay read-only.
#[test]
fn test_cargo_subcommand_show_bump_is_read_only() -> eyre::Result<()> {
    let temp = armed_repo()?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cargo-bumpversion"));
    cmd.current_dir(temp.path())
        .arg("bumpversion")
        .arg("show-bump")
        .arg("major");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("old_version=1.2.3"))
        .stdout(predicate::str::contains("new_version=2.0.0"));

    assert_untouched(temp.path())
}

/// The same command invoked directly, without cargo's injected subcommand name.
#[test]
fn test_cargo_bumpversion_direct_show_bump_is_read_only() -> eyre::Result<()> {
    let temp = armed_repo()?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cargo-bumpversion"));
    cmd.current_dir(temp.path()).arg("show-bump").arg("major");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("new_version=2.0.0"));

    assert_untouched(temp.path())
}

/// Both invocation styles must reach the same parse, so a real bump still works.
#[test]
fn test_cargo_subcommand_bump_matches_direct_invocation() -> eyre::Result<()> {
    for prefix in [vec!["bumpversion"], vec![]] {
        let temp = armed_repo()?;

        let mut cmd = Command::new(env!("CARGO_BIN_EXE_cargo-bumpversion"));
        cmd.current_dir(temp.path())
            .args(prefix)
            .arg("bump")
            .arg("patch")
            .arg("--allow-dirty")
            .arg("--no-commit")
            .arg("--no-tag");
        cmd.assert().success();

        assert_eq!(fs::read_to_string(temp.path().join("VERSION"))?, "1.2.4");
    }
    Ok(())
}

/// With no component and no `--new-version`, both binaries must refuse rather
/// than guess.
#[test]
fn test_cargo_subcommand_without_component_fails() -> eyre::Result<()> {
    let temp = armed_repo()?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cargo-bumpversion"));
    cmd.current_dir(temp.path()).arg("bumpversion");
    cmd.assert().failure().stderr(predicate::str::contains(
        "missing version component to bump",
    ));

    assert_untouched(temp.path())
}

/// Write `contents` to `name` in a fresh repository.
fn repo_with(name: &str, contents: &str) -> eyre::Result<tempfile::TempDir> {
    let temp = tempfile::tempdir()?;
    fs::write(temp.path().join(name), contents)?;
    git_init(temp.path())?;
    Ok(temp)
}

/// `--config-file` was parsed and then never read, so a config under any
/// non-default name was silently ignored.
#[test]
fn test_config_file_flag_selects_the_file() -> eyre::Result<()> {
    let temp = repo_with(
        "custom-release.toml",
        "[tool.bumpversion]\ncurrent_version = \"9.9.9\"\n",
    )?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path())
        .args(["--config-file", "custom-release.toml"])
        .args(["show", "current_version"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("9.9.9"));

    // Without the flag there is no config file to discover at all.
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path())
        .args(["show", "current_version"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("missing config file"));
    Ok(())
}

/// A path that does not exist must be reported, not silently fall back to
/// discovery.
#[test]
fn test_config_file_flag_rejects_a_missing_file() -> eyre::Result<()> {
    let temp = repo_with(
        ".bumpversion.toml",
        "[tool.bumpversion]\ncurrent_version = \"1.2.3\"\n",
    )?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path())
        .args(["--config-file", "nope.toml"])
        .args(["show", "current_version"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
    Ok(())
}

/// A repository whose `VERSION` file needs a regex to match.
fn regex_search_repo() -> eyre::Result<tempfile::TempDir> {
    let temp = tempfile::tempdir()?;
    fs::write(
        temp.path().join(".bumpversion.toml"),
        r#"
[tool.bumpversion]
current_version = "1.0.0"

[[tool.bumpversion.files]]
filename = "VERSION"
"#,
    )?;
    fs::write(temp.path().join("VERSION"), "version = \"1.0.0\"\n")?;
    git_init(temp.path())?;
    Ok(temp)
}

const REGEX_SEARCH: &str = r#"version = "[0-9]+\.[0-9]+\.[0-9]+""#;
const REGEX_REPLACE: &str = r#"version = "{new_version}""#;

/// `--regex` had no effect at all; the search was always matched literally.
#[test]
fn test_regex_flag_enables_regex_search() -> eyre::Result<()> {
    let temp = regex_search_repo()?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path())
        .args(["--no-commit", "--no-tag", "--regex"])
        .args(["--search", REGEX_SEARCH])
        .args(["--replace", REGEX_REPLACE])
        .args(["bump", "patch"]);
    cmd.assert().success();

    assert_eq!(
        fs::read_to_string(temp.path().join("VERSION"))?,
        "version = \"1.0.1\"\n"
    );
    Ok(())
}

/// The same search without `--regex` is a literal string, so it matches nothing.
#[test]
fn test_no_regex_treats_search_literally() -> eyre::Result<()> {
    let temp = regex_search_repo()?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path())
        .args(["--no-commit", "--no-tag", "--no-regex"])
        .args(["--search", REGEX_SEARCH])
        .args(["--replace", REGEX_REPLACE])
        .args(["bump", "patch"]);
    let _ = cmd.assert();

    assert_eq!(
        fs::read_to_string(temp.path().join("VERSION"))?,
        "version = \"1.0.0\"\n",
        "a literal search must not match"
    );
    Ok(())
}

/// `--allow-dirty` used to be what actually selected regex mode, so an unrelated
/// flag changed how `search` was interpreted.
#[test]
fn test_allow_dirty_does_not_enable_regex() -> eyre::Result<()> {
    let temp = regex_search_repo()?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path())
        .args(["--no-commit", "--no-tag", "--allow-dirty"])
        .args(["--search", REGEX_SEARCH])
        .args(["--replace", REGEX_REPLACE])
        .args(["bump", "patch"]);
    let _ = cmd.assert();

    assert_eq!(
        fs::read_to_string(temp.path().join("VERSION"))?,
        "version = \"1.0.0\"\n",
        "--allow-dirty must not imply --regex"
    );
    Ok(())
}

/// `--tag-message` took its value from `--tag-name`, so the tag message could
/// never be set from the command line.
#[test]
fn test_tag_message_is_independent_of_tag_name() -> eyre::Result<()> {
    let temp = repo_with(
        ".bumpversion.toml",
        "[tool.bumpversion]\ncurrent_version = \"1.0.0\"\n",
    )?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path())
        .args(["--dry-run", "-v", "--tag"])
        .args(["--tag-name", "rel-{new_version}"])
        .args(["--tag-message", "shipping {new_version}"])
        .args(["bump", "minor"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("tag = rel-1.1.0"))
        .stdout(predicate::str::contains("message = shipping 1.1.0"));
    Ok(())
}

/// The INI writeback was a stub that rewrote the file unchanged, so a `.cfg`
/// config kept reporting the old version and the next bump repeated it.
#[test]
fn test_ini_config_updates_current_version() -> eyre::Result<()> {
    let temp = tempfile::tempdir()?;
    fs::write(
        temp.path().join(".bumpversion.cfg"),
        "[bumpversion]\ncurrent_version = 2.1.0\n\n[bumpversion:file:VERSION]\n",
    )?;
    fs::write(temp.path().join("VERSION"), "2.1.0")?;
    git_init(temp.path())?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path())
        .args(["--allow-dirty", "--no-commit", "--no-tag"])
        .args(["bump", "patch"]);
    cmd.assert().success();

    assert_eq!(fs::read_to_string(temp.path().join("VERSION"))?, "2.1.1");
    let config = fs::read_to_string(temp.path().join(".bumpversion.cfg"))?;
    assert!(
        config.contains("current_version = 2.1.1"),
        "the INI config must record the new version, got:\n{config}"
    );
    assert!(
        config.contains("[bumpversion:file:VERSION]"),
        "the rest of the file must be preserved, got:\n{config}"
    );
    Ok(())
}

/// `RUST_LOG` was passed to `with_env_var`, which expects a variable *name*, so
/// every setting failed to resolve and fell back to the defaults.
#[test]
fn test_rust_log_env_var_is_honored() -> eyre::Result<()> {
    let temp = repo_with(
        ".bumpversion.toml",
        "[tool.bumpversion]\ncurrent_version = \"1.2.3\"\n",
    )?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path())
        .env("RUST_LOG", "bumpversion=debug")
        .args(["show", "current_version"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("DEBUG"))
        .stderr(predicate::str::contains("invalid log filter").not());
    Ok(())
}

/// `first_value` was never read from the config, so a reset always went to the
/// first entry of `values` (or `0`) and could not be pointed elsewhere.
#[test]
fn test_parts_first_value_is_read() -> eyre::Result<()> {
    let temp = repo_with(
        ".bumpversion.toml",
        r#"
[tool.bumpversion]
current_version = "1.2.3"

[tool.bumpversion.parts.patch]
first_value = "1"
"#,
    )?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path()).args(["show-bump", "minor"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("new_version=1.3.1"));
    Ok(())
}

/// The idiomatic pre-release ladder: with `first_value` set to the optional
/// value, a component bump collapses the suffix instead of reopening it.
#[test]
fn test_parts_first_value_collapses_a_pre_release() -> eyre::Result<()> {
    let temp = repo_with(
        ".bumpversion.toml",
        r#"
[tool.bumpversion]
current_version = "1.2.0-alpha.1"
parse = '(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)(\-(?P<pre_label>[a-z]+)\.(?P<pre_n>\d+))?'
serialize = [
  "{major}.{minor}.{patch}-{pre_label}.{pre_n}",
  "{major}.{minor}.{patch}",
]

[tool.bumpversion.parts.pre_label]
values = ["alpha", "beta", "rc", "final"]
optional_value = "final"
first_value = "final"
"#,
    )?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path()).args(["show-bump", "patch"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("new_version=1.2.1"));
    Ok(())
}

/// Configure a git identity and commit everything, so the repository is in the
/// state a real bump runs against: a clean tree of tracked files. `git add
/// --update` only stages files git already knows about, so a bump that commits
/// cannot work in a repository that has never committed.
fn git_commit_all(dir: &Path) -> eyre::Result<()> {
    for (key, value) in [
        ("user.email", "test@example.com"),
        ("user.name", "test"),
        ("commit.gpgsign", "false"),
    ] {
        let output = std::process::Command::new("git")
            .args(["config", key, value])
            .current_dir(dir)
            .output()?;
        eyre::ensure!(output.status.success(), "failed to configure git");
    }
    for args in [vec!["add", "-A"], vec!["commit", "-m", "initial commit"]] {
        let output = std::process::Command::new("git")
            .args(&args)
            .current_dir(dir)
            .output()?;
        eyre::ensure!(
            output.status.success(),
            "failed to run git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

/// `finalize` resumes after replacements and pre-commit checks without applying another bump.
#[test]
fn test_finalize_commits_and_tags_an_applied_bump() -> eyre::Result<()> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join(".bumpversion.toml");
    let initial_config = indoc! {r#"
        [tool.bumpversion]
        current_version = "1.2.3"
        commit = true
        tag = true
        pre_commit_hooks = ['printf "%s->%s\n" "$BVHOOK_CURRENT_VERSION" "$BVHOOK_NEW_VERSION" > Cargo.lock']
        additional_files = ["Cargo.lock"]

        [[tool.bumpversion.files]]
        filename = "VERSION"
    "#};
    fs::write(&config_path, initial_config)?;
    fs::write(temp.path().join("VERSION"), "1.2.3\n")?;
    fs::write(temp.path().join("Cargo.lock"), "1.2.3->pending\n")?;
    git_init(temp.path())?;
    git_commit_all(temp.path())?;

    let tag = std::process::Command::new("git")
        .args(["tag", "v1.2.3"])
        .current_dir(temp.path())
        .output()?;
    eyre::ensure!(
        tag.status.success(),
        "failed to tag previous version: {}",
        String::from_utf8_lossy(&tag.stderr)
    );

    fs::write(&config_path, initial_config.replace("1.2.3", "1.2.4"))?;
    fs::write(temp.path().join("VERSION"), "1.2.4\n")?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path())
        .args(["finalize", "--allow-dirty"]);
    cmd.assert().success();

    assert_eq!(fs::read_to_string(temp.path().join("VERSION"))?, "1.2.4\n");
    assert_eq!(
        fs::read_to_string(temp.path().join("Cargo.lock"))?,
        "1.2.3->1.2.4\n"
    );

    let subject = std::process::Command::new("git")
        .args(["log", "-1", "--format=%s"])
        .current_dir(temp.path())
        .output()?;
    eyre::ensure!(subject.status.success(), "failed to read release commit");
    assert_eq!(
        String::from_utf8(subject.stdout)?.trim(),
        "Bump version: 1.2.3 → 1.2.4"
    );

    let tags = std::process::Command::new("git")
        .args(["tag", "--points-at", "HEAD"])
        .current_dir(temp.path())
        .output()?;
    eyre::ensure!(tags.status.success(), "failed to inspect release tag");
    assert_eq!(String::from_utf8(tags.stdout)?.trim(), "v1.2.4");

    let status = std::process::Command::new("git")
        .args(["status", "--short"])
        .current_dir(temp.path())
        .output()?;
    eyre::ensure!(status.status.success(), "failed to inspect git status");
    assert!(
        status.stdout.is_empty(),
        "finalized repository must be clean"
    );
    Ok(())
}

/// A failed pre-commit check prints its output once and ends with recovery instructions.
#[test]
fn test_pre_commit_failure_has_recovery_guidance() -> eyre::Result<()> {
    let temp = tempfile::tempdir()?;
    fs::write(
        temp.path().join(".bumpversion.toml"),
        indoc! {r#"
            [tool.bumpversion]
            current_version = "1.2.3"
            commit = true
            tag = true
            pre_commit_hooks = ['printf "check stdout\n"; printf "check stderr\n" >&2; exit 7']

            [[tool.bumpversion.files]]
            filename = "VERSION"
        "#},
    )?;
    fs::write(temp.path().join("VERSION"), "1.2.3\n")?;
    git_init(temp.path())?;
    git_commit_all(temp.path())?;

    let tag = std::process::Command::new("git")
        .args(["tag", "v1.2.3"])
        .current_dir(temp.path())
        .output()?;
    eyre::ensure!(tag.status.success(), "failed to tag previous version");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path()).args(["bump", "minor"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "Pre-commit hook failed with exit code 7:",
        ))
        .stderr(predicate::str::contains("Stdout:\ncheck stdout"))
        .stderr(predicate::str::contains("Stderr:\ncheck stderr"))
        .stderr(predicate::str::contains("WARN bumpversion::hooks").not())
        .stderr(predicate::str::contains("Backtrace omitted").not())
        .stderr(predicate::str::ends_with(
            "Either revert them and start over, or fix the issue and run:\n  bumpversion finalize --allow-dirty\n",
        ));

    assert_eq!(fs::read_to_string(temp.path().join("VERSION"))?, "1.3.0\n");
    let tags = std::process::Command::new("git")
        .args(["tag", "--points-at", "HEAD"])
        .current_dir(temp.path())
        .output()?;
    eyre::ensure!(tags.status.success(), "failed to inspect tags");
    assert_eq!(String::from_utf8(tags.stdout)?.trim(), "v1.2.3");
    Ok(())
}

/// Hook scripts were shlex-split before being handed to `sh -c`, which takes the
/// whole script as one argument — so only the first word ran and the rest became
/// positional parameters. Any hook with arguments or a redirect did nothing.
#[test]
fn test_hooks_run_the_whole_script() -> eyre::Result<()> {
    let temp = repo_with(
        ".bumpversion.toml",
        r#"
[tool.bumpversion]
current_version = "1.0.0"
commit = true
tag = true
pre_commit_hooks = ["echo pre > pre.txt"]
post_commit_hooks = ["echo post > post.txt"]
"#,
    )?;
    git_commit_all(temp.path())?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path()).args(["bump", "minor"]);
    cmd.assert().success();

    assert_eq!(fs::read_to_string(temp.path().join("pre.txt"))?, "pre\n");
    assert_eq!(fs::read_to_string(temp.path().join("post.txt"))?, "post\n");
    Ok(())
}

#[test]
fn test_pre_commit_hook_additional_file_reaches_commit() -> eyre::Result<()> {
    let temp = repo_with(
        ".bumpversion.toml",
        indoc! {r#"
            [tool.bumpversion]
            current_version = "1.0.0"
            commit = true
            tag = false
            pre_commit_hooks = ['printf "version=%s\n" "$BVHOOK_NEW_VERSION" > Cargo.lock']
            additional_files = ["Cargo.lock"]

            [[tool.bumpversion.files]]
            filename = "VERSION"
        "#},
    )?;
    fs::write(temp.path().join("VERSION"), "1.0.0")?;
    fs::write(temp.path().join("Cargo.lock"), "version=1.0.0\n")?;
    git_commit_all(temp.path())?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path()).args(["bump", "minor"]);
    cmd.assert().success();

    let committed_lock = std::process::Command::new("git")
        .args(["show", "HEAD:Cargo.lock"])
        .current_dir(temp.path())
        .output()?;
    eyre::ensure!(
        committed_lock.status.success(),
        "failed to read committed Cargo.lock: {}",
        String::from_utf8_lossy(&committed_lock.stderr)
    );
    assert_eq!(String::from_utf8(committed_lock.stdout)?, "version=1.1.0\n");
    let status = std::process::Command::new("git")
        .args(["status", "--short"])
        .current_dir(temp.path())
        .output()?;
    eyre::ensure!(status.status.success(), "failed to inspect git status");
    assert!(
        status.stdout.is_empty(),
        "hook-generated files must not remain outside the bump commit: {}",
        String::from_utf8_lossy(&status.stdout)
    );
    Ok(())
}

/// `BVHOOK_NEW_VERSION_TAG` carried the tag already on the repository instead of
/// the one the bump was about to create.
#[test]
fn test_hook_sees_the_tag_being_created() -> eyre::Result<()> {
    let temp = repo_with(
        ".bumpversion.toml",
        r#"
[tool.bumpversion]
current_version = "1.0.0"
commit = true
tag = true
tag_name = "rel-{new_version}"
post_commit_hooks = ['echo "$BVHOOK_NEW_VERSION_TAG" > saw.txt']
"#,
    )?;
    git_commit_all(temp.path())?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path()).args(["bump", "minor"]);
    cmd.assert().success();

    assert_eq!(
        fs::read_to_string(temp.path().join("saw.txt"))?.trim(),
        "rel-1.1.0",
        "the hook must see the tag being created, not the previous one"
    );
    Ok(())
}

#[test]
fn test_show_current_version() -> eyre::Result<()> {
    // We need to run this in a context with a valid config
    // Let's use a temporary directory and create a config file
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join("pyproject.toml");

    fs::write(
        &config_path,
        r#"
[tool.bumpversion]
current_version = "1.2.3"
"#,
    )?;

    // Initialize a git repo so bumpversion doesn't complain (though show might not strict check it depending on implementation)
    // Actually our implementation checks git unless we handle it, but we removed check_is_dirty for show.
    // However, it still tries to open the repo: `let repo = GitRepository::open(&dir)?;` in common.rs
    // So we must init a git repo.

    git_init(temp.path())?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path())
        .arg("show")
        .arg("current_version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("1.2.3"));
    Ok(())
}

#[test]
fn test_show_bump_major() -> eyre::Result<()> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join(".bumpversion.toml");

    fs::write(
        &config_path,
        r#"
[tool.bumpversion]
current_version = "1.2.3"
"#,
    )?;

    git_init(temp.path())?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path()).arg("show-bump").arg("major");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("old_version=1.2.3"))
        .stdout(predicate::str::contains("new_version=2.0.0"));
    Ok(())
}

#[test]
fn test_values_bump_scenario() -> eyre::Result<()> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join("pyproject.toml");

    fs::write(
        &config_path,
        r#"
[tool.bumpversion]
current_version = "1.0.0"
parse = "(?P<major>\\d+)\\.(?P<minor>\\d+)\\.(?P<patch>\\d+)(?:-(?P<release>[a-z]+))?"
serialize = ["{major}.{minor}.{patch}-{release}", "{major}.{minor}.{patch}"]

[tool.bumpversion.parts.release]
values = ["alpha", "beta", "rc", "final"]
optional_value = "final"
"#,
    )?;

    git_init(temp.path())?;

    // Test bump from 1.0.0 to 1.0.0-alpha (bumping release)
    // Wait, 1.0.0 matches the second pattern. Bumping release (which is currently "final" implicitly?)
    // If optional_value="final", then 1.0.0 is effectively 1.0.0-final.
    // Bumping "final" -> error (max reached).

    // Ah, wait. If we want to go from 1.0.0 to 1.0.0-alpha, we aren't bumping "release" part directly if it's already at max?
    // Actually, usually you bump 'patch' to get 1.0.1, then 'release' to get 1.0.1-alpha?
    // Or if we have 1.0.0-alpha, bumping release gives 1.0.0-beta.

    // Let's test explicit component bumping if we start with pre-release.
    // Reset config to have current_version = "1.0.0-alpha"

    fs::write(
        &config_path,
        r#"
[tool.bumpversion]
current_version = "1.0.0-alpha"
parse = "(?P<major>\\d+)\\.(?P<minor>\\d+)\\.(?P<patch>\\d+)(?:-(?P<release>[a-z]+))?"
serialize = ["{major}.{minor}.{patch}-{release}", "{major}.{minor}.{patch}"]

[tool.bumpversion.parts.release]
values = ["alpha", "beta", "rc", "final"]
optional_value = "final"
"#,
    )?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path()).arg("show-bump").arg("release");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("old_version=1.0.0-alpha"))
        .stdout(predicate::str::contains("new_version=1.0.0-beta"));
    Ok(())
}

#[test]
fn test_bump_modifies_file() -> eyre::Result<()> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join(".bumpversion.toml");
    let source_path = temp.path().join("VERSION");

    fs::write(
        &config_path,
        r#"
[tool.bumpversion]
current_version = "1.2.3"

[[tool.bumpversion.files]]
filename = "VERSION"
"#,
    )?;

    fs::write(&source_path, "1.2.3")?;

    git_init(temp.path())?;

    // We need to configure git user for commit to work (if bumpversion commits by default, which it might not if --no-commit or default is false)
    // Default config: commit = false. So we should be fine without git config unless we enable it.
    // However, to be safe and allow dirty check to pass (or fail if we don't commit), let's see.
    // We'll pass --allow-dirty to avoid git strictness issues in test env.

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path())
        .arg("bump")
        .arg("patch")
        .arg("--allow-dirty")
        .arg("--no-commit")
        .arg("--no-tag");

    cmd.assert().success();

    let content = fs::read_to_string(&source_path)?;
    assert_eq!(content, "1.2.4");

    let config_content = fs::read_to_string(&config_path)?;
    assert!(config_content.contains(r#"current_version = "1.2.4""#));
    Ok(())
}

#[test]
fn test_bump_custom_search_replace() -> eyre::Result<()> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join(".bumpversion.toml");
    let source_path = temp.path().join("VERSION");

    fs::write(
        &config_path,
        r#"
[tool.bumpversion]
current_version = "1.2.3"
search = "my-version: {current_version}"
replace = "my-version: {new_version}"

[[tool.bumpversion.files]]
filename = "VERSION"
"#,
    )?;

    fs::write(&source_path, "my-version: 1.2.3")?;

    git_init(temp.path())?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path())
        .arg("bump")
        .arg("patch")
        .arg("--allow-dirty")
        .arg("--no-commit")
        .arg("--no-tag");

    cmd.assert().success();

    let content = fs::read_to_string(&source_path)?;
    assert_eq!(content, "my-version: 1.2.4");
    Ok(())
}

#[test]
fn test_bump_dry_run() -> eyre::Result<()> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join(".bumpversion.toml");
    let source_path = temp.path().join("VERSION");

    fs::write(
        &config_path,
        r#"
[tool.bumpversion]
current_version = "1.2.3"

[[tool.bumpversion.files]]
filename = "VERSION"
"#,
    )?;

    let initial_content = "1.2.3";
    fs::write(&source_path, initial_content)?;

    git_init(temp.path())?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.current_dir(temp.path())
        .arg("bump")
        .arg("patch")
        .arg("--dry-run")
        .arg("--allow-dirty")
        .arg("--no-commit")
        .arg("--no-tag");

    cmd.assert().success();

    let content = fs::read_to_string(&source_path)?;
    assert_eq!(
        content, initial_content,
        "File should not change in dry-run"
    );

    let config_content = fs::read_to_string(&config_path)?;
    assert!(
        config_content.contains(r#"current_version = "1.2.3""#),
        "Config should not change in dry-run"
    );
    Ok(())
}
