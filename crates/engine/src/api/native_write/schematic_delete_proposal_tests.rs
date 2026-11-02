//! Actual authored source and persisted Proposal lifecycle, including one-step history.
use super::*;
use crate::{
    api::native_write::{
        commit_prepared, context::PreparedWrite, electrical::topology_tests,
        registry::find_native_write_verb, schematic_connectivity::*,
        schematic_edit::prepare_native_birth, schematic_symbols::build_place_schematic_symbol,
    },
    ir::geometry::Point,
    schematic::{
        HiddenPowerBehavior, Junction, LabelKind, NetLabel, NoConnectMarker, PinElectricalType,
        PlacedSymbol, Schematic, SchematicWire, SymbolDisplayMode, SymbolPin,
    },
    substrate::{ElectricalSelectionSnapshot, ProjectResolver},
};
use std::path::Path;

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "batch deletion fixture")
}
fn source(model: &DesignModel) -> Schematic {
    ElectricalSelectionSnapshot::capture(model)
        .unwrap()
        .schematic_source()
        .unwrap()
        .clone()
}
fn persist(model: &mut DesignModel, root: &Path, write: PreparedWrite) {
    let write = prepare_native_birth(model, write).unwrap();
    commit_prepared(model, root, write).unwrap();
}
fn verb(model: &mut DesignModel, root: &Path, id: &str, params: serde_json::Value) -> Uuid {
    let write = (find_native_write_verb(id).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
    .unwrap();
    let id = write.primary_object_id.unwrap();
    commit_prepared(model, root, write).unwrap();
    id
}
fn request(
    model: &DesignModel,
    root: &Path,
    action: NativeProposalAction,
) -> NativeProposalRequest {
    NativeProposalRequest {
        project_root: root.into(),
        expected_model_revision: model.model_revision.0.clone(),
        expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
        action,
    }
}
fn action(
    model: &mut DesignModel,
    root: &Path,
    action: NativeProposalAction,
) -> NativeProposalResponse {
    let request = request(model, root, action);
    execute(model, &request).unwrap()
}
fn create(id: Uuid, sheet: Uuid, targets: Vec<DeleteTarget>) -> NativeProposalAction {
    NativeProposalAction::CreateDelete {
        proposal_id: id,
        sheet,
        instance_path: vec![],
        targets,
    }
}
fn target(class: DeleteClass, id: Uuid) -> DeleteTarget {
    DeleteTarget { class, id }
}
fn fixture(name: &str) -> (PathBuf, DesignModel, Uuid, Vec<DeleteTarget>) {
    let (root, mut model, _, sheet) = topology_tests::fixture(name);
    let wire = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::zero(),
        to: Point::new(100, 0),
    };
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        name: "D0".into(),
        kind: LabelKind::Local,
        position: wire.from,
    };
    let mut write = build_create_schematic_wire(&model, provenance(), sheet, &wire).unwrap();
    write.batch.operations.extend(
        build_create_schematic_label(&model, provenance(), sheet, &label)
            .unwrap()
            .batch
            .operations,
    );
    persist(&mut model, &root, write);
    let bus = verb(
        &mut model,
        &root,
        "datum.schematic.create_bus",
        serde_json::json!({
        "sheet":sheet,"instance_path":[],"name":"DATA","members":["D0"],
        "points":[{"x":1000,"y":-1000},{"x":1000,"y":1000}],
        "entries":[{"wire":wire.uuid,"x_nm":1000,"y_nm":100,"wire_end":"to"}]}),
    );
    let port = verb(
        &mut model,
        &root,
        "datum.schematic.place_port",
        serde_json::json!({
        "sheet":sheet,"name":"ENABLE","direction":"input","x_nm":5000,"y_nm":5000}),
    );
    let symbol = PlacedSymbol {
        uuid: Uuid::new_v4(),
        part: None,
        entity: None,
        gate: None,
        lib_id: None,
        reference: "R1".into(),
        value: "10k".into(),
        fields: vec![],
        pins: vec![SymbolPin {
            uuid: Uuid::new_v4(),
            library_pin: None,
            number: "1".into(),
            name: "~".into(),
            electrical_type: PinElectricalType::Passive,
            position: Point::new(9000, 9000),
        }],
        position: Point::new(9000, 9000),
        rotation: 0,
        mirrored: false,
        unit_selection: None,
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: vec![],
        hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
    };
    let write = build_place_schematic_symbol(&model, provenance(), sheet, &symbol, None).unwrap();
    persist(&mut model, &root, write);
    let nc = NoConnectMarker {
        uuid: Uuid::new_v4(),
        symbol: symbol.uuid,
        pin: symbol.pins[0].uuid,
        position: symbol.pins[0].position,
    };
    let write = build_create_schematic_noconnect(&model, provenance(), sheet, &nc).unwrap();
    persist(&mut model, &root, write);
    let junction = Junction {
        uuid: Uuid::new_v4(),
        position: wire.from,
    };
    let write = build_create_schematic_junction(&model, provenance(), sheet, &junction).unwrap();
    persist(&mut model, &root, write);
    let targets = vec![
        target(DeleteClass::Symbol, symbol.uuid),
        target(DeleteClass::Wire, wire.uuid),
        target(DeleteClass::Label, label.uuid),
        target(DeleteClass::Port, port),
        target(DeleteClass::BusSection, bus),
        target(DeleteClass::NoConnect, nc.uuid),
        target(DeleteClass::Junction, junction.uuid),
    ];
    (root, model, sheet, targets)
}
#[test]
fn native_batch_delete_proposal_mixed_review_atomic_history_and_reopen() {
    let (root, mut model, sheet, mut targets) = fixture("batch_delete_lifecycle");
    // Force children after every random v4 parent in UUID order. A UUID-sorted
    // deletion batch then deterministically attempts to restore children first.
    let initial = source(&model);
    let initial_sheet = &initial.sheets[&sheet];
    let entry = initial_sheet.bus_entries.values().next().unwrap();
    let marker = initial_sheet.noconnects.values().next().unwrap();
    let mut adverse_entry = entry.clone();
    adverse_entry.uuid = Uuid::from_u128(u128::MAX);
    let mut adverse_marker = marker.clone();
    adverse_marker.uuid = Uuid::from_u128(u128::MAX - 1);
    assert!(adverse_entry.uuid > adverse_entry.bus);
    assert!(adverse_entry.uuid > adverse_entry.wire.unwrap());
    assert!(adverse_marker.uuid > adverse_marker.symbol);
    let mut replacement =
        build_delete_schematic_bus_entry(&model, provenance(), sheet, entry).unwrap();
    for write in [
        build_create_schematic_bus_entry(&model, provenance(), sheet, &adverse_entry).unwrap(),
        build_delete_schematic_noconnect(&model, provenance(), sheet, marker).unwrap(),
        build_create_schematic_noconnect(&model, provenance(), sheet, &adverse_marker).unwrap(),
    ] {
        replacement.batch.operations.extend(write.batch.operations);
    }
    persist(&mut model, &root, replacement);
    targets
        .iter_mut()
        .find(|t| t.class == DeleteClass::NoConnect)
        .unwrap()
        .id = adverse_marker.uuid;
    let before = source(&model);
    let identities = model.electrical_identities.clone();
    let bus_identity = identities
        .values()
        .find(|r| matches!(r.identity, crate::substrate::ElectricalIdentity::Bus { .. }))
        .unwrap()
        .clone();
    let id = Uuid::new_v4();
    let draft = action(&mut model, &root, create(id, sheet, targets.clone()));
    assert_eq!(draft.status, ProposalStatus::Draft);
    let operations = &model.proposals[&id].batch.operations;
    let deletion_index = |object_id| {
        operations
            .iter()
            .position(|op| {
                super::schematic_delete::deleted_object(op).is_some_and(|d| d.id == object_id)
            })
            .unwrap()
    };
    assert!(deletion_index(adverse_entry.uuid) < deletion_index(adverse_entry.bus));
    assert!(deletion_index(adverse_entry.uuid) < deletion_index(adverse_entry.wire.unwrap()));
    assert!(deletion_index(adverse_marker.uuid) < deletion_index(adverse_marker.symbol));
    assert_eq!(draft.selected_count, Some(7));
    assert_eq!(draft.owned_count, Some(1));
    assert_eq!(
        draft.deleted.len(),
        8,
        "selected and owner-derived marker must deduplicate"
    );
    assert!(
        draft
            .deleted
            .iter()
            .filter(|d| matches!(
                d.class.as_str(),
                "wire" | "junction" | "no_connect" | "bus_entry"
            ))
            .all(|d| d.name.is_empty())
    );
    assert_eq!(source(&model), before);
    assert_eq!(model.electrical_identities, identities);
    let tip = crate::revision::transaction_tip(&model.journal);
    let len = model.journal.len();
    assert_eq!(
        action(
            &mut model,
            &root,
            NativeProposalAction::Preview { proposal_id: id }
        )
        .deleted,
        draft.deleted
    );
    assert_eq!(model.journal.len(), len);
    assert_eq!(crate::revision::transaction_tip(&model.journal), tip);
    assert_eq!(
        source(&ProjectResolver::new(&root).resolve().unwrap()),
        before
    );
    let applied = action(
        &mut model,
        &root,
        NativeProposalAction::AcceptApply { proposal_id: id },
    );
    assert_eq!(applied.status, ProposalStatus::Applied);
    assert_eq!(applied.transaction_id, applied.transaction_tip);
    let after = source(&model);
    let after_identities = model.electrical_identities.clone();
    let after_bus = &after_identities[&bus_identity.id];
    assert_eq!(
        after_bus.object_revision.0,
        bus_identity.object_revision.0 + 1
    );
    assert!(
        matches!(&after_bus.identity,crate::substrate::ElectricalIdentity::Bus{representations,scalar_nets,retired:false,..} if representations.is_empty() && scalar_nets.is_empty())
    );
    assert_eq!(model.proposals[&id].batch.operations.iter().filter(|op|matches!(op,Operation::SetElectricalIdentity{record,..} if record.id==bus_identity.id)).count(),1);
    let remaining = &after.sheets[&sheet];
    assert!(
        remaining.symbols.is_empty()
            && remaining.wires.is_empty()
            && remaining.labels.is_empty()
            && remaining.ports.is_empty()
            && remaining.buses.is_empty()
            && remaining.bus_entries.is_empty()
            && remaining.noconnects.is_empty()
            && remaining.junctions.is_empty()
    );
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model), after);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model), before);
    assert_eq!(model.electrical_identities, identities);
    assert_eq!(model.proposals[&id].status, ProposalStatus::Accepted);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model), after);
    assert_eq!(model.electrical_identities, after_identities);
    assert_eq!(
        action(
            &mut model,
            &root,
            NativeProposalAction::Preview { proposal_id: id }
        )
        .status,
        ProposalStatus::Applied
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn native_batch_delete_proposal_external_entry_refuses_whole_set() {
    let (root, mut model, sheet, targets) = fixture("batch_delete_dependencies");
    let wire = targets
        .iter()
        .find(|t| t.class == DeleteClass::Wire)
        .unwrap()
        .clone();
    let label = targets
        .iter()
        .find(|t| t.class == DeleteClass::Label)
        .unwrap()
        .clone();
    let before = source(&model);
    let tip = crate::revision::transaction_tip(&model.journal);
    let r = request(
        &model,
        &root,
        create(Uuid::new_v4(), sheet, vec![wire.clone(), label]),
    );
    assert!(
        execute(&mut model, &r)
            .unwrap_err()
            .to_string()
            .contains("surviving Bus Entry")
    );
    assert_eq!(source(&model), before);
    assert_eq!(crate::revision::transaction_tip(&model.journal), tip);
    // An unselected second Bus still owns a reference even when the first Bus is selected.
    verb(
        &mut model,
        &root,
        "datum.schematic.create_bus",
        serde_json::json!({
        "sheet":sheet,"instance_path":[],"name":"OTHER","members":["D0"],
        "points":[{"x":2000,"y":-1000},{"x":2000,"y":1000}],
        "entries":[{"wire":wire.id,"x_nm":2000,"y_nm":100,"wire_end":"to"}]}),
    );
    let before = source(&model);
    let r = request(&model, &root, create(Uuid::new_v4(), sheet, targets));
    assert!(execute(&mut model, &r).is_err());
    assert_eq!(source(&model), before);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn native_batch_delete_proposal_stale_reject_and_complete_selection_guards() {
    let (root, mut model, sheet, targets) = fixture("batch_delete_guards");
    let before = source(&model);
    let id = Uuid::new_v4();
    let stale = request(
        &model,
        &root,
        create(Uuid::new_v4(), sheet, targets.clone()),
    );
    action(&mut model, &root, create(id, sheet, targets.clone()));
    assert!(
        execute(&mut model, &stale).is_err(),
        "metadata history makes a stale tip invalid"
    );
    let rejected = action(
        &mut model,
        &root,
        NativeProposalAction::Reject { proposal_id: id },
    );
    assert_eq!(rejected.status, ProposalStatus::Rejected);
    assert_eq!(source(&model), before);
    let r = request(
        &model,
        &root,
        NativeProposalAction::AcceptApply { proposal_id: id },
    );
    assert!(execute(&mut model, &r).is_err());
    for action in [
        create(
            Uuid::new_v4(),
            sheet,
            vec![targets[0].clone(), targets[0].clone()],
        ),
        create(
            Uuid::new_v4(),
            sheet,
            vec![
                targets[0].clone(),
                target(DeleteClass::Wire, Uuid::new_v4()),
            ],
        ),
        NativeProposalAction::CreateDelete {
            proposal_id: Uuid::new_v4(),
            sheet,
            instance_path: vec![Uuid::new_v4()],
            targets: targets.clone(),
        },
    ] {
        let r = request(&model, &root, action);
        assert!(execute(&mut model, &r).is_err());
    }
    assert_eq!(
        source(&ProjectResolver::new(&root).resolve().unwrap()),
        before
    );
    assert!(
        serde_json::from_value::<DeleteTarget>(
            serde_json::json!({"class":"bus_entry","id":Uuid::new_v4()})
        )
        .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_batch_delete_proposal_bound_symbol_and_stale_design_refuse() {
    let mut fixture =
        crate::api::native_write::electrical::binding_fixture::fixture("batch_delete_bound");
    let r = request(
        &fixture.model,
        &fixture.root,
        create(
            Uuid::new_v4(),
            fixture.sheet,
            vec![target(DeleteClass::Symbol, fixture.symbol.uuid)],
        ),
    );
    let tip = crate::revision::transaction_tip(&fixture.model.journal);
    assert!(
        execute(&mut fixture.model, &r)
            .unwrap_err()
            .to_string()
            .contains("linked to a component")
    );
    assert_eq!(
        crate::revision::transaction_tip(&fixture.model.journal),
        tip
    );
    std::fs::remove_dir_all(fixture.root).unwrap();
    let (root, mut model, sheet, targets) = self::fixture("batch_delete_stale_design");
    let id = Uuid::new_v4();
    action(&mut model, &root, create(id, sheet, targets));
    verb(
        &mut model,
        &root,
        "datum.schematic.place_port",
        serde_json::json!({
        "sheet":sheet,"name":"LATER","direction":"passive","x_nm":19000,"y_nm":19000}),
    );
    let before = source(&model);
    let tip = crate::revision::transaction_tip(&model.journal);
    let r = request(
        &model,
        &root,
        NativeProposalAction::AcceptApply { proposal_id: id },
    );
    assert!(execute(&mut model, &r).is_err());
    assert_eq!(model.proposals[&id].status, ProposalStatus::Draft);
    assert_eq!(crate::revision::transaction_tip(&model.journal), tip);
    assert_eq!(source(&model), before);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_batch_delete_proposal_combines_shared_bus_projection_removals_once() {
    use crate::substrate::{ElectricalIdentity, ElectricalOccurrence};
    let (root, mut model, sheet, targets) = fixture("batch_delete_shared_projection");
    let selected: Vec<_> = targets
        .into_iter()
        .filter(|t| matches!(t.class, DeleteClass::Label | DeleteClass::Port))
        .collect();
    let mut declaration = model
        .electrical_identities
        .values()
        .find(|r| matches!(r.identity, ElectricalIdentity::Bus { .. }))
        .unwrap()
        .clone();
    let ElectricalIdentity::Bus {
        representations, ..
    } = &mut declaration.identity
    else {
        unreachable!()
    };
    for target in &selected {
        representations.insert(ElectricalOccurrence {
            class: target.class.source_class().into(),
            source_id: target.id,
            instance_path: vec![],
        });
    }
    let write = crate::api::native_write::electrical::build_set_electrical_identity(
        &model,
        provenance(),
        declaration,
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let before = source(&model);
    let identities = model.electrical_identities.clone();
    let original = identities
        .values()
        .find(|r| matches!(r.identity, ElectricalIdentity::Bus { .. }))
        .unwrap();
    let id = Uuid::new_v4();
    action(&mut model, &root, create(id, sheet, selected));
    let changes: Vec<_> = model.proposals[&id]
        .batch
        .operations
        .iter()
        .filter_map(|op| match op {
            Operation::SetElectricalIdentity { record, .. } if record.id == original.id => {
                Some(record)
            }
            _ => None,
        })
        .collect();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].object_revision.0, original.object_revision.0 + 1);
    assert!(
        matches!(&changes[0].identity,ElectricalIdentity::Bus{representations,..} if representations.len()==1 && representations.iter().all(|r|r.class=="buses"))
    );
    action(
        &mut model,
        &root,
        NativeProposalAction::AcceptApply { proposal_id: id },
    );
    let after = source(&model);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model), before);
    assert_eq!(model.electrical_identities, identities);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model), after);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_batch_delete_proposal_explicit_bus_semantic_write_is_not_silently_remapped() {
    let (root, mut model, sheet, targets) = fixture("batch_delete_explicit_semantic");
    let before = source(&model);
    let identities = model.electrical_identities.clone();
    let tip = crate::revision::transaction_tip(&model.journal);
    let mut prepared = crate::api::native_write::schematic_delete::build(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        WriteProvenance::new("test", CommitSource::Manual, "explicit semantic intent"),
        sheet,
        &[],
        &targets,
    )
    .unwrap();
    let bus_edit = prepared
        .batch
        .operations
        .iter_mut()
        .find_map(|op| match op {
            Operation::SetElectricalIdentity { record, .. }
                if matches!(
                    record.identity,
                    crate::substrate::ElectricalIdentity::Bus { .. }
                ) =>
            {
                Some(record)
            }
            _ => None,
        })
        .unwrap();
    let crate::substrate::ElectricalIdentity::Bus { name, .. } = &mut bus_edit.identity else {
        unreachable!()
    };
    *name = "Explicit semantic change".into();
    let error = create_draft_proposal_from_batch(
        &mut model,
        &root,
        ProposalCreateRequest {
            proposal_id: Some(Uuid::new_v4()),
            batch: prepared.batch,
            rationale: "Explicit semantic control".into(),
            source: ProposalSource::Manual,
            checks_run: vec![],
            finding_fingerprints: vec![],
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("Net"), "{error}");
    assert_eq!(source(&model), before);
    assert_eq!(model.electrical_identities, identities);
    assert_eq!(crate::revision::transaction_tip(&model.journal), tip);
    std::fs::remove_dir_all(root).unwrap();
}
