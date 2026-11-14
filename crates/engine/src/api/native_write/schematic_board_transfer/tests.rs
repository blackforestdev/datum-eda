//! Source-native linked initial transfer through the same public writer facades.
#[path = "field_revision_tests.rs"]
mod field_revision_tests;
use super::*;
use crate::api::native_write::{
    self as native, commit_prepared,
    registry::{NativeWriteContext, find_native_write_verb},
};
use crate::substrate::{
    CommitSource, DesignModel, ElectricalSelectionSnapshot, ProjectResolver, SourceShardKind,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::PathBuf;

struct F {
    root: PathBuf,
    model: DesignModel,
    sheet: Uuid,
    map: Uuid,
    request: TransferRequest,
}
impl Drop for F {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "transfer-test",
        CommitSource::Manual,
        "Initial connected PCB transfer",
    )
}
fn write(model: &mut DesignModel, root: &std::path::Path, verb: &str, params: Value) -> Uuid {
    let p = (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
    .unwrap();
    let id = p.primary_object_id.unwrap_or(Uuid::nil());
    commit_prepared(model, root, p).unwrap();
    id
}
fn bytes(f: &F) -> BTreeMap<String, Vec<u8>> {
    let mut result: BTreeMap<_, _> = f
        .model
        .source_shards
        .iter()
        .map(|s| {
            (
                s.relative_path.clone(),
                std::fs::read(f.root.join(&s.relative_path)).unwrap(),
            )
        })
        .collect();
    for p in [
        crate::substrate::transaction_journal_path(&f.root),
        f.root.join(".datum/journal/cursor.json"),
    ] {
        if p.exists() {
            result.insert(
                p.strip_prefix(&f.root).unwrap().to_string_lossy().into(),
                std::fs::read(p).unwrap(),
            );
        }
    }
    result
}
fn fixture(adopt: bool) -> F {
    fixture_mode(adopt, false)
}
fn fixture_mode(adopt: bool, complete: bool) -> F {
    let root = native::test_support::temp_project_root("connected_board_transfer");
    let report = native::genesis::bootstrap_native_project(
        &root,
        native::genesis::GenesisSpec {
            project_name: "Connected transfer".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let (unit, pin, symbol, entity, gate, package, stack, footprint, pad, part, map, sheet) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
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
        (
            "units",
            unit,
            json!({"name":"Passive pin","pins":{pin.to_string():{"name":"Signal","electrical_type":"Passive"}}}),
        ),
        (
            "symbols",
            symbol,
            json!({"name":"Transfer body","unit":unit,"drawings":[{"Rect":{"min":{"x":-500000,"y":-500000},"max":{"x":500000,"y":500000},"width":100000}}],"pin_anchors":[{"pin":pin,"position":{"x":-2000000,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]}),
        ),
        ("packages", package, json!({"name":"Package"})),
        (
            "padstacks",
            stack,
            json!({"name":"Copper","aperture":{"circle":{"diameter_nm":1000000}}}),
        ),
        (
            "footprints",
            footprint,
            json!({"name":"One pad","package":package,"pads":{pad.to_string():{"uuid":pad,"name":"1","position":{"x":0,"y":0},"padstack":stack,"layer":1}}}),
        ),
        (
            "entities",
            entity,
            json!({"name":"Entity","manufacturer":"Designer","prefix":"U","gates":{gate.to_string():{"uuid":gate,"name":"G","unit":unit,"symbol":symbol}}}),
        ),
        (
            "parts",
            part,
            json!({"manufacturer":"Designer","mpn":"TRANSFER","description":"","datasheet":"","value":"Signal","entity":entity,"package":package,"default_footprint":footprint}),
        ),
        (
            "pin_pad_maps",
            map,
            json!({"part":part,"footprint":footprint,"mappings":{pad.to_string():{"gate":gate,"pin":pin}},"assign_default":complete}),
        ),
    ] {
        write(
            &mut model,
            &root,
            "datum.library.create",
            json!({"kind":kind,"object_id":id,"pool_path":"pool","fields":fields}),
        );
    }
    write(
        &mut model,
        &root,
        "datum.schematic.create_sheet",
        json!({"sheet":sheet,"name":"Root"}),
    );
    for (reference, x) in [("U1", 0), ("U2", 10000000)] {
        let intent = native::schematic_library::part_placement::PartGateSelection::capture(
            &model, part, gate, symbol,
        )
        .unwrap();
        write(
            &mut model,
            &root,
            "datum.schematic.place_symbol",
            json!({"sheet":sheet,"instance_path":[],"library_symbol":symbol,"explicit_part":intent,"reference":reference,"value":"Signal","x_nm":x,"y_nm":0}),
        );
    }
    write(
        &mut model,
        &root,
        "datum.schematic.draw_wire",
        json!({"sheet":sheet,"instance_path":[],"points":[{"x":-2000000,"y":0},{"x":8000000,"y":0}]}),
    );
    if adopt {
        let p = native::board_layout::source_profile::build_adopt_board_source_profile(
            &model,
            provenance(),
            report.board_uuid,
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
    }
    let request = TransferRequest {
        sheet,
        choices: model
            .component_instances
            .values()
            .map(|ci| TransferChoice {
                component_instance: ci.id,
                pin_pad_map: map,
            })
            .collect(),
        edit_seed: Uuid::new_v4(),
        origin: Point::zero(),
        pitch_nm: 5000000,
        layer: 1,
    };
    F {
        root,
        model,
        sheet,
        map,
        request,
    }
}

#[test]
fn complete_component_handoff_uses_authored_physical_choice_without_per_design_mapping() {
    let mut f = fixture_mode(true, true);
    f.request.choices.clear();
    let before = bytes(&f);
    let plan = plan_transfer(&f.model, &f.request).unwrap();
    assert_eq!(plan.ready_count(), 2);
    assert!(
        plan.components
            .iter()
            .all(|row| row.pin_pad_map == Some(f.map) && row.footprint.is_some())
    );
    assert_eq!(bytes(&f), before);
    let prepared = prepare_transfer(&f.model, provenance(), &f.request).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let cold = ProjectResolver::new(&f.root).resolve().unwrap();
    let repeated = plan_transfer(&cold, &f.request).unwrap();
    assert_eq!(repeated.ready_count(), 0);
    assert!(
        repeated
            .components
            .iter()
            .all(|row| row.status == TransferStatus::Existing)
    );
    assert_eq!(cold.component_instances.len(), 2);
}

#[test]
fn repeated_explicit_default_map_cannot_bypass_declared_footprint_or_padstack_drift() {
    use native::library::{PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec};
    for kind in ["footprints", "padstacks"] {
        let mut f = fixture_mode(true, true);
        assert_eq!(
            plan_transfer(&f.model, &f.request).unwrap().ready_count(),
            2
        );
        let first = plan_transfer(&f.model, &f.request)
            .unwrap()
            .components
            .remove(0);
        let footprint = native::schematic_library::materialized_pool_object(
            &f.model,
            first.footprint.unwrap(),
            "footprints",
        )
        .unwrap();
        let id = if kind == "footprints" {
            first.footprint.unwrap()
        } else {
            footprint["pads"]
                .as_object()
                .unwrap()
                .values()
                .next()
                .unwrap()["padstack"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap()
        };
        let mut raw =
            native::schematic_library::materialized_pool_object(&f.model, id, kind).unwrap();
        if kind == "footprints" {
            raw["pads"]
                .as_object_mut()
                .unwrap()
                .values_mut()
                .next()
                .unwrap()["position"]["x"] = json!(500000);
        } else {
            raw["aperture"]["circle"]["diameter_nm"] = json!(2000000);
        }
        let prepared = native::library::build_pool_library_write(
            &f.model,
            provenance(),
            Some("pool"),
            vec![Spec::Set {
                target: Target::new("pool", kind, id),
                object: raw,
            }],
        )
        .unwrap();
        commit_prepared(&mut f.model, &f.root, prepared).unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        let before = bytes(&f);
        let plan = plan_transfer(&f.model, &f.request).unwrap();
        assert_eq!(
            plan.ready_count(),
            0,
            "same explicit Map must not bypass {kind} drift"
        );
        assert!(plan.components.iter().all(|row| {
            row.reason
                .as_deref()
                .is_some_and(|reason| reason.contains("physical definition changed"))
        }));
        assert!(prepare_transfer(&f.model, provenance(), &f.request).is_err());
        assert_eq!(bytes(&f), before);
    }
}

#[test]
fn initial_transfer_plan_is_readonly_explicit_and_refuses_unprepared_source() {
    let f = fixture(false);
    let before = bytes(&f);
    let mut request = f.request.clone();
    request.choices.clear();
    let plan = plan_transfer(&f.model, &request).unwrap();
    assert!(plan.requires_profile_adoption);
    assert_eq!(plan.ready_count(), 0);
    assert_eq!(plan.components.len(), 2);
    assert!(plan.components.iter().all(|c| !c.map_options.is_empty()));
    assert_eq!(bytes(&f), before);
    assert!(prepare_transfer(&f.model, provenance(), &f.request).is_err());
    assert_eq!(bytes(&f), before);
}

#[test]
fn initial_transfer_creates_owned_mapped_pads_nets_and_noop_repeat() {
    let mut f = fixture(true);
    let before = bytes(&f);
    let journal = f.model.journal.len();
    let plan = plan_transfer(&f.model, &f.request).unwrap();
    assert_eq!(plan.ready_count(), 2);
    let p = prepare_transfer(&f.model, provenance(), &f.request).unwrap();
    let second = prepare_transfer(&f.model, provenance(), &f.request).unwrap();
    assert_eq!(p, second);
    assert_eq!(bytes(&f), before, "preview must not write");
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.model.journal.len(), journal + 1);
    let board = f
        .model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    assert_eq!(board["packages"].as_object().unwrap().len(), 2);
    assert!(
        board["packages"]
            .as_object()
            .unwrap()
            .values()
            .all(|p| p["value"] == "Signal")
    );
    for ci in f.model.component_instances.values() {
        assert_eq!(ci.placed_package_refs.len(), 1);
        let pads = &board["component_pads"][ci.placed_package_refs[0].to_string()];
        assert_eq!(pads.as_array().unwrap().len(), 1);
        assert_eq!(
            pads[0]["source_binding"]["component_instance"]["object_id"],
            json!(ci.id)
        );
        assert_eq!(
            pads[0]["source_binding"]["component_instance"]["object_revision"],
            json!(ci.object_revision)
        );
        assert_eq!(
            pads[0]["source_binding"]["pin_pad_map"]["object_id"],
            json!(f.map)
        );
    }
    let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
    let airwires = snapshot.unrouted_display(&f.model.model_revision).unwrap();
    assert_eq!(airwires.len(), 1);
    assert!(
        snapshot
            .unrouted_display(&ModelRevision("stale".into()))
            .is_err()
    );
    let first = &plan.components[0].connections[0];
    let net = snapshot
        .net_for_origin(&f.model.model_revision, &first.schematic_terminal)
        .unwrap();
    assert_eq!(net.owned.iter().filter(|o| o.class == "pads").count(), 2);
    assert_eq!(net.owned.iter().filter(|o| o.class == "pins").count(), 2);
    let stable = bytes(&f);
    let repeated = plan_transfer(&f.model, &f.request).unwrap();
    assert_eq!(repeated.ready_count(), 0);
    assert!(
        repeated
            .components
            .iter()
            .all(|c| c.status == TransferStatus::Existing)
    );
    assert!(prepare_transfer(&f.model, provenance(), &f.request).is_err());
    assert_eq!(bytes(&f), stable);
    assert_eq!(f.sheet, f.request.sheet);
}

#[test]
fn connected_transfer_history_restores_complete_mapped_graph_after_cold_resolve() {
    let mut f = fixture(true);
    let terminal = plan_transfer(&f.model, &f.request).unwrap().components[0].connections[0]
        .schematic_terminal
        .clone();
    let symbols_before = f
        .model
        .materialized_source_shard_value(SourceShardKind::SchematicRoot)
        .unwrap();
    let p = prepare_transfer(&f.model, provenance(), &f.request).unwrap();
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    let packages: Vec<_> = f
        .model
        .component_instances
        .values()
        .flat_map(|ci| ci.placed_package_refs.clone())
        .collect();
    for redo in [false, true] {
        let tip = crate::revision::transaction_tip(&f.model.journal).unwrap();
        native::history::commit_history(
            &mut f.model,
            &f.root,
            native::history::HistoryGuards { expected_tip: tip },
            redo,
            provenance(),
        )
        .unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(
            f.model
                .materialized_source_shard_value(SourceShardKind::SchematicRoot)
                .unwrap(),
            symbols_before
        );
        let board = f
            .model
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .unwrap();
        assert_eq!(
            board["packages"].as_object().unwrap().len(),
            if redo { 2 } else { 0 }
        );
        let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
        assert_eq!(
            snapshot
                .unrouted_display(&f.model.model_revision)
                .unwrap()
                .len(),
            if redo { 1 } else { 0 }
        );
        if redo {
            for id in &packages {
                assert!(board["packages"].get(id.to_string()).is_some());
            }
            let plan = plan_transfer(&f.model, &f.request).unwrap();
            assert!(
                plan.components
                    .iter()
                    .all(|c| c.status == TransferStatus::Existing)
            );
            let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
            let net = snapshot
                .net_for_origin(&f.model.model_revision, &terminal)
                .unwrap();
            assert_eq!(net.owned.iter().filter(|o| o.class == "pads").count(), 2);
        }
    }
}

#[test]
fn transfer_incomplete_net_and_tampered_birth_refuse_without_writes() {
    let mut f = fixture(true);
    let before = bytes(&f);
    let mut partial = f.request.clone();
    partial.choices.truncate(1);
    assert_eq!(plan_transfer(&f.model, &partial).unwrap().ready_count(), 0);
    assert!(prepare_transfer(&f.model, provenance(), &partial).is_err());
    let mut p = prepare_transfer(&f.model, provenance(), &f.request).unwrap();
    for op in &mut p.batch.operations {
        if let crate::substrate::Operation::CreateBoardPackage { materialized, .. } = op {
            materialized["component_pads"][0]["source_binding"]["pin_pad_map"] = Value::Null;
            break;
        }
    }
    assert!(commit_prepared(&mut f.model, &f.root, p).is_err());
    assert_eq!(bytes(&f), before);
}

#[test]
fn initial_transfer_reports_conflicting_values_of_actual_authored_component_gates() {
    use native::schematic_delete_proposal::NativeProposalAction as Action;
    use native::schematic_gate_placement::tests as gates;
    let (root, mut model, _, request) = gates::setup("transfer-gate-values");
    gates::action(
        &mut model,
        &root,
        Action::CreateGatePlacement(request.clone()),
    );
    gates::action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: request.proposal_id,
        },
    );
    let request = TransferRequest {
        sheet: request.sheet,
        choices: vec![],
        edit_seed: Uuid::new_v4(),
        origin: Point::zero(),
        pitch_nm: 5_000_000,
        layer: 1,
    };
    let f = F {
        root,
        model,
        sheet: request.sheet,
        map: Uuid::nil(),
        request,
    };
    let before = bytes(&f);
    let plan = plan_transfer(&f.model, &f.request).unwrap();
    let row = plan
        .components
        .iter()
        .find(|c| c.symbols.len() == 2)
        .unwrap();
    assert_eq!(row.status, TransferStatus::Unresolved);
    assert!(
        row.reason
            .as_deref()
            .unwrap()
            .contains("instance values disagree")
    );
    assert!(row.value.is_none());
    assert_eq!(bytes(&f), before);
}
