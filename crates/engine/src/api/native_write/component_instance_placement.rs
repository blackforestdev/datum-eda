//! Pure component placement birth image shared by live admission and receipts.
use super::ids;
use crate::substrate::Operation;
use uuid::Uuid;

/// Pure expected birth image. Actual placement must obtain its revision from
/// current model admission; a receipt uses this only to compare cached output.
pub(in crate::api::native_write) fn placed_symbol_component_instance_birth(
    project: Uuid,
    placed_symbol_id: Uuid,
    pool_symbol_id: Uuid,
    part_id: Uuid,
    revision: crate::substrate::ObjectRevision,
) -> Operation {
    let component_instance_id = ids::derive_object_id(
        &project,
        "component-instance",
        &[
            "schematic".into(),
            pool_symbol_id.to_string(),
            placed_symbol_id.to_string(),
        ],
    );
    let binding = ids::derive_object_id(
        &project,
        "library-binding",
        &[
            component_instance_id.to_string(),
            "part".into(),
            part_id.to_string(),
        ],
    );
    Operation::CreateComponentInstance {
        component_instance_id,
        component_instance: serde_json::json!({
            "uuid": component_instance_id,
            "object_revision": 0,
            "part_ref": {"object_id": part_id, "object_revision": revision},
            "library_bindings": {binding.to_string(): {
                "target_object_id": part_id,
                "pinned_object_revision": revision,
                "binding_role": "part"
            }},
            "placed_symbol_refs": [{"object_id": placed_symbol_id,"object_revision":0}],
            "placed_package_refs": [],
            "placed_symbol_roles": {placed_symbol_id.to_string(): {"role":"primary"}},
            "placed_package_roles": {}
        }),
    }
}
