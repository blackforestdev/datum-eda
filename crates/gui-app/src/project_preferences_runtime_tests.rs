use super::*;

use eda_engine::api::native_write::genesis::{GenesisSpec, bootstrap_native_project};
use eda_engine::ir::units::{BOARD_LENGTH_KEY, DRILL_HOLE_KEY, LengthUnit, LengthUnitChoice};
use eda_engine::preferences::{
    FixedPreferenceLocationProvider, GlobalPreferencesProductService, PreferenceActorKindV1,
    PreferenceActorV1, PreferenceLocations, ProjectGenesisRequestV1, ProjectUnitsSourceV1,
};
use uuid::Uuid;

fn project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "datum-project-preferences-{label}-{}",
        std::process::id()
    ))
}

fn bootstrap(label: &str) -> PathBuf {
    let root = project_root(label);
    let _ = std::fs::remove_dir_all(&root);
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Units Surface Fixture".to_owned(),
            existing_ids: None,
        },
    )
    .unwrap();
    root
}

#[test]
fn project_surface_reads_v2_product_genesis_receipt_without_migration() {
    let root = project_root("v2-genesis");
    let config = project_root("v2-genesis-config");
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&config);
    std::fs::create_dir(&config).unwrap();
    let provider = FixedPreferenceLocationProvider(PreferenceLocations {
        configuration_base: config.clone(),
        repository_root: config.join("preferences"),
        legacy_console_path: config.join("gui-preferences.json"),
    });
    let service = GlobalPreferencesProductService::open(&provider, "gui-v2-test").unwrap();
    let actor = PreferenceActorV1 {
        kind: PreferenceActorKindV1::HumanGui,
        session_id: "gui-v2-test".to_owned(),
        local_actor_id: "test-user".to_owned(),
        invocation_id: Uuid::new_v4(),
    };
    service
        .create_project(
            ProjectGenesisRequestV1 {
                request_id: Uuid::new_v4(),
                destination: root.clone(),
                project_name: "V2 Units Project".to_owned(),
                project_id: Some(Uuid::new_v4()),
                units_source: ProjectUnitsSourceV1::Factory {
                    profile_id: "datum.units.factory.v1".to_owned(),
                },
            },
            &actor,
        )
        .unwrap();

    let mut coordinator = ProjectPreferencesCoordinator::new();
    let mut dialog = GlobalPreferencesDialogState::project_units_default();
    dialog.reduced_motion = true;
    coordinator.load_or_migrate(&root, &mut dialog).unwrap();
    assert!(dialog.reduced_motion);
    assert_eq!(dialog.rows.len(), 8);
    assert!(dialog.rows.iter().all(|row| row.writable && !row.changed));
    assert_eq!(
        ProjectResolver::new(&root).resolve().unwrap().journal.len(),
        0
    );

    let _ = std::fs::remove_dir_all(root);
    let _ = std::fs::remove_dir_all(config);
}

#[test]
fn project_surface_migrates_edits_resets_and_refuses_stale_writes() {
    let root = bootstrap("lifecycle");
    let board_before = std::fs::read(root.join("board/board.json")).unwrap();
    let schematic_before = std::fs::read(root.join("schematic/schematic.json")).unwrap();
    let mut coordinator = ProjectPreferencesCoordinator::new();
    let mut dialog = GlobalPreferencesDialogState::project_units_default();
    dialog.reduced_motion = true;

    coordinator.load_or_migrate(&root, &mut dialog).unwrap();
    assert!(dialog.reduced_motion);
    assert_eq!(dialog.rows.len(), 8);
    assert!(dialog.rows.iter().all(|row| row.section_id == "units"));
    assert!(dialog.rows.iter().all(|row| row.writable));
    let migrated = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(migrated.journal.len(), 1);
    let receipt = migrated.project.project_units_seed_receipt.clone();

    coordinator
        .set_value(
            BOARD_LENGTH_KEY,
            Value::String("mil".to_owned()),
            &mut dialog,
        )
        .unwrap();
    assert!(dialog.reduced_motion);
    let changed = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(changed.journal.len(), 2);
    assert_eq!(
        project_display_units(&changed).unwrap().board.unit,
        LengthUnitChoice::Explicit(LengthUnit::Mil)
    );
    assert_eq!(changed.project.project_units_seed_receipt, receipt);
    let board_row = dialog
        .rows
        .iter()
        .find(|row| row.key == BOARD_LENGTH_KEY)
        .unwrap();
    assert!(board_row.changed);
    assert!(board_row.reset_description.contains("recorded seed"));
    assert!(board_row.reset_description.contains("does not read Global"));

    coordinator.reset(BOARD_LENGTH_KEY, &mut dialog).unwrap();
    assert!(dialog.reduced_motion);
    let reset = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reset.journal.len(), 3);
    assert_eq!(
        project_display_units(&reset).unwrap().board.unit,
        LengthUnitChoice::FollowSystem
    );
    assert_eq!(reset.project.project_units_seed_receipt, receipt);

    let mut external = reset;
    let mut external_profile = project_display_units(&external).unwrap();
    external_profile.drill.unit = LengthUnitChoice::Explicit(LengthUnit::Inch);
    let prepared = build_set_project_display_units(
        &external,
        WriteProvenance::new("external-test", CommitSource::Test, "concurrent Units edit"),
        external_profile,
    )
    .unwrap();
    commit_prepared(&mut external, &root, prepared).unwrap();
    let stale = coordinator
        .set_value(DRILL_HOLE_KEY, Value::String("mil".to_owned()), &mut dialog)
        .unwrap_err();
    assert!(stale.to_string().contains("stale edit was not applied"));
    let after_stale = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(after_stale.journal.len(), 4);
    assert_eq!(
        project_display_units(&after_stale).unwrap().drill.unit,
        LengthUnitChoice::Explicit(LengthUnit::Inch)
    );
    // Reopening after an external commit must refresh the cached read, while
    // the stale-write refusal above still occurs before any attempted mutation.
    coordinator.load_or_migrate(&root, &mut dialog).unwrap();
    assert_eq!(
        coordinator.expected_revision.as_ref(),
        Some(&after_stale.model_revision)
    );
    let expected_dialog = dialog.clone();
    coordinator.load_or_migrate(&root, &mut dialog).unwrap();
    assert_eq!(dialog, expected_dialog);
    assert_eq!(
        std::fs::read(root.join("board/board.json")).unwrap(),
        board_before
    );
    assert_eq!(
        std::fs::read(root.join("schematic/schematic.json")).unwrap(),
        schematic_before
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn invalid_project_units_open_as_preserved_disabled_evidence() {
    let root = bootstrap("invalid");
    let manifest_path = root.join("project.json");
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    manifest["project_display_units"] = serde_json::json!({"system": "metric"});
    std::fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let before = std::fs::read(&manifest_path).unwrap();
    let mut coordinator = ProjectPreferencesCoordinator::new();
    let mut dialog = GlobalPreferencesDialogState::project_units_default();
    dialog.reduced_motion = true;

    let error = coordinator.load_or_migrate(&root, &mut dialog).unwrap_err();
    coordinator.publish_unavailable(&root, &error, &mut dialog);
    assert!(dialog.reduced_motion);
    assert_eq!(dialog.rows.len(), 8);
    assert!(dialog.rows.iter().all(|row| !row.writable));
    assert!(
        dialog
            .rows
            .iter()
            .all(|row| row.unavailable_reason.is_some())
    );
    assert!(matches!(
        dialog.notice,
        Some(GlobalPreferencesNoticeUi::PreservedUnreadable(_))
    ));
    assert_eq!(std::fs::read(&manifest_path).unwrap(), before);
    assert!(coordinator.project_root.is_none());
    let _ = std::fs::remove_dir_all(root);
}
