use super::*;
use crate::{
    api::native_write::{
        commit_prepared,
        genesis::{GenesisSpec, bootstrap_native_project},
    },
    import::kicad_footprint_library,
    substrate::{CommitSource, ProjectResolver},
};

#[test]
#[ignore = "real installed KiCad footprint; run explicitly for canonical acquisition proof"]
fn real_footprint_acquisition_is_atomic_reversible_and_source_qualified() {
    let root = super::super::test_support::temp_project_root("footprint-acquisition");
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Native footprint acquisition".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let source = std::path::Path::new(
        "/usr/share/kicad/footprints/Resistor_SMD.pretty/R_0603_1608Metric.kicad_mod",
    );
    let original = std::fs::read(source).unwrap();
    let converted = kicad_footprint_library::convert(source).unwrap();
    let request = LibraryAcquisitionRequest {
        source_format: "kicad_footprint".into(),
        source_path: source.into(),
        symbol_name: converted.footprint.name.clone(),
        expected_source_hash: converted.source_hash.clone(),
    };
    let provenance = || {
        WriteProvenance::new(
            "datum-gui",
            CommitSource::Manual,
            "Acquire selected footprint",
        )
    };
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let before_objects = model.objects.clone();
    let prepare = |model: &crate::substrate::DesignModel, request| {
        build_acquisition(
            &NativeWriteContext {
                model,
                project_root: &root,
            },
            provenance(),
            request,
        )
    };
    drop(prepare(&model, request.clone()).unwrap());
    assert_eq!(
        ProjectResolver::new(&root).resolve().unwrap().objects,
        before_objects
    );
    let mut stale = request.clone();
    stale.expected_source_hash = "stale".into();
    assert!(prepare(&model, stale).is_err());
    let prepared = prepare(&model, request.clone()).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    for id in [converted.package.uuid, converted.footprint.uuid]
        .into_iter()
        .chain(converted.padstacks.iter().map(|s| s.uuid))
    {
        assert!(reopened.objects.contains_key(&id));
        assert!(
            reopened
                .import_map
                .values()
                .any(|e| e.object_id == id && e.source_hash == converted.source_hash)
        );
    }
    let raw = reopened
        .materialized_source_shard_value_by_relative_path(&format!(
            "pool/footprints/{}.json",
            converted.footprint.uuid
        ))
        .unwrap();
    assert_eq!(
        raw["original_kicad_source"].as_str(),
        Some(converted.source_document.as_str())
    );
    assert_eq!(
        serde_json::from_value::<crate::pool::Footprint>(raw).unwrap(),
        converted.footprint
    );
    assert!(prepare(&reopened, request).is_err());
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(
        ProjectResolver::new(&root).resolve().unwrap().objects,
        before_objects
    );
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .objects
            .contains_key(&converted.footprint.uuid)
    );
    assert_eq!(std::fs::read(source).unwrap(), original);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn rounded_padstack_edits_preserve_raw_extensions_and_refuse_loss() {
    use crate::api::native_write::{
        library::{PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write},
        library_gui::{LibraryDefinitionRequest, build_definition},
    };
    use serde_json::{Value, json};
    let root = super::super::test_support::temp_project_root("rounded-padstack-edit");
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Rounded geometry edits".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let provenance = || {
        WriteProvenance::new(
            "datum-gui",
            CommitSource::Manual,
            "Edit saved rounded padstack",
        )
    };
    let id = uuid::Uuid::new_v4();
    let object = json!({"schema_version":1,"uuid":id,"name":"Rounded","vendor":{"keep":true},
      "aperture":{"round_rect":{"width_nm":800000,"height_nm":950000,"rratio_ppm":250000,"raw_extension":7}}});
    let created = build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", "padstacks", id),
            object,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, created).unwrap();
    let edit = |model: &crate::substrate::DesignModel, aperture: Value| {
        build_definition(
            &NativeWriteContext {
                model,
                project_root: &root,
            },
            provenance(),
            LibraryDefinitionRequest {
                kind: "padstacks".into(),
                object_id: id,
                pool_path: "pool".into(),
                fields: json!({"aperture":aperture}).as_object().unwrap().clone(),
            },
            false,
        )
    };
    for shape in [
        json!({"rect":{"width_nm":800000,"height_nm":950000}}),
        json!({"circle":{"diameter_nm":800000}}),
    ] {
        assert!(
            edit(&model, shape).is_err(),
            "shape replacement must not discard raw extension"
        );
    }
    let same = edit(
        &model,
        json!({"round_rect":{"width_nm":850000,"height_nm":950000,"rratio_ppm":250000}}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, same).unwrap();
    let path = format!("pool/padstacks/{id}.json");
    let raw = ProjectResolver::new(&root)
        .resolve()
        .unwrap()
        .materialized_source_shard_value_by_relative_path(&path)
        .unwrap();
    assert_eq!(raw["aperture"]["round_rect"]["raw_extension"], 7);
    assert_eq!(raw["aperture"]["round_rect"]["width_nm"], 850000);
    assert_eq!(raw["vendor"]["keep"], true);
    // A clean typed source can change shape without retaining two enum keys.
    let clean_id = uuid::Uuid::new_v4();
    let clean = json!({"schema_version":1,"uuid":clean_id,"name":"Clean rounded","aperture":{"round_rect":{"width_nm":800000,"height_nm":950000,"rratio_ppm":250000}}});
    let created = build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", "padstacks", clean_id),
            object: clean,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, created).unwrap();
    for aperture in [
        json!({"rect":{"width_nm":800000,"height_nm":950000}}),
        json!({"circle":{"diameter_nm":800000}}),
    ] {
        let prepared = build_definition(
            &NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            provenance(),
            LibraryDefinitionRequest {
                kind: "padstacks".into(),
                object_id: clean_id,
                pool_path: "pool".into(),
                fields: json!({"aperture":aperture.clone()})
                    .as_object()
                    .unwrap()
                    .clone(),
            },
            false,
        )
        .unwrap();
        commit_prepared(&mut model, &root, prepared).unwrap();
        let saved = model
            .materialized_source_shard_value_by_relative_path(&format!(
                "pool/padstacks/{clean_id}.json"
            ))
            .unwrap();
        assert_eq!(saved["aperture"], aperture);
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
    }
    std::fs::remove_dir_all(root).unwrap();
}
