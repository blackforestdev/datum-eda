//! Shared native eight-pad materialization fixture for pinned library revision tests.
use super::*;
use crate::api::native_write::board_components::pad_materialization::ComponentPadMaterializationRequest;
use crate::ir::geometry::Point;
pub(super) fn eight_pad_fixture(
    name: &str,
) -> (
    PathBuf,
    DesignModel,
    Uuid,
    Uuid,
    Uuid,
    Uuid,
    Uuid,
    ComponentPadMaterializationRequest,
) {
    let (root, mut model, symbol, ci, part) = setup(name);
    let package = physical(
        &mut model,
        &root,
        symbol,
        ci,
        part,
        crate::ir::geometry::Point::zero(),
    );
    // setup() selects its additional legitimate Part, not the baseline Part
    // named by the baseline map. Create a separate explicit map for that Part.
    use crate::api::native_write::library::{
        self, PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec,
    };
    let map = Uuid::new_v4();
    let mut mapping = raw(
        &model,
        "a0000000-0000-4000-8000-000000000001".parse().unwrap(),
    );
    mapping["uuid"] = json!(map);
    mapping["part"] = json!(part);
    let p = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![Spec::Create {
            target: Target::new("pool", "pin_pad_maps", map),
            object: mapping,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let previous = raw(&model, ci)["component_instance"].clone();
    let mut next = previous.clone();
    next["object_revision"] = json!(model.objects[&ci].object_revision.0 + 1);
    next["library_bindings"][Uuid::new_v4().to_string()] = json!({"target_object_id":map,
        "pinned_object_revision":model.objects[&map].object_revision.0,"binding_role":"pin_pad_map"});
    let p = BatchComposer::compose(&model, provenance())
        .push_op(Operation::SetComponentInstance {
            component_instance_id: ci,
            previous_component_instance: previous,
            component_instance: next,
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    use crate::api::native_write::{
        board_components::pad_materialization::{
            ComponentPadMaterializationRequest, PadLandPatternSource,
            build_materialize_component_pad_sources,
        },
        board_layout::source_profile::build_adopt_board_source_profile,
    };
    let board = model
        .materialized_source_shard_value(crate::substrate::SourceShardKind::BoardRoot)
        .unwrap();
    let board_id = serde_json::from_value(board["uuid"].clone()).unwrap();
    let p = build_adopt_board_source_profile(&model, provenance(), board_id).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let board = model
        .materialized_source_shard_value(crate::substrate::SourceShardKind::BoardRoot)
        .unwrap();
    let request = ComponentPadMaterializationRequest {
        placement: serde_json::from_value(board["packages"][package.to_string()].clone()).unwrap(),
        land_pattern: PadLandPatternSource::Footprint(
            "60000000-0000-4000-8000-000000000001".parse().unwrap(),
        ),
        component_instance: Some(ci),
        pin_pad_map: Some(map),
        correspondences: vec![],
        adopt_existing: false,
    };
    let p = build_materialize_component_pad_sources(&model, provenance(), &request).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let board = model
        .materialized_source_shard_value(crate::substrate::SourceShardKind::BoardRoot)
        .unwrap();
    assert_eq!(
        board["component_pads"][package.to_string()]
            .as_array()
            .unwrap()
            .len(),
        8
    );
    (root, model, symbol, ci, part, package, map, request)
}

/// A separate current-map component ready for its first materialization, with
/// another eight-pad component already owning a Complete connected Net.
pub(super) struct CurrentMaterializationFixture {
    pub root: PathBuf,
    pub model: DesignModel,
    pub symbol: Uuid,
    pub ci: Uuid,
    pub old_ci: crate::substrate::ComponentInstance,
    pub old_net: Uuid,
    pub old_members: BTreeSet<ElectricalOccurrence>,
    pub package: Uuid,
    pub old_pads: Value,
    pub map: Uuid,
    pub revised: Value,
    pub new_symbol: Uuid,
    pub new_ci: Uuid,
    pub new_package: Uuid,
    pub selected: Uuid,
    pub request: ComponentPadMaterializationRequest,
}
pub(super) fn current_materialization_fixture(name: &str) -> CurrentMaterializationFixture {
    use crate::api::native_write::board_components::pad_materialization::PadLandPatternSource;
    use crate::api::native_write::schematic_library::part_placement::PartGateSelection;
    let (root, mut model, symbol, ci, part, package, map, mut request) = eight_pad_fixture(name);
    let (old_net, selected, _) =
        super::map_tests::connected(&mut model, &root, symbol, package, ci, map);
    let before = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let old_members = before.global_net(before.revision(), old_net).unwrap().owned;
    let old_ci = model.component_instances[&ci].clone();
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let old_pads = board["component_pads"][package.to_string()].clone();
    let original = raw(&model, map);
    let mut mappings = original["mappings"].as_object().unwrap().clone();
    let other = mappings
        .keys()
        .find(|id| id.parse::<Uuid>().unwrap() != selected)
        .unwrap()
        .clone();
    let old = mappings[&selected.to_string()].clone();
    let replacement = mappings[&other].clone();
    mappings.insert(selected.to_string(), replacement);
    mappings.insert(other, old);
    let p=(find_native_write_verb("datum.library.edit").unwrap().build)(
        &NativeWriteContext { model:&model,project_root:&root },provenance(),
        json!({"kind":"pin_pad_maps","object_id":map,"pool_path":"pool","fields":{"mappings":mappings}}),
    ).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let revised = raw(&model, map);
    assert_ne!(revised["mappings"], original["mappings"]);
    // A separate ordinary placement chooses the same Part/gate explicitly.
    // No old binding is retargeted to make the revised map usable.
    let source = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let (sheet, old_symbol) = source
        .schematic_source()
        .unwrap()
        .sheets
        .values()
        .find_map(|s| s.symbols.get(&symbol).map(|p| (s.uuid, p.clone())))
        .unwrap();
    let entity = raw(&model, old_symbol.entity.unwrap());
    let library_symbol: Uuid = serde_json::from_value(
        entity["gates"][old_symbol.gate.unwrap().to_string()]["symbol"].clone(),
    )
    .unwrap();
    let intent =
        PartGateSelection::capture(&model, part, old_symbol.gate.unwrap(), library_symbol).unwrap();
    let p=(find_native_write_verb("datum.schematic.place_symbol").unwrap().build)(
        &NativeWriteContext { model:&model,project_root:&root },provenance(),
        json!({"sheet":sheet,"library_symbol":library_symbol,"explicit_part":intent,"reference":"U2","value":"New current map","x_nm":45_000_000,"y_nm":20_000_000}),
    ).unwrap();
    let new_symbol = p.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let new_ci = model
        .component_instances
        .values()
        .find(|c| c.placed_symbol_refs.contains(&new_symbol))
        .unwrap()
        .id;
    let previous = raw(&model, new_ci)["component_instance"].clone();
    let mut updated = previous.clone();
    updated["object_revision"] = json!(model.objects[&new_ci].object_revision.0 + 1);
    updated["library_bindings"][Uuid::new_v4().to_string()] =
        json!({"target_object_id":map,"pinned_object_revision":1,"binding_role":"pin_pad_map"});
    let p = BatchComposer::compose(&model, provenance())
        .push_op(Operation::SetComponentInstance {
            component_instance_id: new_ci,
            previous_component_instance: previous,
            component_instance: updated,
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let new_package = physical(
        &mut model,
        &root,
        new_symbol,
        new_ci,
        part,
        Point::new(45_000_000, 0),
    );
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    request.placement =
        serde_json::from_value(board["packages"][new_package.to_string()].clone()).unwrap();
    request.component_instance = Some(new_ci);
    assert!(matches!(
        request.land_pattern,
        PadLandPatternSource::Footprint(_)
    ));
    CurrentMaterializationFixture {
        root,
        model,
        symbol,
        ci,
        old_ci,
        old_net,
        old_members,
        package,
        old_pads,
        map,
        revised,
        new_symbol,
        new_ci,
        new_package,
        selected,
        request,
    }
}
