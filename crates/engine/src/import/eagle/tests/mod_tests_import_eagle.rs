use super::*;
use std::fs;

use crate::api::Engine;
use crate::ir::serialization::to_json_deterministic;

#[test]
fn imports_symbol_package_entity_and_part() {
    let pool = import_library_str(SIMPLE_EAGLE_LIBRARY).expect("fixture should import");

    assert_eq!(pool.units.len(), 1);
    assert_eq!(pool.symbols.len(), 1);
    assert_eq!(pool.entities.len(), 2);
    assert_eq!(pool.packages.len(), 2);
    assert_eq!(pool.parts.len(), 2);
    assert_eq!(pool.padstacks.len(), 6);

    let unit = pool.units.values().next().unwrap();
    assert_eq!(unit.name, "OPAMP");
    assert_eq!(unit.pins.len(), 3);

    let entity = pool
        .entities
        .values()
        .find(|entity| entity.name == "LMV321")
        .unwrap();
    assert_eq!(entity.name, "LMV321");
    assert_eq!(entity.prefix, "U");
    assert_eq!(entity.gates.len(), 1);

    let package = pool
        .packages
        .values()
        .find(|package| package.name == "SOT23-5")
        .unwrap();
    assert_eq!(package.name, "SOT23-5");
    assert_eq!(package.pads.len(), 3);
    assert_eq!(package.silkscreen.len(), 1);

    let part = pool
        .parts
        .values()
        .find(|part| part.value == "LMV321")
        .unwrap();
    assert_eq!(part.entity, entity.uuid);
    assert_eq!(part.package, package.uuid);
    assert_eq!(part.pad_map.len(), 3);
}

#[test]
fn import_is_deterministic_for_same_library() {
    let a = import_library_str(SIMPLE_EAGLE_LIBRARY).expect("fixture should import");
    let b = import_library_str(SIMPLE_EAGLE_LIBRARY).expect("fixture should import");

    assert_eq!(a, b);
}

#[test]
fn rejects_unknown_connect_symbol_binding() {
    let broken = SIMPLE_EAGLE_LIBRARY.replace("pad=\"3\"", "pad=\"99\"");
    let err = import_library_str(&broken).expect_err("broken fixture must fail");
    let msg = err.to_string();
    assert!(msg.contains("unknown pad 99"), "unexpected error: {msg}");
}

#[test]
fn imports_multi_gate_device_and_through_hole_pad() {
    let pool = import_library_str(DUAL_GATE_EAGLE_LIBRARY).expect("fixture should import");

    assert_eq!(pool.units.len(), 1);
    assert_eq!(pool.symbols.len(), 1);
    assert_eq!(pool.entities.len(), 1);
    assert_eq!(pool.packages.len(), 1);
    assert_eq!(pool.parts.len(), 1);

    let entity = pool.entities.values().next().unwrap();
    assert_eq!(entity.gates.len(), 2);

    let package = pool.packages.values().next().unwrap();
    assert_eq!(package.pads.len(), 4);
    assert!(package.pads.values().any(|pad| {
        package
            .pads
            .values()
            .find(|p| p.uuid == pad.uuid)
            .map(|_| true)
            .unwrap_or(false)
    }));

    let through_hole_count = package
        .pads
        .values()
        .filter(|pad| {
            pool.padstacks
                .get(&pad.padstack)
                .map(|stack| stack.name.starts_with("th:"))
                .unwrap_or(false)
        })
        .count();
    assert_eq!(through_hole_count, 4);

    let part = pool.parts.values().next().unwrap();
    assert_eq!(part.pad_map.len(), 4);
}

#[test]
fn engine_import_eagle_library_returns_report_and_indexes_parts() {
    let mut engine = Engine::new().expect("engine should initialize");
    let path = fixture_path("simple-opamp.lbr");

    let report = engine
        .import_eagle_library(&path)
        .expect("fixture should import through engine facade");

    assert_eq!(report.kind, ImportKind::EagleLibrary);
    assert_eq!(report.counts.units, 1);
    assert_eq!(report.counts.parts, 2);
    assert!(report.warnings.is_empty());
    assert_eq!(
        report.metadata.get("library_name").map(String::as_str),
        Some("demo-analog")
    );

    let search = engine
        .search_pool("SOT23")
        .expect("pool search should work");
    assert_eq!(search.len(), 1);
    let alt_search = engine
        .search_pool("ALTAMP")
        .expect("pool search should work");
    assert_eq!(alt_search.len(), 1);
}

#[test]
fn eagle_fixture_corpus_imports_and_is_deterministic() {
    let fixtures = corpus_fixture_paths();
    assert!(
        fixtures.len() >= 20,
        "expected at least 20 Eagle library fixtures, found {}",
        fixtures.len()
    );

    for path in fixtures {
        let xml = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let a = import_library_str(&xml)
            .unwrap_or_else(|err| panic!("failed to import {}: {err}", path.display()));
        let b = import_library_str(&xml)
            .unwrap_or_else(|err| panic!("failed to re-import {}: {err}", path.display()));

        assert_eq!(
            a,
            b,
            "import must be deterministic for fixture {}",
            path.display()
        );
    }
}

#[test]
fn eagle_fixture_corpus_canonicalizes_deterministically() {
    let fixtures = corpus_fixture_paths();
    assert!(
        fixtures.len() >= 20,
        "expected at least 20 Eagle library fixtures, found {}",
        fixtures.len()
    );

    for path in fixtures {
        let xml = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let pool = import_library_str(&xml)
            .unwrap_or_else(|err| panic!("failed to import {}: {err}", path.display()));
        let json_a = serde_json::to_string(&pool)
            .unwrap_or_else(|err| panic!("failed to serialize {}: {err}", path.display()));
        let json_b = serde_json::to_string(&pool)
            .unwrap_or_else(|err| panic!("failed to reserialize {}: {err}", path.display()));
        assert_eq!(
            json_a,
            json_b,
            "canonical serialization must be stable for {}",
            path.display()
        );
    }
}

#[test]
fn eagle_golden_subset_matches_checked_in_canonical_json() {
    let subset = ["simple-opamp.lbr", "dual-nand.lbr", "regulator-sot223.lbr"];

    for fixture in subset {
        let fixture_path = fixture_path(fixture);
        let xml = fs::read_to_string(&fixture_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", fixture_path.display()));
        let pool = import_library_str(&xml)
            .unwrap_or_else(|err| panic!("failed to import {}: {err}", fixture_path.display()));
        let canonical = to_json_deterministic(&pool)
            .unwrap_or_else(|err| panic!("failed to serialize {}: {err}", fixture_path.display()));
        let golden_path = golden_path_for_fixture(fixture);

        if std::env::var_os("UPDATE_GOLDENS").is_some() {
            if let Some(parent) = golden_path.parent() {
                fs::create_dir_all(parent).unwrap_or_else(|err| {
                    panic!("failed to create golden dir {}: {err}", parent.display())
                });
            }
            fs::write(&golden_path, &canonical).unwrap_or_else(|err| {
                panic!("failed to write golden {}: {err}", golden_path.display())
            });
            continue;
        }

        let expected = fs::read_to_string(&golden_path).unwrap_or_else(|err| {
            panic!(
                "failed to read golden {} (set UPDATE_GOLDENS=1 to regenerate): {err}",
                golden_path.display()
            )
        });
        assert_eq!(
            canonical,
            expected,
            "golden mismatch for fixture {}",
            fixture_path.display()
        );
    }
}
