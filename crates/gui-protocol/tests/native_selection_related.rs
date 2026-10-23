//! Real canonical component bindings through shared query, view and transport.
#[path = "support/native_selection_fixture.rs"]
mod fixture;
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Identity, CompoundSelection,
    DatumSelectionContext, SelectionSubject as Subject, SelectionTarget,
    selection_resolution::NativeSelectionResolution as Native,
    selection_view::{InspectionDetailKind, InspectionValue, build_selection_view},
};
use eda_engine::{
    api::native_write::{self, board_components, component_instances, library, schematic_symbols},
    board::PlacedPackage,
    ir::geometry::Point,
    schematic::{HiddenPowerBehavior, PlacedSymbol, SymbolDisplayMode},
    substrate::{ElectricalOccurrence, ElectricalQueryFailure, ModelRevision},
};
use fixture::{Fixture, id, provenance};
use std::collections::BTreeSet;

fn identity(class: Class, n: u128) -> Identity {
    Identity {
        class,
        id: id(n),
        instance_path: vec![],
    }
}
fn setup() -> Fixture {
    let mut f = Fixture::new("native-component-relations");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test-harness/testdata/library/native_authored_baseline_v1/pool");
    let mut writes = Vec::new();
    for kind in std::fs::read_dir(root).unwrap() {
        let kind = kind.unwrap();
        let name = kind.file_name().to_string_lossy().into_owned();
        for file in std::fs::read_dir(kind.path()).unwrap() {
            let value: serde_json::Value =
                serde_json::from_slice(&std::fs::read(file.unwrap().path()).unwrap()).unwrap();
            writes.push(library::PoolLibraryOperationSpec::Create {
                target: library::PoolLibraryObjectTarget::new(
                    "pool",
                    &name,
                    value["uuid"].as_str().unwrap().parse().unwrap(),
                ),
                object: value,
            });
        }
    }
    let write =
        library::build_pool_library_write(&f.model, provenance(), Some("pool"), writes).unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    let pool = |text: &str| text.parse().unwrap();
    let part = pool("90000000-0000-4000-8000-000000000001");
    let lib = "30000000-0000-4000-8000-000000000001";
    let symbol = PlacedSymbol {
        uuid: id(1),
        part: Some(part),
        entity: Some(pool("40000000-0000-4000-8000-000000000001")),
        gate: Some(pool("40000000-0000-4000-8000-000000000101")),
        lib_id: Some(lib.into()),
        reference: "UXP".into(),
        value: "native binding".into(),
        fields: vec![],
        pins: native_write::schematic_library::materialize_pool_symbol_pins(&f.model, Some(lib))
            .unwrap(),
        position: Point::zero(),
        rotation: 0,
        mirrored: false,
        unit_selection: None,
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: vec![],
        hidden_power_behavior: HiddenPowerBehavior::SourceDefinedImplicit,
    };
    let write = schematic_symbols::build_place_schematic_symbol(
        &f.model,
        provenance(),
        f.sheet,
        &symbol,
        None,
    )
    .unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    let package = PlacedPackage {
        uuid: id(2),
        part,
        package: pool("50000000-0000-4000-8000-000000000001"),
        reference: "UXP".into(),
        value: "native binding".into(),
        position: Point::zero(),
        rotation: 0,
        layer: 1,
        locked: false,
    };
    let write = board_components::build_place_board_package(
        &f.model,
        provenance(),
        &board_components::BoardPackagePlacement {
            package,
            materialized: serde_json::json!({}),
        },
    )
    .unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    bind(&mut f, id(3));
    f.model = f.reopen();
    f
}
fn bind(f: &mut Fixture, ci: uuid::Uuid) {
    let spec = component_instances::ComponentInstanceSpec {
        part_id: Some("90000000-0000-4000-8000-000000000001".parse().unwrap()),
        symbol_ids: vec![id(1)],
        package_id: id(2),
        symbol_roles: vec![],
        package_roles: vec![],
    };
    let (write, _) =
        component_instances::build_bind_component_instance(&f.model, provenance(), Some(ci), &spec)
            .unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
}
fn clear_binding(f: &mut Fixture) {
    let (write, _) =
        component_instances::build_delete_component_instance(&f.model, provenance(), id(3))
            .unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    f.model = f.reopen();
}

#[test]
fn authored_binding_is_bidirectional_related_only_and_revision_bound() {
    let f = setup();
    let before = f.source_bytes();
    let native = Native::capture(&f.model).unwrap();
    let symbol = identity(Class::Symbol, 1);
    let footprint = identity(Class::Footprint, 2);
    for (source, target) in [(&symbol, &footprint), (&footprint, &symbol)] {
        let projection = native.resolve(&Subject::Object(source.clone())).unwrap();
        assert_eq!(projection.members, BTreeSet::from([source.clone()]));
        assert_eq!(projection.related, BTreeSet::from([target.clone()]));
        let relation = projection.component_context.relations.first().unwrap();
        assert_eq!(relation.component_instance, id(3));
        assert_eq!(relation.counterpart.source_id, target.id);
        assert_eq!(relation.counterpart_reference.as_deref(), Some("UXP"));
    }
    let both = Subject::Compound(
        CompoundSelection::new([symbol.clone(), footprint.clone()], Some(symbol.clone())).unwrap(),
    );
    let projection = native.resolve(&both).unwrap();
    assert_eq!(projection.members.len(), 2);
    assert!(
        projection.related.is_empty(),
        "already selected counterpart is not double-counted"
    );
    assert!(matches!(
        native.snapshot().component_selection_context(
            &ModelRevision("stale".into()),
            &BTreeSet::from([ElectricalOccurrence {
                class: "symbols".into(),
                source_id: id(1),
                instance_path: vec![]
            }])
        ),
        Err(ElectricalQueryFailure::StaleRevision)
    ));
    Fixture::assert_bytes(&before);
}

#[test]
fn ambiguous_or_absent_authored_bindings_do_not_infer_a_counterpart() {
    let mut f = setup();
    bind(&mut f, id(4));
    f.model = f.reopen();
    let native = Native::capture(&f.model).unwrap();
    let projection = native
        .resolve(&Subject::Object(identity(Class::Symbol, 1)))
        .unwrap();
    assert_eq!(projection.members.len(), 1);
    assert!(projection.related.is_empty());
    assert!(
        projection
            .component_context
            .unavailable
            .values()
            .any(|v| v.contains("ambiguous"))
    );
    let (write, _) =
        component_instances::build_delete_component_instance(&f.model, provenance(), id(4))
            .unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    f.model = f.reopen();
    clear_binding(&mut f);
    assert!(f.model.component_instances.is_empty());
    let before = f.source_bytes();
    let projection = Native::capture(&f.model)
        .unwrap()
        .resolve(&Subject::Object(identity(Class::Symbol, 1)))
        .unwrap();
    assert!(
        projection.related.is_empty(),
        "matching names/Part cannot replace authored binding"
    );
    assert!(!projection.component_context.unavailable.is_empty());
    Fixture::assert_bytes(&before);
}

#[test]
fn fresh_revision_drops_removed_binding_without_rewriting_prior_snapshot() {
    let mut f = setup();
    let prior = Native::capture(&f.model).unwrap();
    let subject = Subject::Object(identity(Class::Symbol, 1));
    assert_eq!(prior.resolve(&subject).unwrap().related.len(), 1);
    clear_binding(&mut f);
    let before = f.source_bytes();
    let current = Native::capture(&f.model).unwrap();
    assert!(current.resolve(&subject).unwrap().related.is_empty());
    assert_eq!(prior.resolve(&subject).unwrap().related.len(), 1);
    Fixture::assert_bytes(&before);
}

#[test]
fn endpoint_revision_change_cannot_reuse_stale_component_references() {
    let mut f = setup();
    let write = board_components::build_edit_board_package(
        &f.model,
        provenance(),
        id(2),
        board_components::BoardPackageEdit::Rotation { rotation: 90 },
    )
    .unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    f.model = f.reopen();
    let before = f.source_bytes();
    let projection = Native::capture(&f.model)
        .unwrap()
        .resolve(&Subject::Object(identity(Class::Symbol, 1)))
        .unwrap();
    assert_eq!(projection.members.len(), 1);
    assert!(
        projection.related.is_empty(),
        "source existence cannot validate an old endpoint revision"
    );
    assert!(!projection.component_context.unavailable.is_empty());
    Fixture::assert_bytes(&before);
}

#[test]
fn source_only_binding_cannot_choose_root_over_repeated_sheet_occurrences() {
    use eda_engine::api::native_write::schematic_sheets;
    use eda_engine::schematic::{SheetDefinition, SheetInstance};
    let mut f = setup();
    let definition = SheetDefinition {
        uuid: id(700),
        root_sheet: f.sheet,
        name: "repeated".into(),
    };
    let write = schematic_sheets::build_create_schematic_definition(
        &f.model,
        provenance(),
        f.schematic,
        definition.uuid,
        "definitions/repeated.json",
        serde_json::to_value(definition).unwrap(),
    )
    .unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    for n in [701, 702] {
        let instance = SheetInstance {
            uuid: id(n),
            definition: id(700),
            parent_sheet: None,
            name: "repeated".into(),
            position: Point::zero(),
            ports: vec![],
        };
        let write = schematic_sheets::build_create_schematic_sheet_instance(
            &f.model,
            provenance(),
            f.schematic,
            instance.uuid,
            serde_json::to_value(instance).unwrap(),
        )
        .unwrap();
        native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    }
    f.model = f.reopen();
    let native = Native::capture(&f.model).unwrap();
    assert!(
        !native
            .authored_identities()
            .contains(&identity(Class::Symbol, 1)),
        "definition-only Symbol has no fabricated root occurrence"
    );
    for origin in [
        identity(Class::Footprint, 2),
        Identity {
            class: Class::Symbol,
            id: id(1),
            instance_path: vec![id(701)],
        },
        Identity {
            class: Class::Symbol,
            id: id(1),
            instance_path: vec![id(702)],
        },
    ] {
        let projection = native.resolve(&Subject::Object(origin)).unwrap();
        assert_eq!(projection.members.len(), 1);
        assert!(projection.related.is_empty());
        assert!(!projection.component_context.unavailable.is_empty());
    }
}

#[test]
#[ignore = "owned local frozen-fixture diagnosis; not regular regression or native proof"]
fn frozen_fixture_current_binding_diagnosis() {
    use eda_engine::substrate::{Operation, ProjectResolver, SourceShardKind};
    let root = std::path::PathBuf::from(
        std::env::var("DATUM_CROSSPROBE_DIAGNOSTIC_ROOT").expect("explicit owned clone required"),
    );
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let ci: uuid::Uuid = "78000000-0000-4000-8000-000000000003".parse().unwrap();
    let symbol: uuid::Uuid = "78000000-0000-4000-8000-000000000001".parse().unwrap();
    let package: uuid::Uuid = "78000000-0000-4000-8000-000000000002".parse().unwrap();
    let mut stages = vec![];
    let inspect = |model: &eda_engine::substrate::DesignModel, label: &str| {
        let object = &model.objects[&ci];
        let shard = model
            .source_shards
            .iter()
            .find(|s| s.shard_id == object.source_shard_id)
            .unwrap();
        let source = model
            .materialized_source_shard_value_by_relative_path(&shard.relative_path)
            .unwrap();
        let native = Native::capture(model).unwrap();
        let projection = native
            .resolve(&Subject::Object(Identity {
                class: Class::Symbol,
                id: symbol,
                instance_path: vec![],
            }))
            .unwrap();
        let board = model
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .unwrap();
        let relationships: Vec<_> = model
            .electrical_identities
            .values()
            .filter(|r| {
                matches!(
                    r.identity,
                    eda_engine::substrate::ElectricalIdentity::NetRelationship { .. }
                )
            })
            .collect();
        let net: uuid::Uuid = "2e838d28-abb8-4133-a2ad-413578c5d913".parse().unwrap();
        let net_query = native
            .snapshot()
            .global_net(native.snapshot().revision(), net)
            .map(|m| m.owned)
            .map_err(|e| format!("{e:?}"));
        serde_json::json!({"label":label,"revision":model.model_revision,"ci_source":source,
            "relationships":relationships,"global_net_query":net_query,
            "correspondence_status":format!("{:?}",eda_engine::substrate::net_correspondence_status(model,net)),
            "ci_object_revision":object.object_revision,"package_object_revision":model.objects[&package].object_revision,
            "related":projection.related,"unavailable":projection.component_context.unavailable.values().collect::<Vec<_>>(),
            "pad_binding_ci_pins":board["component_pads"][package.to_string()].as_array().unwrap().iter().map(|p|p["source_binding"]["component_instance"].clone()).collect::<Vec<_>>()})
    };
    if std::env::var_os("DATUM_CROSSPROBE_DIAGNOSTIC_READ_ONLY").is_some() {
        println!(
            "CROSSPROBE_CURRENT_BINDING_READ_ONLY {}",
            inspect(&model, "current_without_mutation")
        );
        return;
    }
    stages.push(inspect(&model, "before"));
    let object = &model.objects[&ci];
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == object.source_shard_id)
        .unwrap();
    let source = model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap();
    let previous = source["component_instance"].clone();
    let mut next = previous.clone();
    next["object_revision"] = serde_json::json!(next["object_revision"].as_u64().unwrap() + 1);
    next["placed_package_refs"][0]["object_revision"] =
        serde_json::json!(model.objects[&package].object_revision);
    next["placed_symbol_refs"][0]["object_revision"] =
        serde_json::json!(model.objects[&symbol].object_revision);
    let write = native_write::BatchComposer::compose(&model, provenance())
        .push_op(Operation::SetComponentInstance {
            component_instance_id: ci,
            previous_component_instance: previous,
            component_instance: next,
        })
        .finish()
        .unwrap();
    native_write::commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    stages.push(inspect(&model, "after_current_endpoint_rebind"));
    println!(
        "CROSSPROBE_CURRENT_BINDING_DIAGNOSIS {}",
        serde_json::to_string(&stages).unwrap()
    );
}

#[test]
fn shared_inspection_explains_relation_without_bypassing_transport_detail_cap() {
    let f = setup();
    let native = Native::capture(&f.model).unwrap();
    let mut scene = datum_gui_protocol::load_fixture_workspace_state().scene;
    scene.project_uuid = f.model.project.project_id.to_string();
    scene.source_revision = f.model.model_revision.0.clone();
    let mut view = build_selection_view(
        &f.model,
        &native,
        Subject::Object(identity(Class::Symbol, 1)),
        &scene,
        None,
        |_| true,
    )
    .unwrap();
    let InspectionValue::Common(value) = view.field("Related component") else {
        panic!("binding caption required")
    };
    assert!(value.as_str().unwrap().contains("Footprint UXP"));
    assert_eq!(
        view.inspection
            .iter()
            .find(|s| s.scope == view.scope)
            .unwrap()
            .field_details["Component bindings"],
        InspectionDetailKind::Members
    );
    let retained = view.field("Component bindings");
    for n in 10..267 {
        view.related.insert(identity(Class::Footprint, n));
    }
    let context = DatumSelectionContext::from_selection(&SelectionTarget::Native(Box::new(view)))
        .details
        .unwrap();
    assert_eq!(context.total, 1);
    assert_eq!(context.members.unwrap().len(), 1);
    assert!(
        !context.truncated,
        "selected member transport is still complete"
    );
    assert!(
        context
            .omitted_inspection_fields
            .contains(&"Component bindings".into())
    );
    assert!(!context.inspection.contains_key("Component bindings"));
    assert!(context.inspection.contains_key("Related component"));
    assert!(matches!(retained, InspectionValue::Common(_)));
}

#[test]
fn enumerated_binding_detail_is_bounded_even_when_selected_and_related_sets_are_small() {
    let f = setup();
    let native = Native::capture(&f.model).unwrap();
    let scene = datum_gui_protocol::load_fixture_workspace_state().scene;
    let mut view = build_selection_view(
        &f.model,
        &native,
        Subject::Object(identity(Class::Symbol, 1)),
        &scene,
        None,
        |_| true,
    )
    .unwrap();
    // Defensive legacy/direct-field replacement: the public transport must
    // inspect actual identity detail rather than assume its size from selection.
    let relations: Vec<_> = (10..139)
        .map(|n| {
            serde_json::json!({
                "source":{"class":"symbols","source_id":id(n)},
                "counterpart":{"class":"packages","source_id":id(1000+n)},
                "component_instance":id(3)
            })
        })
        .collect();
    let full = serde_json::json!({"relations":relations,"unavailable":[]});
    view.inspection
        .iter_mut()
        .find(|s| s.scope == view.scope)
        .unwrap()
        .fields
        .insert(
            "Component bindings".into(),
            InspectionValue::Common(full.clone()),
        );
    assert_eq!(view.members.len(), 1);
    assert_eq!(view.related.len(), 1);
    let retained = view.clone();
    let context = DatumSelectionContext::from_selection(&SelectionTarget::Native(Box::new(view)))
        .details
        .unwrap();
    assert!(!context.inspection.contains_key("Component bindings"));
    assert!(
        context
            .omitted_inspection_fields
            .contains(&"Component bindings".into())
    );
    assert_eq!(
        retained.field("Component bindings"),
        InspectionValue::Common(full)
    );
    assert_eq!(context.total, 1);
    assert!(!context.truncated);
}
