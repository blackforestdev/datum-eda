//! Real canonical library graph with multiple Parts and shared-Symbol gates.
#[path = "schematic_component_choice_tests.rs"]
mod complete_component;
#[path = "../../substrate/component_instance/part_physical_source_tests.rs"]
mod physical_history;
use super::*;
use crate::api::native_write::{
    self as native, commit_prepared,
    context::{PreparedWrite, WriteProvenance},
    registry::{NativeWriteContext, find_native_write_verb},
};
use crate::substrate::{CommitSource, ProjectResolver};
use serde_json::{Value, json};
use std::path::PathBuf;
struct F {
    root: PathBuf,
    model: DesignModel,
    part: Uuid,
    other: Uuid,
    entity: Uuid,
    unit: Uuid,
    symbol: Uuid,
    pin: Uuid,
    a: Uuid,
    b: Uuid,
    sheet: Uuid,
}
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Explicit initial Part gate")
}
fn prepare(f: &F, verb: &str, params: Value) -> Result<PreparedWrite, EngineError> {
    (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model: &f.model,
            project_root: &f.root,
        },
        provenance(),
        params,
    )
}
fn write(f: &mut F, verb: &str, params: Value) -> Uuid {
    let p = prepare(f, verb, params).unwrap();
    let id = p.primary_object_id.unwrap_or(Uuid::nil());
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    id
}
fn fixture() -> F {
    let root = native::test_support::temp_project_root("explicit_part_gate");
    native::genesis::bootstrap_native_project(
        &root,
        native::genesis::GenesisSpec {
            project_name: "Explicit placement".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut f = F {
        model: ProjectResolver::new(&root).resolve().unwrap(),
        root,
        part: Uuid::new_v4(),
        other: Uuid::new_v4(),
        entity: Uuid::new_v4(),
        unit: Uuid::new_v4(),
        symbol: Uuid::new_v4(),
        pin: Uuid::new_v4(),
        a: Uuid::new_v4(),
        b: Uuid::new_v4(),
        sheet: Uuid::new_v4(),
    };
    let (package, foot, stack, pad, pad2, map) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    for (kind, id, fields) in [
        (
            "units",
            f.unit,
            json!({"name":"Logic","pins":{f.pin.to_string():{"name":"Input","electrical_type":"Input"}}}),
        ),
        (
            "symbols",
            f.symbol,
            json!({"name":"Shared body","unit":f.unit,"drawings":[{"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}}],"pin_anchors":[{"pin":f.pin,"position":{"x":-2000000,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]}),
        ),
        ("packages", package, json!({"name":"Body"})),
        (
            "padstacks",
            stack,
            json!({"name":"Copper","aperture":{"circle":{"diameter_nm":1000000}}}),
        ),
        (
            "footprints",
            foot,
            json!({"name":"Land","package":package,"pads":{pad.to_string():{"uuid":pad,"name":"1","position":{"x":0,"y":0},"padstack":stack,"layer":0},pad2.to_string():{"uuid":pad2,"name":"2","position":{"x":2000000,"y":0},"padstack":stack,"layer":0}}}),
        ),
        (
            "entities",
            f.entity,
            json!({"name":"Dual","manufacturer":"","prefix":"U","gates":{f.a.to_string():{"uuid":f.a,"name":"A","unit":f.unit,"symbol":f.symbol},f.b.to_string():{"uuid":f.b,"name":"B","unit":f.unit,"symbol":f.symbol}}}),
        ),
        (
            "parts",
            f.part,
            json!({"manufacturer":"Designer","mpn":"FIRST","description":"","datasheet":"","value":"First value","entity":f.entity,"package":package,"default_footprint":foot}),
        ),
        (
            "parts",
            f.other,
            json!({"manufacturer":"Designer","mpn":"SECOND","description":"","datasheet":"","value":"Second value","entity":f.entity,"package":package,"default_footprint":foot}),
        ),
        (
            "pin_pad_maps",
            map,
            json!({"part":f.other,"footprint":foot,"mappings":{pad.to_string():{"gate":f.a,"pin":f.pin},pad2.to_string():{"gate":f.b,"pin":f.pin}},"assign_default":true}),
        ),
    ] {
        write(
            &mut f,
            "datum.library.create",
            json!({"kind":kind,"object_id":id,"pool_path":"pool","fields":fields}),
        );
    }
    let sheet = f.sheet;
    write(
        &mut f,
        "datum.schematic.create_sheet",
        json!({"sheet":sheet,"name":"Root"}),
    );
    f
}
fn params(f: &F, intent: Option<&PartGateSelection>) -> Value {
    json!({"sheet":f.sheet,"instance_path":[],"library_symbol":f.symbol,"explicit_part":intent,"reference":"U1","value":"Chosen","x_nm":10000000,"y_nm":20000000,"rotation_deg":90,"mirrored":true})
}
fn journal(f: &F) -> Vec<u8> {
    std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap()
}

/// Reuse the real library graph for substrate compensation refusal controls.
pub(crate) fn complete_assignment_history_fixture() -> (PathBuf, DesignModel) {
    use native::schematic_delete_proposal::NativeProposalAction as Action;
    let mut f = fixture();
    let request = params(&f, None);
    let placed = write(&mut f, "datum.schematic.place_symbol", request);
    let component = CompletePartSelection::capture(&f.model, f.other, f.a, f.symbol).unwrap();
    let proposal_id = Uuid::new_v4();
    native::schematic_gate_placement::tests::action(
        &mut f.model,
        &f.root,
        Action::CreateAssignment {
            proposal_id,
            symbol: placed,
            instance_path: vec![],
            part: f.other,
            gate: f.a,
            component: Some(component),
        },
    );
    native::schematic_gate_placement::tests::action(
        &mut f.model,
        &f.root,
        Action::AcceptApply { proposal_id },
    );
    (f.root, f.model)
}
#[test]
fn explicit_shared_symbol_multi_part_gate_and_multigate_map_keep_exact_binding_through_cold_history()
 {
    let mut f = fixture();
    let intent = PartGateSelection::capture(&f.model, f.other, f.b, f.symbol).unwrap();
    let old = materialized_pool_object(&f.model, f.other, "parts").unwrap();
    let inferred =
        super::super::resolve_pool_symbol_component_binding(&f.model, Some(&f.symbol.to_string()))
            .unwrap();
    assert!(inferred.binding.is_none());
    let p = prepare(
        &f,
        "datum.schematic.place_symbol",
        params(&f, Some(&intent)),
    )
    .unwrap();
    let id = p.primary_object_id.unwrap();
    let guards: std::collections::BTreeSet<_> = p
        .batch
        .operations
        .iter()
        .filter_map(|op| {
            if let Operation::GuardObjectRevision { object_id, .. } = op {
                Some(*object_id)
            } else {
                None
            }
        })
        .collect();
    for root in [f.other, f.entity, f.unit, f.symbol] {
        assert!(guards.contains(&root));
    }
    let placed = p
        .batch
        .operations
        .iter()
        .find_map(|op| {
            if let Operation::CreateSchematicSymbol { symbol, .. } = op {
                Some(symbol)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(placed["part"], json!(f.other));
    assert_eq!(placed["entity"], json!(f.entity));
    assert_eq!(placed["gate"], json!(f.b));
    assert_eq!(placed["lib_id"], json!(f.symbol));
    assert_eq!(placed["pins"][0]["library_pin"], json!(f.pin));
    assert_eq!(
        placed["pins"][0]["position"],
        json!({"x":10000000,"y":22000000})
    );
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    let ci_id = native::schematic_symbols::placed_symbol_component_instance_id(
        &f.model.project.project_id,
        f.symbol,
        id,
    );
    let ci = &f.model.component_instances[&ci_id];
    assert_eq!(ci.part_ref, Some(f.other));
    assert_eq!(ci.placed_symbol_refs, vec![id]);
    assert_eq!(ci.library_bindings.len(), 1);
    assert_eq!(
        materialized_pool_object(&f.model, f.other, "parts").unwrap(),
        old
    );
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    assert!(!f.model.component_instances.contains_key(&ci_id));
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    let cold = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(cold.component_instances[&ci_id].part_ref, Some(f.other));
    let source = crate::substrate::ElectricalSelectionSnapshot::capture(&cold).unwrap();
    let symbol = &source.schematic_source().unwrap().sheets[&f.sheet].symbols[&id];
    assert_eq!(symbol.gate, Some(f.b));
    assert_eq!(symbol.part, Some(f.other));
    assert_eq!(symbol.pins[0].library_pin, Some(f.pin));
}
#[test]
fn explicit_missing_wrong_pair_stale_and_wrong_occurrence_refuse_without_journal_writes() {
    let f = fixture();
    let good = PartGateSelection::capture(&f.model, f.part, f.a, f.symbol).unwrap();
    let before = journal(&f);
    for field in [
        "part",
        "entity",
        "gate",
        "unit",
        "symbol",
        "part_revision",
        "entity_revision",
        "unit_revision",
        "symbol_revision",
    ] {
        let mut raw = serde_json::to_value(&good).unwrap();
        raw[field] = if field.ends_with("revision") {
            json!(99)
        } else {
            json!(Uuid::new_v4())
        };
        let mut request = params(&f, None);
        request["explicit_part"] = raw;
        assert!(
            prepare(&f, "datum.schematic.place_symbol", request).is_err(),
            "{field}"
        );
        assert_eq!(journal(&f), before);
    }
    let mut request = params(&f, Some(&good));
    request["instance_path"] = json!([Uuid::new_v4()]);
    assert!(prepare(&f, "datum.schematic.place_symbol", request).is_err());
    assert_eq!(journal(&f), before);
    assert!(PartGateSelection::capture(&f.model, f.part, Uuid::new_v4(), f.symbol).is_err());
}
#[test]
fn omitted_explicit_intent_keeps_original_symbol_only_inference_and_ambiguity() {
    let f = fixture();
    let mut request = params(&f, None);
    request.as_object_mut().unwrap().remove("explicit_part");
    let p = prepare(&f, "datum.schematic.place_symbol", request).unwrap();
    assert!(
        !p.batch
            .operations
            .iter()
            .any(|op| matches!(op, Operation::CreateComponentInstance { .. }))
    );
    let s = p
        .batch
        .operations
        .iter()
        .find_map(|op| {
            if let Operation::CreateSchematicSymbol { symbol, .. } = op {
                Some(symbol)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(s["part"], Value::Null);
    assert_eq!(s["gate"], Value::Null);
}
