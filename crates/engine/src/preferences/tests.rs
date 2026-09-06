use std::collections::BTreeSet;

use super::*;

#[test]
fn active_registry_contains_only_consumer_ready_product_rows() {
    let registry = active_v1_registry();
    let actual: BTreeSet<_> = registry.keys().map(PreferenceKey::as_str).collect();
    let expected = BTreeSet::from([
        "datum.console.feedback_duration",
        "datum.accessibility.reduced_motion",
        "datum.accessibility.high_contrast_noncolor",
        "datum.units.system",
        "datum.units.board_length",
        "datum.units.board_length_precision",
        "datum.units.drill_hole",
        "datum.units.drill_hole_precision",
        "datum.units.schematic_geometry",
        "datum.units.schematic_geometry_precision",
        "datum.units.angle_precision",
    ]);
    assert_eq!(actual, expected);
    assert!(
        registry
            .keys()
            .all(|key| !key.as_str().starts_with("datum.revision."))
    );
    assert_eq!(
        registry
            .resolve_alias("console_duration")
            .map(PreferenceKey::as_str),
        Some("datum.console.feedback_duration")
    );
    assert!(registry.resolve_alias("datum.schematic.theme").is_none());
    let reserved = reserved_v1_registry();
    assert_eq!(reserved.len(), 56);
    assert!(
        reserved
            .get(&PreferenceKey::parse("datum.projects.unit_policy_seed").unwrap())
            .is_some()
    );
    assert!(
        registry
            .get(&PreferenceKey::parse("datum.projects.unit_policy_seed").unwrap())
            .is_none()
    );
}

#[test]
fn duplicate_live_identity_and_alias_collision_are_refused() {
    let descriptor = reserved_v1_registry()
        .get(&PreferenceKey::parse("datum.viewport.snap_enabled").unwrap())
        .unwrap()
        .clone();
    let mut registry = DescriptorRegistry::default();
    registry.register(descriptor.clone()).unwrap();
    assert!(matches!(
        registry.register(descriptor),
        Err(RegistrationRefusal::DuplicateKey(_))
    ));
}

#[test]
fn source_and_control_boundaries_follow_catalog_classes() {
    let registry = active_v1_registry();
    let accessibility = registry
        .get(&PreferenceKey::parse("datum.accessibility.reduced_motion").unwrap())
        .unwrap();
    assert!(
        !accessibility
            .allowed_sources
            .contains(&ResolutionSource::Organization)
    );
    assert!(accessibility.allowed_directives.is_empty());

    let seed = registry
        .get(&PreferenceKey::parse("datum.units.system").unwrap())
        .unwrap();
    assert_eq!(seed.class, SettingClass::ProjectPolicySeed);
    assert!(!seed.allowed_sources.contains(&ResolutionSource::Session));
    assert!(!seed.allowed_sources.contains(&ResolutionSource::Context));
}
