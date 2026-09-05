//! Common logic for the bumpversion CLI and subcommands.
//!
//! Sets up logging, loads configuration, and orchestrates the bump process.
use crate::error::Error;
use crate::options;
use bumpversion::{
    config,
    vcs::{TagAndRevision, VersionControlSystem, git::GitRepository},
};
use std::process::ExitCode;

/// Prints a CLI result and returns the corresponding process exit code.
pub(crate) fn report_result(result: Result<(), Error>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprint!("{}", crate::error::render(&error));
            ExitCode::FAILURE
        }
    }
}

/// Ensure the working directory is clean, unless `allow_dirty` is set.
///
/// # Errors
/// Returns [`Error::Dirty`] if the repository has uncommitted changes the config does not allow.
async fn check_is_dirty(
    repo: &GitRepository,
    config: &config::FinalizedConfig,
) -> Result<(), Error> {
    let dirty_files = repo.dirty_files().await?;
    if !config.global.allow_dirty && !dirty_files.is_empty() {
        // Relative to the repository: the common prefix tells the user nothing.
        let files = dirty_files
            .iter()
            .map(|file| file.strip_prefix(repo.path()).unwrap_or(file).to_path_buf())
            .collect();
        return Err(Error::Dirty { files });
    }

    Ok(())
}

/// Entry point for the `bumpversion` CLI.
///
/// Processes command-line `options`, loads the project config, and performs the bump.
///
/// # Errors
///
/// Returns the [`Error`] that [`report_result`] renders for the user.
pub async fn bumpversion(mut options: options::Options) -> Result<(), Error> {
    let start = std::time::Instant::now();

    let color_choice = options.color_choice.unwrap_or(termcolor::ColorChoice::Auto);
    let use_color = crate::logging::setup(options.log_level, color_choice)?;
    colored::control::set_override(use_color);

    let cwd = std::env::current_dir().map_err(Error::CurrentDir)?;
    let dir = options.dir.as_deref().unwrap_or(&cwd);
    let dir = dir.canonicalize().map_err(|source| Error::ResolveDir {
        path: dir.to_path_buf(),
        source,
    })?;
    let repo = GitRepository::open(&dir)?;

    let printer = bumpversion::diagnostics::Printer::stderr(color_choice.into());

    let cli_overrides = options::global_cli_config(&options)?;
    // An explicitly named config file is resolved relative to the working
    // directory, not `--dir`, so `--dir sub --config-file my.toml` behaves the way
    // a shell path does.
    let config_file = options.config_file.as_deref();
    if let Some(path) = config_file
        && !path.is_file()
    {
        return Err(Error::ConfigFileNotFound {
            path: path.to_path_buf(),
        });
    }
    let (config_file_path, mut config) =
        bumpversion::find_config(&dir, config_file, &cli_overrides, &printer)
            .await?
            .ok_or_else(|| match config_file {
                Some(path) => Error::ConfigFileEmpty {
                    path: path.to_path_buf(),
                },
                None => Error::ConfigNotFound { dir: dir.clone() },
            })?;

    let components = config::version::version_component_configs(&config);
    let (bump, cli_files) = options::parse_positional_arguments(&mut options, &components)?;

    let TagAndRevision { tag, revision } = repo
        .latest_tag_and_revision(
            &config.global.tag_name,
            &config.global.parse_version_pattern,
        )
        .await?;

    tracing::debug!(?tag, "current");
    tracing::debug!(?revision, "current");

    let configured_version = &config.global.current_version;
    let actual_version = tag.as_ref().map(|tag| &tag.current_version).cloned();

    // if both versions are present, they should match
    if let Some((configured_version, actual_version)) =
        configured_version.as_ref().zip(actual_version.as_ref())
        && configured_version != actual_version
    {
        tracing::warn!(
            "version {configured_version} from config does not match last tagged version ({actual_version})",
        );
    }

    let is_read_only_command = matches!(
        options.command,
        Some(options::SubCommand::Show(_) | options::SubCommand::ShowBump(_))
    );

    if !is_read_only_command {
        check_is_dirty(&repo, &config).await?;
    }

    // build resolved file map
    let file_map =
        bumpversion::files::resolve_files_from_config(&mut config, &components, Some(repo.path()))?;

    if options.no_configured_files == Some(true) {
        config.global.excluded_paths = Some(file_map.keys().cloned().collect());
    }

    if !cli_files.is_empty() {
        // file_map.extend(cli_files);
        // config.add_files(files);
        config.global.included_paths = Some(cli_files);
    }

    let verbosity: bumpversion::logging::Verbosity = if options.verbosity.quiet > 0 {
        bumpversion::logging::Verbosity::Off
    } else {
        options.verbosity.verbose.into()
    };

    let logger = crate::verbose::Logger::new(verbosity).dry_run(config.global.dry_run);
    let manager = bumpversion::BumpVersion {
        repo,
        config,
        logger,
        tag_and_revision: TagAndRevision { tag, revision },
        file_map,
        components,
        config_file: Some(config_file_path),
    };

    if let Some(command) = options.command
        && handle_subcommand(command, &manager).await?
    {
        tracing::info!(elapsed = ?start.elapsed(), "done");
        return Ok(());
    }

    let bump = if let Some(new_version) = options.new_version.as_deref() {
        bumpversion::Bump::NewVersion(new_version)
    } else {
        let bump = bump.as_deref().ok_or(Error::MissingComponent)?;
        bumpversion::Bump::Component(bump)
    };

    manager.bump(bump).await?;

    tracing::info!(elapsed = ?start.elapsed(), "done");
    Ok(())
}

async fn handle_subcommand<L>(
    command: options::SubCommand,
    manager: &bumpversion::BumpVersion<GitRepository, L>,
) -> Result<bool, Error>
where
    L: bumpversion::logging::Log,
{
    match command {
        options::SubCommand::Show(show_options) => {
            handle_show(&show_options, manager)?;
            Ok(true)
        }
        options::SubCommand::ShowBump(show_bump_options) => {
            handle_show_bump(&show_bump_options, manager)?;
            Ok(true)
        }
        options::SubCommand::Finalize => {
            manager.finalize().await?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn handle_show<VCS, L>(
    options: &options::ShowOptions,
    manager: &bumpversion::BumpVersion<VCS, L>,
) -> Result<(), Error>
where
    VCS: VersionControlSystem,
    L: bumpversion::logging::Log,
{
    let current_version_serialized = manager
        .config
        .global
        .current_version
        .as_ref()
        .ok_or(Error::MissingCurrentVersion)?;

    let parse_version_pattern = &manager.config.global.parse_version_pattern;
    let version_spec =
        bumpversion::version::VersionSpec::from_components(manager.components.clone());
    let current_version = bumpversion::version::Version::parse(
        current_version_serialized,
        parse_version_pattern,
        &version_spec,
    );

    let ctx: std::collections::HashMap<String, String> = bumpversion::context::get_context(
        Some(&manager.tag_and_revision),
        current_version.as_ref(),
        None,
        Some(current_version_serialized),
        None,
    )
    .collect();

    // Also include flatten config in the context
    // This is a simplification; ideally we merge specific config fields
    let files = manager
        .file_map
        .keys()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("\n");

    let extra_ctx = [("files".to_string(), files)];

    let ctx: std::collections::HashMap<&str, &str> = ctx
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .chain(extra_ctx.iter().map(|(k, v)| (k.as_str(), v.as_str())))
        .collect();

    for variable in &options.variables {
        if let Some(value) = ctx.get(variable.as_str()) {
            if options.variables.len() > 1 {
                println!("{variable}={value}");
            } else {
                println!("{value}");
            }
        } else {
            // Check if it's a config value
            // This is hacky, but consistent with some python behavior
            // For now, we only support context variables and basic config
            tracing::warn!("variable {variable} not found in context");
        }
    }

    Ok(())
}

fn handle_show_bump<VCS, L>(
    options: &options::ShowBumpOptions,
    manager: &bumpversion::BumpVersion<VCS, L>,
) -> Result<(), Error>
where
    VCS: VersionControlSystem,
    L: bumpversion::logging::Log,
{
    let component = options
        .component
        .as_deref()
        .or(options.args.first().map(std::string::String::as_str))
        .ok_or(Error::MissingComponent)?;

    let current_version_serialized = manager
        .config
        .global
        .current_version
        .as_ref()
        .ok_or(Error::MissingCurrentVersion)?;

    let parse_version_pattern = &manager.config.global.parse_version_pattern;
    let version_spec =
        bumpversion::version::VersionSpec::from_components(manager.components.clone());
    let current_version = bumpversion::version::Version::parse(
        current_version_serialized,
        parse_version_pattern,
        &version_spec,
    )
    .ok_or_else(|| Error::InvalidCurrentVersion {
        version: current_version_serialized.clone(),
    })?;

    let new_version = current_version.bump(component)?;
    let serialize_version_patterns = &manager.config.global.serialize_version_patterns;

    // We need a context to serialize
    let ctx_without_new_version: std::collections::HashMap<String, String> =
        bumpversion::context::get_context(
            Some(&manager.tag_and_revision),
            Some(&current_version),
            None,
            Some(current_version_serialized),
            None,
        )
        .collect();

    let new_version_serialized =
        new_version.serialize(serialize_version_patterns, &ctx_without_new_version)?;

    // Mimic bump-my-version output format
    // It usually prints a list of changes or the new version details.
    // Let's print the basic info for now.
    println!("old_version={current_version_serialized}");
    println!("new_version={new_version_serialized}");

    Ok(())
}
