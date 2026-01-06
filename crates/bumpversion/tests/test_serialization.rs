//! Integration tests for version parsing and serialization.

use bumpversion::{
    config::version::VersionComponentSpec,
    f_string::PythonFormatString,
    version::{Version, VersionSpec},
};
use color_eyre::eyre;
use indexmap::IndexMap;
use std::collections::HashMap;
use similar_asserts::assert_eq;

fn semver_spec() -> VersionSpec {
    let mut components = IndexMap::new();
    components.insert("major".to_string(), VersionComponentSpec::default());
    components.insert("minor".to_string(), VersionComponentSpec::default());
    components.insert("patch".to_string(), VersionComponentSpec::default());
    VersionSpec::from_components(components)
}

fn create_version(spec: &VersionSpec, parts: &[(&str, &str)]) -> Version {
    let raw: HashMap<&str, &str> = parts.iter().copied().collect();
    spec.build(&raw)
}

#[test]
fn test_parse_version_empty() -> eyre::Result<()> {
    let spec = semver_spec();
    let regex = regex::Regex::new(r"(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)")?;
    
    assert!(Version::parse("", &regex, &spec).is_none());
    Ok(())
}

#[test]
fn test_parse_version_semver() -> eyre::Result<()> {
    let spec = semver_spec();
    // Python test uses SEMVER_PATTERN, we'll use a simplified one matching the components we have
    let regex = regex::Regex::new(r"(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)")?;
    
    let version = Version::parse("1.2.3", &regex, &spec)
        .ok_or_else(|| eyre::eyre!("expected version to parse"))?;
    
    assert_eq!(
        version
            .get("major")
            .ok_or_else(|| eyre::eyre!("missing major component"))?
            .value(),
        Some("1")
    );
    assert_eq!(
        version
            .get("minor")
            .ok_or_else(|| eyre::eyre!("missing minor component"))?
            .value(),
        Some("2")
    );
    assert_eq!(
        version
            .get("patch")
            .ok_or_else(|| eyre::eyre!("missing patch component"))?
            .value(),
        Some("3")
    );
    Ok(())
}

#[test]
fn test_serialize_format_selection() -> eyre::Result<()> {
    let spec = semver_spec();
    let version = create_version(&spec, &[("major", "1"), ("minor", "2"), ("patch", "3")]);
    
    let patterns: Vec<PythonFormatString> = vec![
        "{major}.{minor}.{patch}".parse()?,
        "{major}.{minor}".parse()?,
        "{major}".parse()?,
    ];
    
    let ctx: HashMap<String, String> = HashMap::new();
    let serialized = version.serialize(&patterns, &ctx)?;
    assert_eq!(serialized, "1.2.3");
    Ok(())
}

#[test]
fn test_serialize_format_selection_shorter() -> eyre::Result<()> {
    let spec = semver_spec();
    // 1.2.0 -> "1.2" if available
    let version = create_version(&spec, &[("major", "1"), ("minor", "2"), ("patch", "0")]);
    
    let patterns: Vec<PythonFormatString> = vec![
        "{major}.{minor}.{patch}".parse()?,
        "{major}.{minor}".parse()?,
        "{major}".parse()?,
    ];
    
    let ctx: HashMap<String, String> = HashMap::new();
    // In Python: test_is_string_with_fewest_required_labels
    // It picks the shortest one that contains all *required* (non-optional/default) labels?
    // Actually, Version::serialize sorts by:
    // 1. has_required_components (desc)
    // 2. num_labels (asc)
    // 3. index (asc)
    
    // For 1.2.0, patch is 0. If 0 is the default/first value, is it required?
    // Version::required_component_names returns components where value != optional_value (or first_value?)
    // In Rust impl: filter(|(_, v)| v.value() != v.spec.optional_value.as_deref())
    
    // By default first_value is "0" (via Component::new logic we added).
    // So if patch is "0", it is NOT required.
    // So "1.2.0" has required: major="1", minor="2".
    
    // Patterns:
    // 1. {major}, {minor}, {patch} -> 3 labels. Covers required {major, minor}.
    // 2. {major}, {minor} -> 2 labels. Covers required {major, minor}.
    // 3. {major} -> 1 label. MISSING minor.
    
    // So it should pick pattern 2: "1.2"
    
    let serialized = version.serialize(&patterns, &ctx)?;
    assert_eq!(serialized, "1.2");
    Ok(())
}

#[test]
fn test_serialize_format_selection_shortest() -> eyre::Result<()> {
    let spec = semver_spec();
    // 1.0.0 -> "1"
    let version = create_version(&spec, &[("major", "1"), ("minor", "0"), ("patch", "0")]);
    
    let patterns: Vec<PythonFormatString> = vec![
        "{major}.{minor}.{patch}".parse()?,
        "{major}.{minor}".parse()?,
        "{major}".parse()?,
    ];
    
    let ctx: HashMap<String, String> = HashMap::new();
    let serialized = version.serialize(&patterns, &ctx)?;
    assert_eq!(serialized, "1");
    Ok(())
}

#[test]
fn test_serialize_with_newlines() -> eyre::Result<()> {
    let spec = semver_spec();
    let version = create_version(&spec, &[("major", "31"), ("minor", "0"), ("patch", "3")]);
    
    let patterns: Vec<PythonFormatString> = vec![
        PythonFormatString::parse("MAJOR={major}\nMINOR={minor}\nPATCH={patch}\n")?
    ];
    
    let ctx: HashMap<String, String> = HashMap::new();
    let serialized = version.serialize(&patterns, &ctx)?;
    assert_eq!(serialized, "MAJOR=31\nMINOR=0\nPATCH=3\n");
    Ok(())
}
