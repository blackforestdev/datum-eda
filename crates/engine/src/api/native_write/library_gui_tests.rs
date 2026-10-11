//! Guarded definition edits against an ordinary canonical native project.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
};
use crate::substrate::{CommitSource, ProjectResolver};

fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "native-library-test",
        CommitSource::Manual,
        "Edit library definition",
    )
}
fn request(id: Uuid, fields: Value) -> LibraryDefinitionRequest {
    LibraryDefinitionRequest {
        kind: "padstacks".into(),
        object_id: id,
        pool_path: "pool".into(),
        fields: fields.as_object().unwrap().clone(),
    }
}

#[test]
fn library_definition_edit_preserves_raw_extensions_and_omitted_policy() {
    let root = super::super::test_support::temp_project_root("library_gui_raw");
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Library workflow".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let id = Uuid::new_v4();
    let target = PoolLibraryObjectTarget::new("pool", "padstacks", id);
    let initial = json!({"schema_version":1,"uuid":id,"name":"Original","aperture":{"circle":{"diameter_nm":1000000}},"vendor_extension":{"retained":[1,2]}});
    let prepared = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Create {
            target: target.clone(),
            object: initial.clone(),
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let prepared = build_definition(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        request(
            id,
            json!({"name":"Edited","aperture":{"rect":{"width_nm":2000000,"height_nm":1000000}}}),
        ),
        false,
    )
    .unwrap();
    let stale = prepared.clone();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    let raw = library::pool_library_object_payload(&reopened, &target.relative_path, id).unwrap();
    assert_eq!(raw["uuid"], initial["uuid"]);
    assert_eq!(raw["vendor_extension"], initial["vendor_extension"]);
    for absent in [
        "drill_nm",
        "plated",
        "layer_span",
        "mask_policy",
        "paste_policy",
    ] {
        assert!(raw.get(absent).is_none(), "{absent} normalized");
    }
    let before = std::fs::read(root.join(&target.relative_path)).unwrap();
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    assert_eq!(
        std::fs::read(root.join(&target.relative_path)).unwrap(),
        before
    );
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
}

#[test]
fn library_definition_refuses_wrong_target_unsupported_geometry_and_collapse() {
    let root = super::super::test_support::temp_project_root("library_gui_refusal");
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Library refusal".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let context = NativeWriteContext {
        model: &model,
        project_root: &root,
    };
    let id = Uuid::new_v4();
    for fields in [
        json!({"name":"Invalid","aperture":{"rounded_rect":{"width_nm":100}}}),
        json!({"name":"Collapsed","aperture":{"circle":{"diameter_nm":100}},"mask_policy":{"expansion_nm":-50}}),
        json!({"name":"Overflow","aperture":{"circle":{"diameter_nm":i64::MAX}},"paste_policy":{"expansion_nm":1}}),
    ] {
        assert!(build_definition(&context, provenance(), request(id, fields), true).is_err());
    }
    let mut wrong = request(
        id,
        json!({"name":"Wrong","aperture":{"circle":{"diameter_nm":100}}}),
    );
    wrong.pool_path = "other-pool".into();
    assert!(
        build_definition(&context, provenance(), wrong, true)
            .unwrap_err()
            .to_string()
            .contains("project-owned")
    );
    assert!(!root.join("pool/padstacks").exists());
    assert!(!crate::substrate::transaction_journal_path(&root).exists());
}

#[test]
fn same_shape_resize_preserves_extensions_and_variant_change_refuses_loss() {
    let root = super::super::test_support::temp_project_root("library_gui_shape_extensions");
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Raw geometry".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let id = Uuid::new_v4();
    let target = PoolLibraryObjectTarget::new("pool", "padstacks", id);
    let object = json!({"schema_version":1,"uuid":id,"name":"Extended circle","aperture":{"circle":{"diameter_nm":1000000,"vendor_geometry":{"retained":true}}},"vendor_aperture":9});
    let created = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Create {
            target: target.clone(),
            object,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, created).unwrap();
    let resized = build_definition(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        request(id, json!({"aperture":{"circle":{"diameter_nm":2000000}}})),
        false,
    )
    .unwrap();
    commit_prepared(&mut model, &root, resized).unwrap();
    let raw = library::pool_library_object_payload(&model, &target.relative_path, id).unwrap();
    assert_eq!(raw["aperture"]["circle"]["diameter_nm"], json!(2000000));
    assert_eq!(
        raw["aperture"]["circle"]["vendor_geometry"],
        json!({"retained":true})
    );
    assert_eq!(raw["vendor_aperture"], json!(9));
    let source = std::fs::read(root.join(&target.relative_path)).unwrap();
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    assert!(
        build_definition(
            &NativeWriteContext {
                model: &model,
                project_root: &root
            },
            provenance(),
            request(
                id,
                json!({"aperture":{"rect":{"width_nm":2000000,"height_nm":1000000}}})
            ),
            false
        )
        .unwrap_err()
        .to_string()
        .contains("extension")
    );
    assert_eq!(
        std::fs::read(root.join(&target.relative_path)).unwrap(),
        source
    );
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
}

#[test]
fn existing_rect_refuses_malformed_aperture_edits_without_raw_merge_noops() {
    let root = super::super::test_support::temp_project_root("library_gui_rect_admission");
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Rect admission".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let id = Uuid::new_v4();
    let created=build_definition(&NativeWriteContext{model:&model,project_root:&root},provenance(),request(id,json!({"name":"Existing rect","aperture":{"rect":{"width_nm":1000000,"height_nm":500000}}})),true).unwrap();
    commit_prepared(&mut model, &root, created).unwrap();
    let path = root.join(format!("pool/padstacks/{id}.json"));
    let source = std::fs::read(&path).unwrap();
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    for aperture in [
        json!({"rounded_rect":{"width_nm":100}}),
        json!({"rect":null}),
        json!({}),
        json!({"rect":{"width_nm":100,"height_nm":100},"circle":{"diameter_nm":100}}),
    ] {
        assert!(
            build_definition(
                &NativeWriteContext {
                    model: &model,
                    project_root: &root
                },
                provenance(),
                request(id, json!({"aperture":aperture})),
                false
            )
            .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), source);
        assert_eq!(
            std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
            journal
        );
    }
}

#[test]
fn footprint_pad_collection_edit_remove_history_reopen_preserves_raw_siblings() {
    let root = super::super::test_support::temp_project_root("library_gui_collection");
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Two pad workflow".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let package = Uuid::new_v4();
    let stack = Uuid::new_v4();
    let footprint = Uuid::new_v4();
    let first = Uuid::new_v4();
    let second = Uuid::new_v4();
    let mut original =
        library_footprint::pool_footprint_payload(footprint, package, "Two pad resistor");
    original["pads"] = json!({first.to_string():{"uuid":first,"name":"1","position":{"x":-1000000,"y":0,"position_extension":7},"padstack":stack,"layer":1,"pad_extension":[1,2]},second.to_string():{"uuid":second,"name":"2","position":{"x":1000000,"y":0},"padstack":stack,"layer":1,"pad_extension":9}});
    original["vendor_extension"] = json!({"retained":true});
    let create=library::build_pool_library_write(&model,provenance(),Some("pool"),vec![
        PoolLibraryOperationSpec::Create {target:PoolLibraryObjectTarget::new("pool","packages",package),object:library::pool_package_payload(package,"Resistor",json!({}))},
        PoolLibraryOperationSpec::Create {target:PoolLibraryObjectTarget::new("pool","padstacks",stack),object:json!({"schema_version":1,"uuid":stack,"name":"Rect copper","aperture":{"rect":{"width_nm":1000000,"height_nm":500000}}})},
        PoolLibraryOperationSpec::Create {target:PoolLibraryObjectTarget::new("pool","footprints",footprint),object:original.clone()},
    ]).unwrap();
    commit_prepared(&mut model, &root, create).unwrap();
    let path = format!("pool/footprints/{footprint}.json");
    let fields = json!({"pads":{first.to_string():{"position":{"x":-2000000,"y":11}},second.to_string():null}});
    let edit = LibraryDefinitionRequest {
        kind: "footprints".into(),
        object_id: footprint,
        pool_path: "pool".into(),
        fields: fields.as_object().unwrap().clone(),
    };
    let prepared = build_definition(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        edit,
        false,
    )
    .unwrap();
    assert_eq!(
        library::pool_library_object_payload(&model, &path, footprint).unwrap(),
        original,
        "preview is read-only"
    );
    commit_prepared(&mut model, &root, prepared).unwrap();
    let edited = library::pool_library_object_payload(&model, &path, footprint).unwrap();
    assert_eq!(
        edited["pads"][first.to_string()]["position"]["x"],
        json!(-2000000)
    );
    assert_eq!(
        edited["pads"][first.to_string()]["position"]["y"],
        json!(11)
    );
    assert_eq!(edited["pads"][first.to_string()]["uuid"], json!(first));
    assert_eq!(
        edited["pads"][first.to_string()]["pad_extension"],
        json!([1, 2])
    );
    assert_eq!(
        edited["pads"][first.to_string()]["position"]["position_extension"],
        json!(7)
    );
    assert!(edited["pads"].get(second.to_string()).is_none());
    assert_eq!(edited["vendor_extension"], original["vendor_extension"]);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(
        library::pool_library_object_payload(&model, &path, footprint).unwrap(),
        original
    );
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        library::pool_library_object_payload(&reopened, &path, footprint).unwrap(),
        edited
    );
    let third = Uuid::new_v4();
    let addition = json!({"pads":{third.to_string():{"uuid":third,"name":"3","position":{"x":3000000,"y":22},"padstack":stack,"layer":1}}});
    let add = build_definition(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        LibraryDefinitionRequest {
            kind: "footprints".into(),
            object_id: footprint,
            pool_path: "pool".into(),
            fields: addition.as_object().unwrap().clone(),
        },
        false,
    )
    .unwrap();
    commit_prepared(&mut model, &root, add).unwrap();
    let added = library::pool_library_object_payload(&model, &path, footprint).unwrap();
    assert_eq!(added["pads"].as_object().unwrap().len(), 2);
    assert_eq!(
        added["pads"][first.to_string()],
        edited["pads"][first.to_string()]
    );
    assert_eq!(added["pads"][third.to_string()]["uuid"], json!(third));
    assert_eq!(
        added["pads"][third.to_string()]["position"],
        json!({"x":3000000,"y":22})
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(
        library::pool_library_object_payload(&model, &path, footprint).unwrap(),
        edited
    );
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        library::pool_library_object_payload(&reopened, &path, footprint).unwrap(),
        added
    );
    let source = std::fs::read(root.join(&path)).unwrap();
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    let missing = json!({"pads":{Uuid::new_v4().to_string():null}});
    assert!(
        build_definition(
            &NativeWriteContext {
                model: &reopened,
                project_root: &root
            },
            provenance(),
            LibraryDefinitionRequest {
                kind: "footprints".into(),
                object_id: footprint,
                pool_path: "pool".into(),
                fields: missing.as_object().unwrap().clone()
            },
            false
        )
        .is_err()
    );
    assert_eq!(std::fs::read(root.join(&path)).unwrap(), source);
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
}
