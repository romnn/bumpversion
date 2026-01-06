//! Integration tests for the `bumpversion` CLI binary.

#![allow(clippy::unnecessary_wraps)]

use assert_cmd::Command;
use color_eyre::eyre;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

fn git_init(dir: &Path) -> eyre::Result<()> {
    let output = std::process::Command::new("git")
        .arg("init")
        .current_dir(dir)
        .output()?;
    eyre::ensure!(output.status.success(), "failed to init git repo");
    Ok(())
}

#[test]
fn test_show_help() -> eyre::Result<()> {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.arg("show").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage: bumpversion show"));
    Ok(())
}

#[test]
fn test_show_bump_help() -> eyre::Result<()> {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bumpversion"));
    cmd.arg("show-bump").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage: bumpversion show-bump"));
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
    cmd.current_dir(temp.path())
        .arg("show-bump")
        .arg("major");
        
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
    cmd.current_dir(temp.path())
        .arg("show-bump")
        .arg("release");
        
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
        
    cmd.assert()
        .success();
        
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
        
    cmd.assert()
        .success();
        
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
        
    cmd.assert()
        .success();
        
    let content = fs::read_to_string(&source_path)?;
    assert_eq!(content, initial_content, "File should not change in dry-run");
    
    let config_content = fs::read_to_string(&config_path)?;
    assert!(config_content.contains(r#"current_version = "1.2.3""#), "Config should not change in dry-run");
    Ok(())
}
