use super::*;
use std::collections::BTreeMap;

fn write_import_map_footprint_board(
    root: &std::path::Path,
    name: &str,
    footprint_uuid: &str,
    pad_uuid: Option<&str>,
) -> std::path::PathBuf {
    let path = root.join(name);
    let pad_uuid_line = pad_uuid
        .map(|uuid| format!(" (uuid {uuid})"))
        .unwrap_or_default();
    std::fs::write(
        &path,
        format!(
            r#"(kicad_pcb
  (version 20221018)
  (layers
    (0 "F.Cu" signal)
    (31 "B.Cu" signal))
  (net 0 "")
  (net 1 "SIG")
  (footprint "Demo:Mapped"
    (layer "F.Cu")
    (uuid {footprint_uuid})
    (at 0 0)
    (property "Reference" "U1" (at 0 0 0))
    (property "Value" "Mapped" (at 0 0 0))
    (pad "1" smd rect (at 0 0) (size 1 1) (layers "F.Cu" "F.Paste" "F.Mask") (net 1 "SIG"){pad_uuid_line})))"#
        ),
    )
    .expect("board should write");
    path
}

fn import_map_entry(
    import_key: String,
    object_id: uuid::Uuid,
    path: &std::path::Path,
    source_uuid: uuid::Uuid,
) -> crate::substrate::ImportMapEntry {
    crate::substrate::ImportMapEntry {
        import_key,
        object_id,
        source_shard_id: uuid::Uuid::new_v4(),
        status: crate::substrate::ImportMapEntryStatus::Active,
        source_tool: "kicad".to_string(),
        source_path: path.display().to_string(),
        source_object_ref: source_uuid.to_string(),
        source_hash: "sha256:test".to_string(),
    }
}

#[test]
fn board_import_reuses_import_map_segment_identity() {
    let root = std::env::temp_dir().join(format!(
        "datum-kicad-board-import-map-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).expect("temp root should exist");
    let path = root.join("mapped-segment.kicad_pcb");
    let source_uuid = uuid::Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
    std::fs::write(
        &path,
        r#"(kicad_pcb
  (version 20221018)
  (layers
    (0 "F.Cu" signal)
    (31 "B.Cu" signal))
  (net 0 "")
  (net 1 "SIG")
  (segment
    (start 1 1)
    (end 5 1)
    (width 0.25)
    (layer "F.Cu")
    (net 1)
    (uuid 11111111-1111-1111-1111-111111111111)))"#,
    )
    .expect("board should write");

    let import_key = board_segment_import_key(&path, source_uuid);
    let mapped_uuid = uuid::Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();
    let mut import_map = BTreeMap::new();
    import_map.insert(
        import_key.clone(),
        import_map_entry(import_key, mapped_uuid, &path, source_uuid),
    );

    let (board, _report) =
        import_board_document_with_import_map(&path, &import_map).expect("board should import");

    assert!(board.tracks.contains_key(&mapped_uuid));
    assert_eq!(board.tracks[&mapped_uuid].uuid, mapped_uuid);
    assert!(!board.tracks.contains_key(&source_uuid));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn board_import_map_mode_allocates_deterministic_segment_identity() {
    let root = std::env::temp_dir().join(format!(
        "datum-kicad-board-import-map-new-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).expect("temp root should exist");
    let path = root.join("new-segment.kicad_pcb");
    let source_uuid = uuid::Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap();
    std::fs::write(
        &path,
        r#"(kicad_pcb
  (version 20221018)
  (layers
    (0 "F.Cu" signal)
    (31 "B.Cu" signal))
  (net 0 "")
  (net 1 "SIG")
  (segment
    (start 1 1)
    (end 5 1)
    (width 0.25)
    (layer "F.Cu")
    (net 1)
    (uuid 33333333-3333-3333-3333-333333333333)))"#,
    )
    .expect("board should write");

    let empty_map = BTreeMap::new();
    let (first, _first_report) =
        import_board_document_with_import_map(&path, &empty_map).expect("first import");
    let (second, _second_report) =
        import_board_document_with_import_map(&path, &empty_map).expect("second import");
    let legacy = import_board_document(&path)
        .expect("legacy import")
        .0
        .tracks
        .keys()
        .next()
        .copied()
        .expect("legacy track");
    let first_track = first.tracks.keys().next().copied().expect("first track");
    let second_track = second.tracks.keys().next().copied().expect("second track");

    assert_eq!(first_track, second_track);
    assert_ne!(first_track, source_uuid);
    assert_eq!(legacy, source_uuid);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn board_import_reuses_import_map_via_identity() {
    let root = std::env::temp_dir().join(format!(
        "datum-kicad-board-import-map-via-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).expect("temp root should exist");
    let path = root.join("mapped-via.kicad_pcb");
    let source_uuid = uuid::Uuid::parse_str("44444444-4444-4444-4444-444444444444").unwrap();
    std::fs::write(
        &path,
        r#"(kicad_pcb
  (version 20221018)
  (layers
    (0 "F.Cu" signal)
    (31 "B.Cu" signal))
  (net 0 "")
  (net 1 "SIG")
  (via
    (at 2 2)
    (size 0.8)
    (drill 0.4)
    (layers "F.Cu" "B.Cu")
    (net 1)
    (uuid 44444444-4444-4444-4444-444444444444)))"#,
    )
    .expect("board should write");

    let import_key = board_via_import_key(&path, source_uuid);
    let mapped_uuid = uuid::Uuid::parse_str("55555555-5555-5555-5555-555555555555").unwrap();
    let mut import_map = BTreeMap::new();
    import_map.insert(
        import_key.clone(),
        import_map_entry(import_key, mapped_uuid, &path, source_uuid),
    );

    let (board, _report) =
        import_board_document_with_import_map(&path, &import_map).expect("board should import");

    assert!(board.vias.contains_key(&mapped_uuid));
    assert_eq!(board.vias[&mapped_uuid].uuid, mapped_uuid);
    assert!(!board.vias.contains_key(&source_uuid));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn board_import_map_mode_allocates_deterministic_via_identity() {
    let root = std::env::temp_dir().join(format!(
        "datum-kicad-board-import-map-new-via-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).expect("temp root should exist");
    let path = root.join("new-via.kicad_pcb");
    let source_uuid = uuid::Uuid::parse_str("66666666-6666-6666-6666-666666666666").unwrap();
    std::fs::write(
        &path,
        r#"(kicad_pcb
  (version 20221018)
  (layers
    (0 "F.Cu" signal)
    (31 "B.Cu" signal))
  (net 0 "")
  (net 1 "SIG")
  (via
    (at 2 2)
    (size 0.8)
    (drill 0.4)
    (layers "F.Cu" "B.Cu")
    (net 1)
    (uuid 66666666-6666-6666-6666-666666666666)))"#,
    )
    .expect("board should write");

    let empty_map = BTreeMap::new();
    let (first, _first_report) =
        import_board_document_with_import_map(&path, &empty_map).expect("first import");
    let (second, _second_report) =
        import_board_document_with_import_map(&path, &empty_map).expect("second import");
    let legacy = import_board_document(&path)
        .expect("legacy import")
        .0
        .vias
        .keys()
        .next()
        .copied()
        .expect("legacy via");
    let first_via = first.vias.keys().next().copied().expect("first via");
    let second_via = second.vias.keys().next().copied().expect("second via");

    assert_eq!(first_via, second_via);
    assert_ne!(first_via, source_uuid);
    assert_eq!(legacy, source_uuid);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn board_import_reuses_import_map_zone_identity() {
    let root = std::env::temp_dir().join(format!(
        "datum-kicad-board-import-map-zone-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).expect("temp root should exist");
    let path = root.join("mapped-zone.kicad_pcb");
    let source_uuid = uuid::Uuid::parse_str("77777777-7777-7777-7777-777777777777").unwrap();
    std::fs::write(
        &path,
        r#"(kicad_pcb
  (version 20221018)
  (layers
    (0 "F.Cu" signal)
    (31 "B.Cu" signal))
  (net 0 "")
  (net 1 "SIG")
  (zone
    (net 1)
    (net_name "SIG")
    (layer "F.Cu")
    (uuid 77777777-7777-7777-7777-777777777777)
    (polygon
      (pts
        (xy 0 0)
        (xy 3 0)
        (xy 3 3)
        (xy 0 3)))))"#,
    )
    .expect("board should write");

    let import_key = board_zone_import_key(&path, source_uuid);
    let mapped_uuid = uuid::Uuid::parse_str("88888888-8888-8888-8888-888888888888").unwrap();
    let mut import_map = BTreeMap::new();
    import_map.insert(
        import_key.clone(),
        import_map_entry(import_key, mapped_uuid, &path, source_uuid),
    );

    let (board, _report) =
        import_board_document_with_import_map(&path, &import_map).expect("board should import");

    assert!(board.zones.contains_key(&mapped_uuid));
    assert_eq!(board.zones[&mapped_uuid].uuid, mapped_uuid);
    assert!(!board.zones.contains_key(&source_uuid));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn board_import_map_mode_allocates_deterministic_zone_identity() {
    let root = std::env::temp_dir().join(format!(
        "datum-kicad-board-import-map-new-zone-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).expect("temp root should exist");
    let path = root.join("new-zone.kicad_pcb");
    let source_uuid = uuid::Uuid::parse_str("99999999-9999-9999-9999-999999999999").unwrap();
    std::fs::write(
        &path,
        r#"(kicad_pcb
  (version 20221018)
  (layers
    (0 "F.Cu" signal)
    (31 "B.Cu" signal))
  (net 0 "")
  (net 1 "SIG")
  (zone
    (net 1)
    (net_name "SIG")
    (layer "F.Cu")
    (uuid 99999999-9999-9999-9999-999999999999)
    (polygon
      (pts
        (xy 0 0)
        (xy 3 0)
        (xy 3 3)
        (xy 0 3)))))"#,
    )
    .expect("board should write");

    let empty_map = BTreeMap::new();
    let (first, _first_report) =
        import_board_document_with_import_map(&path, &empty_map).expect("first import");
    let (second, _second_report) =
        import_board_document_with_import_map(&path, &empty_map).expect("second import");
    let legacy = import_board_document(&path)
        .expect("legacy import")
        .0
        .zones
        .keys()
        .next()
        .copied()
        .expect("legacy zone");
    let first_zone = first.zones.keys().next().copied().expect("first zone");
    let second_zone = second.zones.keys().next().copied().expect("second zone");

    assert_eq!(first_zone, second_zone);
    assert_ne!(first_zone, source_uuid);
    assert_eq!(legacy, source_uuid);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn board_import_reuses_import_map_footprint_identity() {
    let root = std::env::temp_dir().join(format!(
        "datum-kicad-board-import-map-footprint-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).expect("temp root should exist");
    let source_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
    let path = write_import_map_footprint_board(
        &root,
        "mapped-footprint.kicad_pcb",
        &source_uuid.to_string(),
        Some("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"),
    );
    let mapped_uuid = uuid::Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").unwrap();
    let import_key = board_footprint_import_key(&path, source_uuid);
    let mut import_map = BTreeMap::new();
    import_map.insert(
        import_key.clone(),
        import_map_entry(import_key, mapped_uuid, &path, source_uuid),
    );

    let (board, _report) =
        import_board_document_with_import_map(&path, &import_map).expect("board should import");

    assert!(board.packages.contains_key(&mapped_uuid));
    assert_eq!(board.packages[&mapped_uuid].uuid, mapped_uuid);
    assert!(!board.packages.contains_key(&source_uuid));
    assert!(board.pads.values().all(|pad| pad.package == mapped_uuid));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn board_import_map_mode_allocates_deterministic_footprint_identity() {
    let root = std::env::temp_dir().join(format!(
        "datum-kicad-board-import-map-new-footprint-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).expect("temp root should exist");
    let source_uuid = uuid::Uuid::parse_str("dddddddd-dddd-dddd-dddd-dddddddddddd").unwrap();
    let path = write_import_map_footprint_board(
        &root,
        "new-footprint.kicad_pcb",
        &source_uuid.to_string(),
        Some("eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee"),
    );
    let empty_map = BTreeMap::new();

    let (first, _first_report) =
        import_board_document_with_import_map(&path, &empty_map).expect("first import");
    let (second, _second_report) =
        import_board_document_with_import_map(&path, &empty_map).expect("second import");
    let legacy = import_board_document(&path)
        .expect("legacy import")
        .0
        .packages
        .keys()
        .next()
        .copied()
        .expect("legacy footprint");
    let first_package = first
        .packages
        .keys()
        .next()
        .copied()
        .expect("first footprint");
    let second_package = second
        .packages
        .keys()
        .next()
        .copied()
        .expect("second footprint");

    assert_eq!(first_package, second_package);
    assert_ne!(first_package, source_uuid);
    assert_eq!(legacy, source_uuid);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn board_import_reuses_import_map_pad_identity() {
    let root = std::env::temp_dir().join(format!(
        "datum-kicad-board-import-map-pad-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).expect("temp root should exist");
    let footprint_uuid = uuid::Uuid::parse_str("11111111-aaaa-aaaa-aaaa-111111111111").unwrap();
    let pad_source_uuid = uuid::Uuid::parse_str("22222222-bbbb-bbbb-bbbb-222222222222").unwrap();
    let path = write_import_map_footprint_board(
        &root,
        "mapped-pad.kicad_pcb",
        &footprint_uuid.to_string(),
        Some(&pad_source_uuid.to_string()),
    );
    let mapped_uuid = uuid::Uuid::parse_str("33333333-cccc-cccc-cccc-333333333333").unwrap();
    let import_key = board_pad_import_key(&path, pad_source_uuid);
    let mut import_map = BTreeMap::new();
    import_map.insert(
        import_key.clone(),
        import_map_entry(import_key, mapped_uuid, &path, pad_source_uuid),
    );

    let (board, _report) =
        import_board_document_with_import_map(&path, &import_map).expect("board should import");

    assert!(board.pads.contains_key(&mapped_uuid));
    assert_eq!(board.pads[&mapped_uuid].uuid, mapped_uuid);
    assert!(!board.pads.contains_key(&pad_source_uuid));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn board_import_map_mode_allocates_deterministic_pad_identity() {
    let root = std::env::temp_dir().join(format!(
        "datum-kicad-board-import-map-new-pad-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).expect("temp root should exist");
    let footprint_uuid = uuid::Uuid::parse_str("44444444-dddd-dddd-dddd-444444444444").unwrap();
    let pad_source_uuid = uuid::Uuid::parse_str("55555555-eeee-eeee-eeee-555555555555").unwrap();
    let path = write_import_map_footprint_board(
        &root,
        "new-pad.kicad_pcb",
        &footprint_uuid.to_string(),
        Some(&pad_source_uuid.to_string()),
    );
    let empty_map = BTreeMap::new();

    let (first, _first_report) =
        import_board_document_with_import_map(&path, &empty_map).expect("first import");
    let (second, _second_report) =
        import_board_document_with_import_map(&path, &empty_map).expect("second import");
    let legacy = import_board_document(&path)
        .expect("legacy import")
        .0
        .pads
        .keys()
        .next()
        .copied()
        .expect("legacy pad");
    let first_pad = first.pads.keys().next().copied().expect("first pad");
    let second_pad = second.pads.keys().next().copied().expect("second pad");

    assert_eq!(first_pad, second_pad);
    assert_ne!(first_pad, pad_source_uuid);
    assert_eq!(legacy, pad_source_uuid);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn board_import_map_pad_fallback_identity_is_independent_of_mapped_package_identity() {
    let root = std::env::temp_dir().join(format!(
        "datum-kicad-board-import-map-pad-fallback-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).expect("temp root should exist");
    let footprint_uuid = uuid::Uuid::parse_str("66666666-ffff-ffff-ffff-666666666666").unwrap();
    let path = write_import_map_footprint_board(
        &root,
        "fallback-pad.kicad_pcb",
        &footprint_uuid.to_string(),
        None,
    );
    let package_map_key = board_footprint_import_key(&path, footprint_uuid);
    let first_package_uuid = uuid::Uuid::parse_str("77777777-aaaa-bbbb-cccc-777777777777").unwrap();
    let second_package_uuid =
        uuid::Uuid::parse_str("88888888-aaaa-bbbb-cccc-888888888888").unwrap();
    let mut first_map = BTreeMap::new();
    first_map.insert(
        package_map_key.clone(),
        import_map_entry(
            package_map_key.clone(),
            first_package_uuid,
            &path,
            footprint_uuid,
        ),
    );
    let mut second_map = BTreeMap::new();
    second_map.insert(
        package_map_key.clone(),
        import_map_entry(package_map_key, second_package_uuid, &path, footprint_uuid),
    );

    let (first, _first_report) =
        import_board_document_with_import_map(&path, &first_map).expect("first import");
    let (second, _second_report) =
        import_board_document_with_import_map(&path, &second_map).expect("second import");
    let first_pad = first.pads.keys().next().copied().expect("first pad");
    let second_pad = second.pads.keys().next().copied().expect("second pad");

    assert_ne!(first_package_uuid, second_package_uuid);
    assert_eq!(first_pad, second_pad);
    assert!(
        first
            .pads
            .values()
            .all(|pad| pad.package == first_package_uuid)
    );
    assert!(
        second
            .pads
            .values()
            .all(|pad| pad.package == second_package_uuid)
    );

    let _ = std::fs::remove_dir_all(root);
}
