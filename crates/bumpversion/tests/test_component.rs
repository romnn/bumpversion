//! Integration tests for version component bumping.

#![allow(clippy::unnecessary_wraps)]

use bumpversion::{
    config::version::VersionComponentSpec,
    version::Component,
};
use color_eyre::eyre;

#[test]
fn test_numeric_bump() -> eyre::Result<()> {
    let spec = VersionComponentSpec::default();
    let component = Component::new(Some("1"), spec);
    let bumped = component.bump()?;
    assert_eq!(bumped.value(), Some("2"));
    Ok(())
}

#[test]
fn test_numeric_bump_first_value() -> eyre::Result<()> {
    let spec = VersionComponentSpec {
        first_value: Some("1".to_string()),
        ..Default::default()
    };
    let component = Component::new(None, spec.clone());
    // Initial value should be first_value ("1")
    assert_eq!(component.value(), Some("1"));
    
    // Bump should go to 2
    let bumped = component.bump()?;
    assert_eq!(bumped.value(), Some("2"));
    Ok(())
}

#[test]
fn test_values_bump() -> eyre::Result<()> {
    let spec = VersionComponentSpec {
        values: vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()],
        ..Default::default()
    };
    
    let component = Component::new(Some("alpha"), spec.clone());
    let bumped = component.bump()?;
    assert_eq!(bumped.value(), Some("beta"));
    
    let bumped = bumped.bump()?;
    assert_eq!(bumped.value(), Some("gamma"));
    
    assert!(bumped.bump().is_err(), "Should error on max value");
    Ok(())
}

#[test]
fn test_values_optional_value() -> eyre::Result<()> {
    let spec = VersionComponentSpec {
        values: vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()],
        optional_value: Some("gamma".to_string()),
        ..Default::default()
    };
    
    let component = Component::new(None, spec.clone());
    
    // With optional_value set to "gamma", a missing value defaults to "gamma".
    assert_eq!(component.value(), Some("gamma"));
    
    // Bumping "gamma" (last value) should fail
    assert!(component.bump().is_err());
    
    // If we explicitly set it to "alpha", it should bump to "beta"
    let component_alpha = Component::new(Some("alpha"), spec.clone());
    assert_eq!(component_alpha.value(), Some("alpha"));
    let bumped = component_alpha.bump()?;
    assert_eq!(bumped.value(), Some("beta"));
    Ok(())
}

#[test]
fn test_reset_to_first() -> eyre::Result<()> {
    let spec = VersionComponentSpec::default(); // numeric, first_value defaults to "0"
    let component = Component::new(Some("5"), spec);
    let reset = component.first();
    assert_eq!(reset.value(), Some("0"));
    Ok(())
}

#[test]
fn test_reset_to_first_values() -> eyre::Result<()> {
    let spec = VersionComponentSpec {
        values: vec!["a".to_string(), "b".to_string()],
        ..Default::default()
    };
    let component = Component::new(Some("b"), spec);
    let reset = component.first();
    assert_eq!(reset.value(), Some("a"));
    Ok(())
}
