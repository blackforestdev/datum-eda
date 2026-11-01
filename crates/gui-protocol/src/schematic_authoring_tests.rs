//! Picker eligibility uses real canonical library objects, without a physical Part.
use super::*;
use eda_engine::api::native_write::{
    commit_prepared,
    context::WriteProvenance,
    genesis::{GenesisSpec, bootstrap_native_project},
    library::{PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write},
};
use eda_engine::substrate::{CommitSource, ProjectResolver};

#[test]
fn optional_binding_picker_selects_valid_unit_backed_symbol_and_retains_diagnostics() {
    let root =
        std::env::temp_dir().join(format!("datum-picker-optional-binding-{}", Uuid::new_v4()));
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Picker binding test".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let baseline = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test-harness/testdata/library/native_authored_baseline_v1/pool");
    let mut objects = Vec::new();
    for kind in ["units", "symbols"] {
        for entry in std::fs::read_dir(baseline.join(kind)).unwrap() {
            let object: serde_json::Value =
                serde_json::from_slice(&std::fs::read(entry.unwrap().path()).unwrap()).unwrap();
            let id = object["uuid"].as_str().unwrap().parse().unwrap();
            objects.push(PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", kind, id),
                object,
            });
        }
    }
    let prepared = build_pool_library_write(
        &model,
        WriteProvenance::new("test", CommitSource::Test, "picker eligibility"),
        Some("pool"),
        objects,
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let before = model.model_revision.clone();
    let mut picker = SchematicAuthoring::from_model(&model).unwrap();
    assert_eq!(picker.choices.len(), 1);
    assert_eq!(picker.choices[0].pins.len(), 5);
    assert_eq!(picker.choices[0].binding_status, "unresolved_entity_gate");
    assert!(!picker.choices[0].binding_diagnostics.is_empty());
    assert!(
        picker.choices[0]
            .binding_label()
            .contains("No Part binding")
    );
    assert!(picker.select(0));
    assert_eq!(picker.chosen, Some(0));
    assert!(picker.refusal.is_none());
    let reference = picker.reference.clone();
    let default_value = picker.value.clone();
    picker.value_editor.select_all(&picker.value);
    picker
        .value_editor
        .replace(&mut picker.value, "10k")
        .unwrap();
    assert_eq!(
        picker.value, "10k",
        "typing replaces the selected default value"
    );
    assert_eq!(
        picker.reference, reference,
        "value editing leaves reference intact"
    );
    assert!(picker.select(0));
    assert_eq!(picker.value, default_value);
    assert_eq!(picker.value_editor.caret, picker.value.len());
    assert!(picker.value_editor.selection(&picker.value).is_empty());
    assert_eq!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .model_revision,
        before,
        "picker selection must not author bindings"
    );
    drop(model);
    std::fs::remove_dir_all(root).unwrap();
}
