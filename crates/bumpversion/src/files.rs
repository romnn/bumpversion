//! File operations for searching and replacing version strings in project files.
//!
//! Handles reading, modifying, and writing files based on configuration.
use crate::{
    config::{self, FileChange, InputFile, VersionComponentConfigs},
    f_string::{self, PythonFormatString},
    version::{self, Version},
};
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

// /// Does the search pattern match any part of the contents?
// fn contains_pattern(contents: &str, search_pattern: &regex::Regex) -> bool {
//     let matches = search_pattern.captures_iter(contents);
//     let Some(m) = matches.into_iter().next() else {
//         return false;
//     };
//     let Some(m) = m.iter().next().flatten() else {
//         return false;
//     };
//     let line_num = contents[..m.start()].chars().filter(|c| *c == '\n').count() + 1;
//     tracing::info!(
//         "found {:?} at line {}: {:?}",
//         search_pattern.as_str(),
//         line_num,
//         m.as_str(),
//     );
//     true
// }

/// Errors that can occur when replacing version strings in files.
#[derive(thiserror::Error, Debug)]
pub enum ReplaceVersionError {
    #[error(transparent)]
    Io(#[from] IoError),
    #[error(transparent)]
    Serialize(#[from] version::SerializeError),
    #[error(transparent)]
    MissingArgument(#[from] f_string::MissingArgumentError),
    #[error(transparent)]
    InvalidFormatString(#[from] f_string::ParseError),

    #[error(transparent)]
    RegexTemplate(#[from] config::regex::RegexTemplateError),
    #[error(transparent)]
    Toml(#[from] toml_edit::TomlError),
}

/// Apply a list of `changes` to the input `before` content,
/// producing the modified text and a record of replacements.
///
/// # Errors
/// Returns `ReplaceVersionError` if serialization, I/O, or formatting fails.
pub fn replace_version<'a, K, V, S>(
    before: String,
    changes: &'a [FileChange],
    current_version: &'a Version,
    new_version: &'a Version,
    ctx: &'a HashMap<K, V, S>,
) -> Result<Modification, ReplaceVersionError>
where
    K: std::borrow::Borrow<str> + std::hash::Hash + Eq + std::fmt::Debug,
    V: AsRef<str> + std::fmt::Debug,
    S: std::hash::BuildHasher,
{
    let mut after = before.to_string();
    let mut replacements = vec![];
    for change in changes {
        tracing::debug!(
            search = ?change.search,
            replace = ?change.replace,
            "update",
        );

        // we need to update the version because each file may serialize versions differently
        let current_version_serialized =
            current_version.serialize(&change.serialize_version_patterns, ctx)?;
        let new_version_serialized =
            new_version.serialize(&change.serialize_version_patterns, ctx)?;

        let ctx: HashMap<&str, &str> = ctx
            .iter()
            .map(|(k, v)| (k.borrow(), v.as_ref()))
            .chain([
                ("current_version", current_version_serialized.as_str()),
                ("new_version", new_version_serialized.as_str()),
            ])
            .collect();

        let search_pattern = &change.search;
        let search_regex = search_pattern.format(&ctx, true)?;

        let replace_pattern = &change.replace;
        let replacement = PythonFormatString::parse(replace_pattern)?;
        let replacement = replacement.format(&ctx, true)?;

        // TODO(roman): i don't think we need to check if the change pattern is present?
        // // does the file contain the change pattern?
        // let contains_change_pattern: bool = {
        //     if contains_pattern(&before, &search_regex) {
        //         return Ok(true);
        //     }
        //
        //     // The `search` pattern did not match, but the original supplied
        //     // version number (representing the same version component values) might
        //     // match instead. This is probably the case if environment variables are used.
        //     let file_uses_global_search_pattern = file_change.search == version_config.search;
        //
        //     let pattern = regex::RegexBuilder::new(regex::escape(version.original)).build()?;
        //
        //     if file_uses_global_search_pattern && contains_pattern(&before, &pattern) {
        //         // The original version is present, and we're not looking for something
        //         // more specific -> this is accepted as a match
        //         return Ok(true);
        //     }
        //
        //     Ok(false)
        // }?;
        //
        // if !contains_change_pattern {
        //     if file_change.ignore_missing_version {
        //         tracing::warn!("did not find {:?} in file {path:?}", search_regex.as_str());
        //     } else {
        //         eyre::bail!("did not find {:?} in file {path:?}", search_regex.as_str());
        //     }
        //     return Ok(());
        // }

        after = search_regex.replace_all(&after, &replacement).to_string();

        replacements.push(Replacement {
            search_pattern: search_pattern.to_string(),
            search: search_regex.as_str().to_string(),
            replace_pattern: replace_pattern.clone(),
            replace: replacement,
        });
    }

    let modification = Modification {
        before,
        after,
        replacements,
    };
    Ok(modification)
}

/// A single substitution made during version replacement.
#[derive(Debug)]
pub struct Replacement {
    /// The regex string used to search for the existing version.
    pub search: String,
    /// Original search template from configuration before formatting.
    pub search_pattern: String,
    /// Replacement string generated with the new version.
    pub replace: String,
    /// Template used for replacement before formatting.
    pub replace_pattern: String,
}

/// Represents the overall result of modifying a file.
#[derive(Debug)]
pub struct Modification {
    /// Original file content before any replacements.
    pub before: String,
    /// File content after applying all replacements.
    pub after: String,
    /// Detailed list of individual replacements performed.
    pub replacements: Vec<Replacement>,
}

impl Modification {
    /// Generate a unified diff between original and modified content.
    ///
    /// If `path` is provided, include labels in the diff header.
    #[must_use]
    pub fn diff(&self, path: Option<&Path>) -> Option<String> {
        if self.before == self.after {
            None
        } else {
            let (label_before, label_after) = if let Some(path) = path {
                (
                    format!("{} (before)", path.display()),
                    format!("{} (after)", path.display()),
                )
            } else {
                ("before".to_string(), "after".to_string())
            };
            let diff = similar_asserts::SimpleDiff::from_str(
                &self.before,
                &self.after,
                &label_before,
                &label_after,
            );
            Some(diff.to_string())
        }
    }
}

/// Read a file at `path`, apply version replacement, and write back if changed.
///
/// Honors `dry_run` to skip writing. Returns `None` if file unchanged or missing (when allowed).
pub async fn replace_version_in_file<K, V, S>(
    path: &Path,
    changes: &[FileChange],
    current_version: &Version,
    new_version: &Version,
    ctx: &HashMap<K, V, S>,
    dry_run: bool,
) -> Result<Option<Modification>, ReplaceVersionError>
where
    K: std::borrow::Borrow<str> + std::hash::Hash + Eq + std::fmt::Debug,
    V: AsRef<str> + std::fmt::Debug,
    S: std::hash::BuildHasher,
{
    let as_io_error = |source: std::io::Error| -> IoError { IoError::new(source, path) };
    if !path.is_file() {
        if changes.iter().all(|change| change.ignore_missing_file) {
            tracing::info!(?path, "file not found");
            return Ok(None);
        }
        let not_found = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        return Err(ReplaceVersionError::from(as_io_error(not_found)));
    }

    let before = tokio::fs::read_to_string(path).await.map_err(as_io_error)?;
    let modification = replace_version(before, changes, current_version, new_version, ctx)?;

    if modification.before == modification.after {
        // tracing::warn!(?path, "no change after version replacement");
        // TODO(roman): can we also not do this?
        // && current_version.original {
        // og_context = deepcopy(context)
        // og_context["current_version"] = current_version.original
        // search_for_og, _ = self.file_change.get_search_pattern(og_context)
        // file_content_after = search_for_og.sub(replace_with, file_content_before)
        // return Ok(());
    }

    if !dry_run {
        use tokio::io::AsyncWriteExt;
        let file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(false)
            .truncate(true)
            .open(path)
            .await
            .map_err(as_io_error)?;
        let mut writer = tokio::io::BufWriter::new(file);
        writer
            .write_all(modification.after.as_bytes())
            .await
            .map_err(as_io_error)?;
        writer.flush().await.map_err(as_io_error)?;
    }
    Ok(Some(modification))
}

/// Errors encountered when resolving glob patterns to file paths.
#[derive(thiserror::Error, Debug)]
pub enum GlobError {
    #[error(transparent)]
    Pattern(#[from] glob::PatternError),
    #[error(transparent)]
    Glob(#[from] glob::GlobError),
}

/// I/O error with optional path context.
#[derive(thiserror::Error, Debug)]
pub struct IoError {
    /// Underlying I/O error.
    #[source]
    pub source: std::io::Error,
    /// Path of the file that caused the error, if available.
    pub path: Option<PathBuf>,
}

impl IoError {
    pub fn new(source: impl Into<std::io::Error>, path_or_stream: impl Into<PathBuf>) -> Self {
        Self {
            source: source.into(),
            path: Some(path_or_stream.into()),
        }
    }
}

impl std::fmt::Display for IoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.path {
            Some(path) => write!(f, "io error for {}", path.display()),
            None => write!(f, "io error"),
        }
    }
}

/// Combined error type for file resolution operations.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Glob(#[from] GlobError),
    #[error(transparent)]
    Io(#[from] IoError),
}

/// Return a list of file configurations that match the glob pattern
fn resolve_glob_files(
    pattern: &str,
    exclude_patterns: &[String],
) -> Result<Vec<PathBuf>, GlobError> {
    let options = glob::MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    let included: HashSet<PathBuf> =
        glob::glob_with(pattern, options)?.collect::<Result<_, _>>()?;

    let excluded: HashSet<PathBuf> = exclude_patterns
        .iter()
        .map(|pattern| glob::glob_with(pattern, options))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flat_map(std::iter::IntoIterator::into_iter)
        .collect::<Result<_, _>>()?;

    Ok(included.difference(&excluded).cloned().collect())
}

/// Mapping from file paths to the list of version `FileChange`s to apply.
pub type FileMap = IndexMap<PathBuf, Vec<FileChange>>;

/// Build the file map from `config`, expanding glob patterns and relative paths.
///
/// Applies `parts` (version component configs) and resolves paths under `base_dir`.
pub fn resolve_files_from_config(
    config: &mut config::FinalizedConfig,
    parts: &VersionComponentConfigs,
    base_dir: Option<&Path>,
) -> Result<FileMap, Error> {
    let files = config.files.drain(..);
    let new_files: Vec<_> = files
        .into_iter()
        .map(|(file, file_config)| {
            let new_files = match file {
                InputFile::GlobPattern {
                    pattern,
                    exclude_patterns,
                } => resolve_glob_files(&pattern, exclude_patterns.as_deref().unwrap_or_default()),
                InputFile::Path(path) => Ok(vec![path.clone()]),
            }?;

            let file_change = FileChange::new(file_config, parts);
            Ok(new_files
                .into_iter()
                .map(|file| {
                    if file.is_absolute() {
                        Ok(file)
                    } else if let Some(base_dir) = base_dir {
                        let file = base_dir.join(&file);
                        file.canonicalize()
                            .map_err(|source| IoError::new(source, file))
                    } else {
                        Ok(file)
                    }
                })
                .map(move |file| file.map(|file| (file, file_change.clone()))))
        })
        .collect::<Result<_, Error>>()?;

    let new_files = new_files.into_iter().flatten().try_fold(
        IndexMap::<PathBuf, Vec<FileChange>>::new(),
        |mut acc, res| {
            let (file, config) = res?;
            acc.entry(file).or_default().push(config);
            Ok::<_, Error>(acc)
        },
    )?;
    Ok(new_files)
}

/// Filter the `file_map` based on `config` include/exclude settings.
pub fn files_to_modify(
    config: &config::FinalizedConfig,
    file_map: FileMap,
) -> impl Iterator<Item = (PathBuf, Vec<FileChange>)> + use<'_> {
    let excluded_paths_from_config: HashSet<&PathBuf> = config
        .global
        .excluded_paths
        .as_deref()
        .unwrap_or_default()
        .iter()
        .collect();

    let included_paths_from_config: HashSet<&PathBuf> = config
        .global
        .included_paths
        .as_deref()
        .unwrap_or_default()
        .iter()
        .collect();

    let included_files: HashSet<&PathBuf> = file_map
        .keys()
        .collect::<HashSet<&PathBuf>>()
        .difference(&excluded_paths_from_config)
        .copied()
        .collect();

    let included_files: HashSet<PathBuf> = included_paths_from_config
        .union(&included_files)
        .copied()
        .cloned()
        .collect();

    file_map
        .into_iter()
        .filter(move |(file, _)| included_files.contains(file))
}
