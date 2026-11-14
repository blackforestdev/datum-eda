//! Distinct explicit candidates retain real Symbol IDs and original picker semantics.
use super::*;
use eda_engine::api::native_write::{
    commit_prepared,
    context::WriteProvenance,
    genesis::{GenesisSpec, bootstrap_native_project},
    registry::{NativeWriteContext, find_native_write_verb},
};
use eda_engine::substrate::{CommitSource, ProjectResolver};
use serde_json::json;
fn fixture() -> (std::path::PathBuf, DesignModel, Uuid, Uuid, Uuid) {
    let root = std::env::temp_dir().join(format!("datum-explicit-choices-{}", Uuid::new_v4()));
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Named Part choices".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let (unit, symbol, entity, package, a, b, p1, p2) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    for (kind, id, fields) in [
        ("units", unit, json!({"name":"Logic","pins":{}})),
        (
            "symbols",
            symbol,
            json!({"name":"Shared body","unit":unit,"drawings":[{"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}}],"pin_anchors":[]}),
        ),
        ("packages", package, json!({"name":"Body"})),
        (
            "entities",
            entity,
            json!({"name":"Dual","manufacturer":"","prefix":"U","gates":{a.to_string():{"uuid":a,"name":"A","unit":unit,"symbol":symbol},b.to_string():{"uuid":b,"name":"B","unit":unit,"symbol":symbol}}}),
        ),
        (
            "parts",
            p1,
            json!({"manufacturer":"Maker","mpn":"FIRST","description":"","datasheet":"","value":"First value","entity":entity,"package":package}),
        ),
        (
            "parts",
            p2,
            json!({"manufacturer":"Maker","mpn":"SECOND","description":"","datasheet":"","value":"Second value","entity":entity,"package":package}),
        ),
    ] {
        let p = (find_native_write_verb("datum.library.create")
            .unwrap()
            .build)(
            &NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            WriteProvenance::new("test", CommitSource::Manual, "Seed Part choices"),
            json!({"kind":kind,"object_id":id,"pool_path":"pool","fields":fields}),
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
    }
    (root, model, symbol, p2, b)
}
#[test]
fn shared_symbol_rows_keep_four_distinct_part_gate_choices_and_symbol_only_first() {
    let (root, model, symbol, part, gate) = fixture();
    let mut state = SchematicAuthoring::from_model(&model).unwrap();
    assert_eq!(state.choices.len(), 5);
    assert_eq!(state.choices[0].id, symbol);
    assert!(state.choices[0].explicit_part.is_none());
    assert_eq!(state.choices[0].name, "Shared body");
    let keys: std::collections::BTreeSet<_> = state
        .choices
        .iter()
        .filter_map(|c| c.explicit_part.as_ref().map(|s| (s.part, s.gate)))
        .collect();
    assert_eq!(keys.len(), 4);
    assert!(state.choices.iter().all(|c| c.id == symbol));
    let index = state
        .choices
        .iter()
        .position(|c| {
            c.explicit_part
                .as_ref()
                .is_some_and(|s| s.part == part && s.gate == gate)
        })
        .unwrap();
    assert!(
        state.choices[index]
            .name
            .contains("Maker SECOND · B · Dual / Shared body")
    );
    assert!(state.select(index));
    assert_eq!(state.value, "Second value");
    assert_eq!(state.reference, "U1");
    let request = SchematicEditRequest::PlaceSymbol {
        sheet: Uuid::new_v4(),
        instance_path: vec![],
        library_symbol: state.choices[index].id,
        explicit_part: state.choices[index].explicit_part.clone(),
        component: None,
        reference: state.reference.clone(),
        value: state.value.clone(),
        x_nm: 0,
        y_nm: 0,
        rotation_deg: 0,
        mirrored: false,
    };
    let (verb, params) = request.verb_params();
    assert_eq!(verb, "datum.schematic.place_symbol");
    assert_eq!(params["library_symbol"], json!(symbol));
    assert_eq!(params["explicit_part"]["part"], json!(part));
    assert_eq!(params["explicit_part"]["gate"], json!(gate));
    assert!(state.select(0));
    assert_eq!(state.value, "Shared body");
    assert!(state.choices[state.chosen.unwrap()].explicit_part.is_none());
    let _ = std::fs::remove_dir_all(root);
}
#[test]
fn out_of_range_choice_and_cancel_preserve_sources_and_drop_selected_intent() {
    let (root, model, _, _, _) = fixture();
    let mut state = SchematicAuthoring::from_model(&model).unwrap();
    let before = state.source.clone();
    assert!(state.select(1));
    assert!(!state.select(999));
    state.cancel();
    assert!(state.chosen.is_none());
    assert!(state.pending_request.is_none());
    assert_eq!(state.source, before);
    let _ = std::fs::remove_dir_all(root);
}
