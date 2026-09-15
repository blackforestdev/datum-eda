use std::collections::HashSet;

use super::*;
use crate::ir::ids::namespace_eagle;

fn sample_sidecar() -> IdSidecar {
    IdSidecar::new(
        ImportFormat::Kicad,
        "board.kicad_pcb",
        "sha256:abc123",
        "2026-03-24T12:00:00Z",
        HashMap::from([
            ("net:VCC".to_string(), Uuid::from_u128(1).to_string()),
            ("net:GND".to_string(), Uuid::from_u128(2).to_string()),
        ]),
    )
}

#[test]
fn sidecar_path_appends_ids_json() {
    let path = sidecar_path_for_source("/tmp/example/board.kicad_pcb").unwrap();
    assert!(path.ends_with("board.kicad_pcb.ids.json"));
}

#[test]
fn sidecar_path_without_filename_returns_validation_error() {
    let err = sidecar_path_for_source("/").expect_err("root path should fail");
    assert!(matches!(err, EngineError::Validation(_)));
}

#[test]
fn source_hash_is_prefixed_sha256() {
    let hash = compute_source_hash_bytes(b"hello");
    assert!(hash.starts_with("sha256:"));
}

#[test]
fn write_read_sidecar_round_trip() {
    let dir = std::env::temp_dir().join(format!("eda-sidecar-{}", std::process::id()));
    let path = dir.join("example.ids.json");
    fs::create_dir_all(&dir).unwrap();

    let sidecar = sample_sidecar();
    write_sidecar(&path, &sidecar).unwrap();
    let decoded = read_sidecar(&path).unwrap();
    assert_eq!(sidecar, decoded);

    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn restore_exact_when_hash_matches() {
    let sidecar = sample_sidecar();
    let restored = restore_or_merge_mappings(
        vec!["net:VCC".to_string(), "net:GND".to_string()],
        Some(&sidecar),
        "sha256:abc123",
        &namespace_eagle(),
    );
    assert_eq!(restored, sidecar.mappings);
}

#[test]
fn merge_preserves_existing_and_adds_new_paths() {
    let sidecar = sample_sidecar();
    let merged = restore_or_merge_mappings(
        vec![
            "net:VCC".to_string(),
            "net:GND".to_string(),
            "net:SIG".to_string(),
        ],
        Some(&sidecar),
        "sha256:different",
        &namespace_eagle(),
    );

    assert_eq!(
        merged.get("net:VCC").unwrap(),
        &Uuid::from_u128(1).to_string()
    );
    assert_eq!(
        merged.get("net:GND").unwrap(),
        &Uuid::from_u128(2).to_string()
    );
    assert!(Uuid::parse_str(merged.get("net:SIG").unwrap()).is_ok());
    assert_eq!(
        merged.keys().cloned().collect::<HashSet<_>>(),
        HashSet::from([
            "net:VCC".to_string(),
            "net:GND".to_string(),
            "net:SIG".to_string()
        ])
    );
}
