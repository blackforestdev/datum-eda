use super::*;
use crate::api::native_write::{
    commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
};
use crate::pool::process::{ConcreteProcessBasis, ProcessFace};
use crate::pool::{PadstackMaskPolicy, PadstackPastePolicy};
use crate::substrate::{CommitSource, ProjectResolver};
use serde_json::json;

fn provenance() -> WriteProvenance {
    WriteProvenance::new("designer", CommitSource::Test, "process lifecycle")
}

fn project(label: &str) -> (std::path::PathBuf, DesignModel) {
    let root = crate::api::native_write::test_support::temp_project_root(label);
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Process lifecycle".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    (root, model)
}

#[test]
fn process_profile_guard_reopen_history_and_absent_collection() {
    let (root, mut model) = project("process_profile_history");
    assert!(profiles(&model).unwrap().is_none());
    let id = Uuid::new_v4();
    let mut profile = ProcessProfile {
        uuid: id,
        name: "Assembly".into(),
        basis: ConcreteProcessBasis {
            mask: PadstackMaskPolicy::ExpansionNm(10),
            paste: PadstackPastePolicy::ExpansionNm(-2),
            paste_ratio_ppm: 100_000,
            declared_basis: "authored".into(),
            provenance: json!({"actor": "designer"}),
        },
    };
    let prepared = build_set_process_profile(&model, provenance(), profile.clone()).unwrap();
    let stale = prepared.clone();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    assert!(model.diagnostics.is_empty(), "{:?}", model.diagnostics);
    assert_eq!(model.journal.len(), 1);
    assert_eq!(profiles(&model).unwrap().unwrap()[&id], profile);
    assert!(pinned_profile_selector(&model, id).is_ok());
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    profile.basis.paste_ratio_ppm = 200_000;
    let prepared = build_set_process_profile(&model, provenance(), profile.clone()).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    assert!(model.diagnostics.is_empty(), "{:?}", model.diagnostics);
    assert_eq!(profiles(&model).unwrap().unwrap()[&id], profile);
    assert_eq!(model.objects[&id].object_revision.0, 1);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        profiles(&model).unwrap().unwrap()[&id]
            .basis
            .paste_ratio_ppm,
        100_000
    );
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(profiles(&model).unwrap().unwrap()[&id], profile);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn process_capability_forward_only_guarded_adoption_keeps_profile_four() {
    let (root, mut model) = project("process_capability_history");
    let board_id = serde_json::from_value(
        model
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .unwrap()["uuid"]
            .clone(),
    )
    .unwrap();
    let prepared =
        crate::api::native_write::board_layout::source_profile::build_adopt_board_source_profile(
            &model,
            provenance(),
            board_id,
        )
        .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let stackup: crate::board::Stackup = serde_json::from_value(board["stackup"].clone()).unwrap();
    let copper: Vec<_> = stackup
        .layers
        .iter()
        .filter(|l| l.layer_type == crate::board::StackupLayerType::Copper)
        .map(|l| l.id)
        .collect();
    let capability = ProcessSourceCapability {
        version: 1,
        front: ProcessFace {
            copper_layer: *copper.first().unwrap(),
            mask_layer: None,
            paste_layer: None,
        },
        back: ProcessFace {
            copper_layer: *copper.last().unwrap(),
            mask_layer: None,
            paste_layer: None,
        },
    };
    let prepared =
        build_adopt_process_capability(&model, &root, provenance(), capability.clone()).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    assert!(model.diagnostics.is_empty(), "{:?}", model.diagnostics);
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    assert_eq!(board["schema_version"], 4);
    assert_eq!(capability_from_board(&board).unwrap(), Some(capability));
    let before = std::fs::read(root.join("board/board.json")).unwrap();
    let count = model.journal.len();
    assert!(
        model
            .commit_journal_undo(&root, provenance().into())
            .is_err()
    );
    assert_eq!(
        std::fs::read(root.join("board/board.json")).unwrap(),
        before
    );
    assert_eq!(
        ProjectResolver::new(&root).resolve().unwrap().journal.len(),
        count
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn process_inspection_rejects_wrong_library_family_without_mutation() {
    use crate::api::native_write::library::{
        PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write,
    };
    let (root, mut model) = project("process_inspection_family");
    let id = Uuid::new_v4();
    let prepared = build_pool_library_write(&model, provenance(), Some("pool"), vec![PoolLibraryOperationSpec::Create {
        target: PoolLibraryObjectTarget::new("pool", "padstacks", id),
        object: json!({"schema_version": 1, "uuid": id, "name": "explicit suppressed process", "aperture": {"circle": {"diameter_nm": 101}},
            "mask_policy": "tented", "paste_policy": "none"}),
    }]).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let before = std::fs::read(root.join("pool/padstacks").join(format!("{id}.json"))).unwrap();
    let error = inspect_effective_process(&model, id, id)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("footprints") && error.contains(&id.to_string()),
        "{error}"
    );
    assert_eq!(
        std::fs::read(root.join("pool/padstacks").join(format!("{id}.json"))).unwrap(),
        before
    );
    assert_eq!(
        ProjectResolver::new(&root).resolve().unwrap().journal.len(),
        model.journal.len()
    );
    let mut model = model;
    let package_id = Uuid::new_v4();
    let footprint_id = Uuid::new_v4();
    let prepared = build_pool_library_write(&model, provenance(), Some("pool"), vec![
        PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", "packages", package_id),
            object: json!({"schema_version": 1, "uuid": package_id, "name": "authored body", "pads": {}}),
        },
        PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", "footprints", footprint_id),
            object: json!({"schema_version": 1, "uuid": footprint_id, "name": "authored footprint", "package": package_id,
                "pads": {}, "courtyard": {"vertices": [], "closed": false}}),
        },
    ]).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    assert!(inspect_effective_process(&model, footprint_id, id).is_ok());
    let mut without_taxon = model.clone();
    for shard in &mut without_taxon.source_shards {
        if shard.kind == SourceShardKind::Pool {
            shard.taxon = None;
        }
    }
    assert!(inspect_effective_process(&without_taxon, footprint_id, id).is_ok());
    assert!(
        inspect_effective_process(&model, footprint_id, footprint_id)
            .unwrap_err()
            .to_string()
            .contains("padstacks")
    );
    let selector = ProcessBasisSelector::Inline {
        basis: ConcreteProcessBasis {
            mask: PadstackMaskPolicy::Exposed,
            paste: PadstackPastePolicy::Aperture,
            paste_ratio_ppm: 0,
            declared_basis: "authored".into(),
            provenance: json!({"actor": "designer"}),
        },
    };
    let prepared =
        build_select_process_basis(&model, provenance(), footprint_id, selector.clone()).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        inspect_effective_process(&model, footprint_id, id)
            .unwrap()
            .selected_basis,
        Some(selector)
    );
    let _ = std::fs::remove_dir_all(root);
}
