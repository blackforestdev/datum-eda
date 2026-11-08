//! Canonical map-only/default transactions, source preservation and unsafe ownership.
use super::*;
use crate::api::native_write::{self as native, commit_prepared, registry::find_native_write_verb};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver};
use std::path::PathBuf;
struct F {
    root: PathBuf,
    model: DesignModel,
    part: Uuid,
    no_default: Uuid,
    entity: Uuid,
    unit: Uuid,
    pin: Uuid,
    gate: Uuid,
    gate2: Uuid,
    package: Uuid,
    foot: Uuid,
    wrong_foot: Uuid,
    pad: Uuid,
    pad2: Uuid,
    symbol: Uuid,
    placed: Uuid,
    sheet: Uuid,
}
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Author gate-qualified map")
}
fn write(f: &mut F, verb: &str, params: Value) -> Uuid {
    let p = (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model: &f.model,
            project_root: &f.root,
        },
        provenance(),
        params,
    )
    .unwrap();
    let id = p.primary_object_id.unwrap_or(Uuid::nil());
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    id
}
fn fixture() -> F {
    let root = super::super::super::test_support::temp_project_root("map_gui");
    native::genesis::bootstrap_native_project(
        &root,
        native::genesis::GenesisSpec {
            project_name: "Gate mapping".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut f = F {
        model: ProjectResolver::new(&root).resolve().unwrap(),
        root,
        part: Uuid::new_v4(),
        no_default: Uuid::new_v4(),
        entity: Uuid::new_v4(),
        unit: Uuid::new_v4(),
        pin: Uuid::new_v4(),
        gate: Uuid::new_v4(),
        gate2: Uuid::new_v4(),
        package: Uuid::new_v4(),
        foot: Uuid::new_v4(),
        wrong_foot: Uuid::new_v4(),
        pad: Uuid::new_v4(),
        pad2: Uuid::new_v4(),
        symbol: Uuid::new_v4(),
        placed: Uuid::nil(),
        sheet: Uuid::new_v4(),
    };
    let other_package = Uuid::new_v4();
    let stack = Uuid::new_v4();
    for (kind, id, fields) in [
        (
            "units",
            f.unit,
            json!({"name":"Logic","pins":{f.pin.to_string():{"name":"Input","electrical_type":"Input"}}}),
        ),
        (
            "symbols",
            f.symbol,
            json!({"name":"Body","unit":f.unit,"drawings":[{"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}}],"pin_anchors":[{"pin":f.pin,"position":{"x":-2000000,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]}),
        ),
        ("packages", f.package, json!({"name":"Package"})),
        ("packages", other_package, json!({"name":"Other package"})),
        (
            "padstacks",
            stack,
            json!({"name":"Copper","aperture":{"circle":{"diameter_nm":1000000}}}),
        ),
        (
            "footprints",
            f.foot,
            json!({"name":"Land","package":f.package,"pads":{f.pad.to_string():{"uuid":f.pad,"name":"1","position":{"x":0,"y":0},"padstack":stack,"layer":0},f.pad2.to_string():{"uuid":f.pad2,"name":"2","position":{"x":2000000,"y":0},"padstack":stack,"layer":0}}}),
        ),
        (
            "footprints",
            f.wrong_foot,
            json!({"name":"Wrong land","package":other_package,"pads":{}}),
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
    let symbol = f.symbol;
    f.placed = write(
        &mut f,
        "datum.schematic.place_symbol",
        json!({"sheet":sheet,"library_symbol":symbol,"reference":"U1","value":"Original","x_nm":0,"y_nm":0}),
    );
    let (entity, gate, gate2, unit, symbol) = (f.entity, f.gate, f.gate2, f.unit, f.symbol);
    write(
        &mut f,
        "datum.library.create",
        json!({"kind":"entities","object_id":entity,"pool_path":"pool","fields":{"name":"Dual","prefix":"U","manufacturer":"","gates":{gate.to_string():{"uuid":gate,"name":"A","unit":unit,"symbol":symbol},gate2.to_string():{"uuid":gate2,"name":"B","unit":unit,"symbol":symbol}}}}),
    );
    for (part, default) in [(f.part, Some(f.foot)), (f.no_default, None)] {
        let mut raw = library::pool_part_payload(
            part,
            f.entity,
            f.package,
            "PN",
            "Designer",
            "Value",
            "Description",
            "reference.pdf",
            "Unknown",
        );
        raw["default_footprint"] = json!(default);
        raw["x_vendor"] = json!({"opaque":["keep",17],"basis":"not qualified"});
        raw["parametric"] = json!({"as_entered":"value"});
        raw.as_object_mut().unwrap().remove("packaging_options");
        let p = library::build_pool_library_write(
            &f.model,
            provenance(),
            Some("pool"),
            vec![PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "parts", part),
                object: raw,
            }],
        )
        .unwrap();
        commit_prepared(&mut f.model, &f.root, p).unwrap();
    }
    f
}
fn fields(f: &F, assign: bool) -> Value {
    json!({"part":f.part,"footprint":f.foot,"assign_default":assign,"mappings":{f.pad.to_string():{"gate":f.gate,"pin":f.pin},f.pad2.to_string():{"gate":f.gate2,"pin":f.pin}}})
}
fn prepare(f: &F, id: Uuid, fields: Value) -> Result<PreparedWrite, EngineError> {
    (find_native_write_verb("datum.library.create")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &f.model,
            project_root: &f.root,
        },
        provenance(),
        json!({"kind":"pin_pad_maps","object_id":id,"pool_path":"pool","fields":fields}),
    )
}
fn raw_part(f: &F) -> Value {
    library::pool_library_object_payload(&f.model, &format!("pool/parts/{}.json", f.part), f.part)
        .unwrap()
}
fn journal(f: &F) -> Vec<u8> {
    std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap()
}
fn bind(f: &mut F, part_ref: bool) -> Uuid {
    let mut op = native::component_instances::build_placed_symbol_component_instance_op(
        &f.model, f.placed, f.symbol, f.part,
    )
    .unwrap();
    let ci = if let Operation::CreateComponentInstance {
        component_instance_id,
        component_instance,
    } = &mut op
    {
        if !part_ref {
            component_instance
                .as_object_mut()
                .unwrap()
                .remove("part_ref");
        }
        *component_instance_id
    } else {
        unreachable!()
    };
    let batch = native::build_batch(&f.model, provenance(), vec![op]).unwrap();
    f.model.commit_journaled(&f.root, batch).unwrap();
    assert!(f.model.component_instances.contains_key(&ci));
    ci
}
#[test]
fn map_only_partial_and_gate_qualified_pin_identity_survive_history_and_cold() {
    let mut f = fixture();
    let id = Uuid::new_v4();
    let mut intent = fields(&f, false);
    intent["mappings"]
        .as_object_mut()
        .unwrap()
        .remove(&f.pad2.to_string());
    let before_part = raw_part(&f);
    let before = journal(&f);
    let p = prepare(&f, id, intent.clone()).unwrap();
    assert_eq!(journal(&f), before);
    assert!(!f.root.join(format!("pool/pin_pad_maps/{id}.json")).exists());
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    assert_eq!(raw_part(&f), before_part);
    let path = format!("pool/pin_pad_maps/{id}.json");
    let raw = library::pool_library_object_payload(&f.model, &path, id).unwrap();
    assert_eq!(raw["uuid"], json!(id));
    assert_eq!(raw["part"], json!(f.part));
    assert_eq!(raw["footprint"], json!(f.foot));
    assert_eq!(raw["mappings"], intent["mappings"]);
    assert!(raw.get("name").is_none());
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    assert!(!f.root.join(&path).exists());
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(
        library::pool_library_object_payload(&f.model, &path, id).unwrap(),
        raw
    );
    assert_eq!(raw_part(&f), before_part);
    // Both gates may independently use the same logical Unit pin.
    assert!(prepare(&f, Uuid::new_v4(), fields(&f, false)).is_ok());
    std::fs::remove_dir_all(f.root).unwrap();
}
#[test]
fn explicit_default_assignment_is_atomic_and_preserves_raw_part_siblings_and_identity() {
    let mut f = fixture();
    let id = Uuid::new_v4();
    let before = raw_part(&f);
    let board = std::fs::read(f.root.join("board/board.json")).unwrap();
    let source = std::fs::read(f.root.join("schematic/schematic.json")).unwrap();
    let p = prepare(&f, id, fields(&f, true)).unwrap();
    let stale = p.clone();
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    let mut expected = before.clone();
    expected["default_pin_pad_map"] = json!(id);
    assert_eq!(raw_part(&f), expected);
    let path = format!("pool/pin_pad_maps/{id}.json");
    let map = library::pool_library_object_payload(&f.model, &path, id).unwrap();
    assert_eq!(map["mappings"].as_object().unwrap().len(), 2);
    let after = journal(&f);
    assert!(commit_prepared(&mut f.model, &f.root, stale).is_err());
    assert_eq!(journal(&f), after);
    assert_eq!(raw_part(&f), expected);
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    assert_eq!(raw_part(&f), before);
    assert!(!f.root.join(&path).exists());
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(raw_part(&f), expected);
    assert_eq!(
        library::pool_library_object_payload(&f.model, &path, id).unwrap(),
        map
    );
    assert_eq!(
        std::fs::read(f.root.join("board/board.json")).unwrap(),
        board
    );
    assert_eq!(
        std::fs::read(f.root.join("schematic/schematic.json")).unwrap(),
        source
    );
    std::fs::remove_dir_all(f.root).unwrap();
}
#[test]
fn invalid_mapping_membership_aliases_pairs_and_default_eligibility_refuse_without_writes() {
    let f = fixture();
    let id = Uuid::new_v4();
    let valid = fields(&f, false);
    let mut cases = vec![];
    for (key, value) in [
        ("part", json!(f.foot)),
        ("part", json!(Uuid::new_v4())),
        ("footprint", json!(f.part)),
        ("footprint", json!(f.wrong_foot)),
        ("footprint", Value::Null),
        ("assign_default", json!("true")),
        ("name", json!("not a map field")),
    ] {
        let mut bad = valid.clone();
        bad[key] = value;
        cases.push(bad);
    }
    let mut empty = valid.clone();
    empty["mappings"] = json!({});
    cases.push(empty);
    for (field, value) in [
        ("gate", json!(Uuid::new_v4())),
        ("gate", json!(f.unit)),
        ("pin", json!(Uuid::new_v4())),
        ("pin", json!(f.part)),
        ("extra", json!(true)),
    ] {
        let mut bad = valid.clone();
        bad["mappings"][f.pad.to_string()][field] = value;
        cases.push(bad);
    }
    let mut wrong_pad = valid.clone();
    let value = wrong_pad["mappings"]
        .as_object_mut()
        .unwrap()
        .remove(&f.pad.to_string())
        .unwrap();
    wrong_pad["mappings"][Uuid::new_v4().to_string()] = value;
    cases.push(wrong_pad);
    let mut duplicate = valid.clone();
    duplicate["mappings"][f.pad2.to_string()] = duplicate["mappings"][f.pad.to_string()].clone();
    cases.push(duplicate);
    let mut alias = valid.clone();
    alias["mappings"][f.pad.simple().to_string()] = alias["mappings"][f.pad.to_string()].clone();
    cases.push(alias);
    let mut no_default = valid.clone();
    no_default["part"] = json!(f.no_default);
    no_default["assign_default"] = json!(true);
    cases.push(no_default);
    let before = journal(&f);
    let part = raw_part(&f);
    for bad in cases {
        assert!(prepare(&f, id, bad).is_err());
    }
    let request = LibraryDefinitionRequest {
        kind: "pin_pad_maps".into(),
        object_id: id,
        pool_path: "pool".into(),
        fields: valid.as_object().unwrap().clone(),
    };
    assert!(
        super::super::build_definition(
            &NativeWriteContext {
                model: &f.model,
                project_root: &f.root
            },
            provenance(),
            request,
            false
        )
        .is_err()
    );
    assert_eq!(journal(&f), before);
    assert_eq!(raw_part(&f), part);
    assert!(!f.root.join("pool/pin_pad_maps").exists());
    std::fs::remove_dir_all(f.root).unwrap();
}
#[test]
fn map_reference_guards_cover_part_entity_package_footprint_and_used_units() {
    let f = fixture();
    let p = prepare(&f, Uuid::new_v4(), fields(&f, false)).unwrap();
    let ids = HashSet::from([f.part, f.entity, f.package, f.foot, f.unit]);
    let actual: HashSet<_> = p
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
    assert_eq!(actual, ids);
    let mut current = f.model.clone();
    current.commit(p.batch.clone()).unwrap();
    let before = journal(&f);
    for id in ids {
        let mut changed = f.model.clone();
        changed.objects.get_mut(&id).unwrap().object_revision.0 += 1;
        assert!(
            changed
                .commit(p.batch.clone())
                .unwrap_err()
                .to_string()
                .contains("object revision mismatch")
        );
    }
    assert_eq!(journal(&f), before);
    std::fs::remove_dir_all(f.root).unwrap();
}
#[test]
fn resolved_and_dropped_persisted_ci_ownership_block_assignment_but_allow_map_only() {
    let mut f = fixture();
    let ci = bind(&mut f, true);
    let before = journal(&f);
    let part = raw_part(&f);
    assert!(prepare(&f, Uuid::new_v4(), fields(&f, true)).is_err());
    assert_eq!(journal(&f), before);
    assert_eq!(raw_part(&f), part);
    let batch = native::build_batch(
        &f.model,
        provenance(),
        vec![Operation::BumpObjectRevision { object_id: f.part }],
    )
    .unwrap();
    f.model.commit_journaled(&f.root, batch).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert!(!f.model.component_instances.contains_key(&ci));
    let shard = f
        .model
        .source_shards
        .iter()
        .find(|s| s.kind == SourceShardKind::ComponentInstance)
        .unwrap();
    let raw = f
        .model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap();
    assert_eq!(
        raw.pointer("/component_instance/part_ref/object_id"),
        Some(&json!(f.part))
    );
    let before = journal(&f);
    assert!(prepare(&f, Uuid::new_v4(), fields(&f, true)).is_err());
    assert_eq!(journal(&f), before);
    let id = Uuid::new_v4();
    let p = prepare(&f, id, fields(&f, false)).unwrap();
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    assert_eq!(raw_part(&f), part);
    assert!(f.root.join(format!("pool/pin_pad_maps/{id}.json")).exists());
    std::fs::remove_dir_all(f.root).unwrap();
}
#[test]
fn raw_part_role_binding_cannot_disappear_from_default_assignment_proof() {
    let mut f = fixture();
    let ci = bind(&mut f, false);
    assert!(
        f.model.component_instances[&ci]
            .library_bindings
            .values()
            .any(|b| b.binding_role == LibraryBindingRole::Part && b.target_object_id == f.part)
    );
    // A missing projection is never proof of absence: captured declarations remain.
    f.model.component_instances.clear();
    let before = journal(&f);
    assert!(prepare(&f, Uuid::new_v4(), fields(&f, true)).is_err());
    assert_eq!(journal(&f), before);
    assert!(prepare(&f, Uuid::new_v4(), fields(&f, false)).is_ok());
    std::fs::remove_dir_all(f.root).unwrap();
}
#[test]
fn missing_inventory_and_unsupported_shards_refuse_only_optional_default_assignment() {
    let mut f = fixture();
    let dir = f.root.join(".datum/component_instances");
    assert!(!dir.exists());
    std::fs::write(&dir, b"not a directory").unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert!(
        f.model
            .diagnostics
            .iter()
            .any(|d| d.code == "component_instance_inventory_unavailable")
    );
    let before = journal(&f);
    assert!(prepare(&f, Uuid::new_v4(), fields(&f, true)).is_err());
    assert_eq!(journal(&f), before);
    let id = Uuid::new_v4();
    let p = prepare(&f, id, fields(&f, false)).unwrap();
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    std::fs::remove_file(&dir).unwrap();
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(
        dir.join("unsupported.json"),
        br#"{"schema_version":2,"component_instance":{}}"#,
    )
    .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert!(
        f.model
            .diagnostics
            .iter()
            .any(|d| d.code == "invalid_component_instance_shard")
    );
    assert!(prepare(&f, Uuid::new_v4(), fields(&f, true)).is_err());
    assert!(prepare(&f, Uuid::new_v4(), fields(&f, false)).is_ok());
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn direct_placed_symbol_and_board_package_declarations_block_default_assignment() {
    use native::board_components::{BoardPackagePlacement, build_place_board_package};
    for board in [false, true] {
        let mut f = fixture();
        let p = if board {
            build_place_board_package(
                &f.model,
                provenance(),
                &BoardPackagePlacement {
                    package: crate::board::PlacedPackage {
                        uuid: Uuid::new_v4(),
                        part: f.part,
                        package: f.package,
                        reference: "P1".into(),
                        value: "Original".into(),
                        position: crate::ir::geometry::Point { x: 0, y: 0 },
                        rotation: 0,
                        layer: 0,
                        locked: false,
                    },
                    materialized: json!({}),
                },
            )
            .unwrap()
        } else {
            let captured = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
            let mut symbol =
                captured.schematic_source().unwrap().sheets[&f.sheet].symbols[&f.placed].clone();
            symbol.uuid = Uuid::new_v4();
            symbol.reference = "U2".into();
            symbol.part = Some(f.part);
            symbol.entity = Some(f.entity);
            symbol.gate = Some(f.gate);
            native::schematic_symbols::build_place_schematic_symbol(
                &f.model,
                provenance(),
                f.sheet,
                &symbol,
                None,
            )
            .unwrap()
        };
        commit_prepared(&mut f.model, &f.root, p).unwrap();
        // Source declarations remain authoritative even if a projection is absent.
        f.model.component_instances.clear();
        let before = journal(&f);
        assert!(prepare(&f, Uuid::new_v4(), fields(&f, true)).is_err());
        assert_eq!(journal(&f), before);
        assert!(prepare(&f, Uuid::new_v4(), fields(&f, false)).is_ok());
        std::fs::remove_dir_all(f.root).unwrap();
    }
}
