//! Shared native eight-pad materialization fixture for pinned library revision tests.
use super::*;
use crate::api::native_write::board_components::pad_materialization::ComponentPadMaterializationRequest;
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
