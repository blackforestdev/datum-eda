//! Real native Part births, copied connectivity, immutable proposals and history.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    context::{BatchComposer, WriteProvenance},
    design_clipboard::{ClipboardPlacement, ClipboardTarget},
    design_clipboard_edit::{self, ClipboardContactPolicy, DesignEditIntent},
    genesis::{GenesisSpec, bootstrap_native_project},
    registry::{NativeWriteContext, find_native_write_verb},
    schematic_delete_proposal::{NativeProposalAction, NativeProposalRequest, execute},
    schematic_symbol_payload::source_sheet_payload,
    test_support::temp_project_root,
};
use crate::substrate::{CommitSource, ProjectResolver};
use serde_json::json;
use std::path::{Path, PathBuf};

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Copy native component block")
}
fn write(model: &mut DesignModel, root: &Path, verb: &str, params: Value) -> Uuid {
    let prepared = (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
    .unwrap();
    let id = prepared.primary_object_id.unwrap_or_default();
    commit_prepared(model, root, prepared).unwrap();
    id
}
struct Fixture {
    root: PathBuf,
    model: DesignModel,
    sheet: Uuid,
    symbols: Vec<Uuid>,
    wire: Uuid,
}
fn fixture() -> Fixture {
    let root = temp_project_root("component_block_clipboard");
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Component block".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let (unit, pin, symbol, entity, gate, package, part, sheet) = (
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
            json!({"name":"Passive","pins":{pin.to_string():{"name":"Contact","electrical_type":"Passive"}}}),
        ),
        (
            "symbols",
            symbol,
            json!({"name":"Block body","unit":unit,"drawings":[{"Rect":{"min":{"x":-1000000,"y":-500000},"max":{"x":1000000,"y":500000},"width":100000}}],"pin_anchors":[{"pin":pin,"position":{"x":-2000000,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]}),
        ),
        ("packages", package, json!({"name":"Body"})),
        (
            "entities",
            entity,
            json!({"name":"Single gate","prefix":"R","manufacturer":"","gates":{gate.to_string():{"uuid":gate,"name":"A","unit":unit,"symbol":symbol}}}),
        ),
        (
            "parts",
            part,
            json!({"manufacturer":"Designer","mpn":"BLOCK","description":"","datasheet":"","value":"Passive","entity":entity,"package":package}),
        ),
    ] {
        write(
            &mut model,
            &root,
            "datum.library.create",
            json!({"kind":kind,"pool_path":"pool","object_id":id,"fields":fields}),
        );
    }
    write(
        &mut model,
        &root,
        "datum.schematic.create_sheet",
        json!({"sheet":sheet,"name":"Root"}),
    );
    let mut symbols = vec![];
    for (reference, x) in [("R1", 0), ("R2", 10000000)] {
        symbols.push(write(&mut model, &root, "datum.schematic.place_symbol", json!({"sheet":sheet,"instance_path":[],"library_symbol":symbol,"reference":reference,"value":"Passive","x_nm":x,"y_nm":0})));
    }
    write(
        &mut model,
        &root,
        "datum.schematic.draw_wire",
        json!({"sheet":sheet,"instance_path":[],"points":[{"x":-2000000,"y":0},{"x":8000000,"y":0}]}),
    );
    let wire = *ElectricalSelectionSnapshot::capture(&model)
        .unwrap()
        .schematic_source()
        .unwrap()
        .sheets[&sheet]
        .wires
        .keys()
        .next()
        .unwrap();
    // Use the ordinary bound-field producer so its shared owner repinning is part
    // of the fixture, exactly as in the application property editor.
    write(
        &mut model,
        &root,
        "datum.schematic.set_symbol_field",
        json!({"symbol":symbols[0],"instance_path":[],"field":"custom","key":"Tolerance","value":"1%"}),
    );
    let field = ElectricalSelectionSnapshot::capture(&model)
        .unwrap()
        .schematic_source()
        .unwrap()
        .sheets[&sheet]
        .symbols[&symbols[0]]
        .fields[0]
        .uuid;
    write(
        &mut model,
        &root,
        "datum.schematic.set_symbol_field",
        json!({"symbol":symbols[0],"instance_path":[],"field":"custom","field_id":field,"visible":true,"position":{"x":1000000,"y":1000000}}),
    );
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        model.component_instances.len(),
        2,
        "Fixture must retain current authored owners after cold resolution"
    );
    Fixture {
        root,
        model,
        sheet,
        symbols,
        wire,
    }
}
fn capture(f: &Fixture) -> DesignClipboardBundle {
    let mut targets: Vec<_> = f
        .symbols
        .iter()
        .map(|id| ClipboardTarget {
            class: ClipboardClass::Symbol,
            id: *id,
        })
        .collect();
    targets.push(ClipboardTarget {
        class: ClipboardClass::Wire,
        id: f.wire,
    });
    DesignClipboardBundle::capture(&f.model, f.sheet, &[], &targets, Point::zero()).unwrap()
}
fn edit(bundle: DesignClipboardBundle) -> DesignEditIntent {
    DesignEditIntent::PlaceBundle {
        bundle,
        placement: ClipboardPlacement {
            position: Point::new(30000000, 20000000),
            rotation_deg: 90,
            mirrored: true,
        },
        edit_seed: Uuid::new_v4(),
        contact_policy: ClipboardContactPolicy::DrawWireEndpoints,
    }
}
fn act(f: &mut Fixture, action: NativeProposalAction) {
    let request = NativeProposalRequest {
        project_root: f.root.clone(),
        expected_model_revision: f.model.model_revision.0.clone(),
        expected_transaction_tip: f.model.journal.last().map(|t| t.transaction_id),
        action,
    };
    execute(&mut f.model, &request).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
}
#[test]
fn native_component_block_proposal_preserves_sources_allocates_owners_and_replays_cold_history() {
    let mut f = fixture();
    let before = source_sheet_payload(&f.model, f.sheet, f.symbols[0]).unwrap();
    let owners = f.model.component_instances.clone();
    let mut bundle = capture(&f);
    bundle.prepare_placement(&f.model).unwrap();
    let edit = edit(bundle);
    let DesignEditIntent::PlaceBundle {
        bundle,
        placement,
        edit_seed,
        ..
    } = &edit
    else {
        unreachable!()
    };
    let preview = design_clipboard_edit::preview_placement(
        ElectricalSelectionSnapshot::capture(&f.model)
            .unwrap()
            .schematic_source()
            .unwrap()
            .sheets
            .get(&f.sheet)
            .unwrap(),
        bundle,
        placement,
        edit_seed,
        ClipboardContactPolicy::DrawWireEndpoints,
    )
    .unwrap();
    assert_eq!(preview.symbols.len(), 4);
    let new: Vec<_> = preview
        .symbols
        .values()
        .filter(|s| !f.symbols.contains(&s.uuid))
        .collect();
    assert_eq!(
        new.iter()
            .map(|s| s.reference.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["R3", "R4"])
    );
    let proposal_id = Uuid::new_v4();
    let sheet = f.sheet;
    act(
        &mut f,
        NativeProposalAction::CreateDesignEdit {
            proposal_id,
            sheet,
            instance_path: vec![],
            edit: edit.clone(),
        },
    );
    assert_eq!(
        source_sheet_payload(&f.model, f.sheet, f.symbols[0]).unwrap(),
        before
    );
    let cached = f.model.proposals[&proposal_id].batch.clone();
    act(&mut f, NativeProposalAction::Preview { proposal_id });
    assert_eq!(f.model.proposals[&proposal_id].batch, cached);
    act(&mut f, NativeProposalAction::AcceptApply { proposal_id });
    let after = source_sheet_payload(&f.model, f.sheet, f.symbols[0]).unwrap();
    assert_eq!(
        serde_json::from_value::<crate::schematic::Sheet>(after.clone()).unwrap(),
        preview
    );
    assert_eq!(f.model.component_instances.len(), 4);
    for (id, owner) in &owners {
        assert_eq!(f.model.component_instances.get(id), Some(owner));
    }
    for symbol in &new {
        let owner = f
            .model
            .component_instances
            .values()
            .find(|ci| ci.placed_symbol_refs == [symbol.uuid])
            .unwrap();
        assert!(owner.placed_package_refs.is_empty());
        assert!(!owners.contains_key(&owner.id));
        assert!(
            symbol
                .pins
                .iter()
                .all(|pin| !before.to_string().contains(&pin.uuid.to_string()))
        );
        assert_eq!(
            f.model
                .materialized_placed_symbol_definition(symbol.uuid, library(symbol).unwrap())
                .unwrap(),
            Some(
                bundle
                    .symbols
                    .as_ref()
                    .unwrap()
                    .sources
                    .values()
                    .next()
                    .unwrap()
                    .definition
                    .clone()
            )
        );
    }
    let field_owner = new.iter().find(|s| !s.fields.is_empty()).unwrap();
    assert_eq!(
        field_owner.fields[0].position,
        Some(Point::new(29_000_000, 19_000_000))
    );
    assert_eq!(field_owner.fields[0].key, "Tolerance");
    assert_eq!(field_owner.fields[0].value, "1%");
    assert!(field_owner.fields[0].visible);
    for redo in [false, true] {
        if redo {
            f.model
                .commit_journal_redo(&f.root, provenance().into())
                .unwrap();
        } else {
            f.model
                .commit_journal_undo(&f.root, provenance().into())
                .unwrap();
        }
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(
            source_sheet_payload(&f.model, f.sheet, f.symbols[0]).unwrap(),
            if redo { after.clone() } else { before.clone() }
        );
        assert_eq!(f.model.component_instances.len(), if redo { 4 } else { 2 });
    }
    std::fs::remove_dir_all(f.root).unwrap();
}
fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    fn visit(root: &Path, path: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                visit(root, &p, out)
            } else {
                out.insert(
                    p.strip_prefix(root).unwrap().to_owned(),
                    std::fs::read(p).unwrap(),
                );
            }
        }
    }
    visit(root, root, &mut out);
    out
}
#[test]
fn symbol_block_cross_project_tampering_and_stale_source_refuse_without_writes() {
    let mut f = fixture();
    let bundle = capture(&f);
    let mut foreign = bundle.clone();
    foreign.source_project = Uuid::new_v4();
    let mut forged = bundle.clone();
    forged
        .items
        .iter_mut()
        .find(|i| i.class == ClipboardClass::Symbol)
        .unwrap()
        .raw["pins"][0]["name"] = "Forged".into();
    for bundle in [foreign, forged] {
        let before = files(&f.root);
        assert!(
            design_clipboard_edit::build(
                &NativeWriteContext {
                    model: &f.model,
                    project_root: &f.root
                },
                provenance(),
                f.sheet,
                &[],
                &edit(bundle)
            )
            .is_err()
        );
        assert_eq!(files(&f.root), before);
    }
    let symbol = f.symbols[0];
    write(
        &mut f.model,
        &f.root,
        "datum.schematic.set_symbol_value",
        json!({"symbol":symbol,"instance_path":[],"value":"Changed after copy"}),
    );
    let before = files(&f.root);
    let result = design_clipboard_edit::build(
        &NativeWriteContext {
            model: &f.model,
            project_root: &f.root,
        },
        provenance(),
        f.sheet,
        &[],
        &edit(bundle),
    );
    assert!(result.is_err());
    assert_eq!(files(&f.root), before);
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn canonical_custom_owner_role_refuses_entire_block_without_copy_writes() {
    let mut f = fixture();
    let symbol = f.symbols[0];
    let owner = f
        .model
        .component_instances
        .values()
        .find(|ci| ci.placed_symbol_refs == [symbol])
        .unwrap()
        .id;
    let shard = f
        .model
        .source_shards
        .iter()
        .find(|s| s.shard_id == f.model.objects[&owner].source_shard_id)
        .unwrap();
    let previous = f
        .model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap()["component_instance"]
        .clone();
    let mut next = previous.clone();
    next["object_revision"] = json!(f.model.objects[&owner].object_revision.0 + 1);
    next["placed_symbol_roles"][symbol.to_string()]["label"] = json!("User-owned functional label");
    let prepared = BatchComposer::compose(&f.model, provenance())
        .push_op(Operation::SetComponentInstance {
            component_instance_id: owner,
            previous_component_instance: previous,
            component_instance: next,
        })
        .finish()
        .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(
        f.model.component_instances[&owner].placed_symbol_roles[&symbol]
            .label
            .as_deref(),
        Some("User-owned functional label")
    );
    let targets: Vec<_> = f
        .symbols
        .iter()
        .map(|id| ClipboardTarget {
            class: ClipboardClass::Symbol,
            id: *id,
        })
        .chain(std::iter::once(ClipboardTarget {
            class: ClipboardClass::Wire,
            id: f.wire,
        }))
        .collect();
    let before = files(&f.root);
    let result = DesignClipboardBundle::capture(&f.model, f.sheet, &[], &targets, Point::zero());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("owner role overrides")
    );
    assert_eq!(files(&f.root), before);
    std::fs::remove_dir_all(f.root).unwrap();
}
