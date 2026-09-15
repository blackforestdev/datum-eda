//! Native footprint builders for the native write facade.
//!
//! Family B (footprint slice): the create-footprint payload template and the
//! IPC-7351B generated-footprint operation composition live here, shared by
//! the direct CLI commands in
//! `crates/cli/src/command_project_library_footprint.rs` and the proposal
//! flows in `crates/cli/src/command_project_library_footprint_proposals.rs`
//! (both feed the same [`super::library::PoolLibraryOperationSpec`]s into
//! [`super::library::build_pool_library_write`], committing directly or
//! drafting a proposal from the uncommitted batch).
//!
//! Payload shapes are byte-for-byte the CLI's historical behavior — pool
//! footprint shards must not drift.

use uuid::Uuid;

use crate::error::EngineError;
use crate::pool::GeneratedIpcFootprint;

use super::library::{PoolLibraryObjectTarget, PoolLibraryOperationSpec};

/// Historical create-payload template for a native pool footprint.
pub fn pool_footprint_payload(
    footprint_id: Uuid,
    package_id: Uuid,
    name: &str,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "uuid": footprint_id,
        "name": name,
        "package": package_id,
        "pads": {},
        "courtyard": {"vertices": [], "closed": true},
        "silkscreen": [],
        "fab": [],
        "assembly": [],
        "mechanical": [],
        "models_3d": [],
        "standards_basis": null,
        "process_aperture_policy": null,
        "tags": []
    })
}

/// The create specs for one generated IPC footprint: the generated padstack
/// followed by the generated footprint, both serialized with the historical
/// `schema_version: 1` marker inserted.
pub fn generated_ipc_footprint_specs(
    pool_path: &str,
    generated: GeneratedIpcFootprint,
) -> Result<Vec<PoolLibraryOperationSpec>, EngineError> {
    let padstack_id = generated.padstack.uuid;
    let footprint_id = generated.footprint.uuid;
    Ok(vec![
        PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new(pool_path, "padstacks", padstack_id),
            object: pool_object_value_with_schema(serde_json::to_value(generated.padstack)?)?,
        },
        PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new(pool_path, "footprints", footprint_id),
            object: pool_object_value_with_schema(serde_json::to_value(generated.footprint)?)?,
        },
    ])
}

fn pool_object_value_with_schema(
    mut object: serde_json::Value,
) -> Result<serde_json::Value, EngineError> {
    object
        .as_object_mut()
        .ok_or_else(|| {
            EngineError::Validation(
                "generated pool object must serialize as a JSON object".to_string(),
            )
        })?
        .insert("schema_version".to_string(), serde_json::json!(1));
    Ok(object)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::{
        IpcDensityLevel, IpcSourceDimensions, IpcTwoTerminalChipSpec,
        generate_ipc7351b_two_terminal_chip,
    };

    #[test]
    fn footprint_payload_matches_historical_shape() {
        let footprint_id = Uuid::new_v4();
        let package_id = Uuid::new_v4();
        assert_eq!(
            pool_footprint_payload(footprint_id, package_id, "R0603"),
            serde_json::json!({
                "schema_version": 1,
                "uuid": footprint_id,
                "name": "R0603",
                "package": package_id,
                "pads": {},
                "courtyard": {"vertices": [], "closed": true},
                "silkscreen": [],
                "fab": [],
                "assembly": [],
                "mechanical": [],
                "models_3d": [],
                "standards_basis": null,
                "process_aperture_policy": null,
                "tags": []
            })
        );
    }

    #[test]
    fn generated_ipc_specs_serialize_padstack_then_footprint_with_schema_marker() {
        let spec = IpcTwoTerminalChipSpec {
            footprint_uuid: Uuid::new_v4(),
            package_uuid: Uuid::new_v4(),
            padstack_uuid: Uuid::new_v4(),
            pad_a_uuid: Uuid::new_v4(),
            pad_b_uuid: Uuid::new_v4(),
            name: None,
            metric_code: "1608".to_string(),
            dimensions: IpcSourceDimensions {
                body_length_nm: 1_600_000,
                body_width_nm: 800_000,
                terminal_length_nm: 300_000,
                terminal_width_nm: 800_000,
            },
            density_level: IpcDensityLevel::Nominal,
            mask_expansion_nm: 0,
            paste_reduction_nm: 0,
        };
        let footprint_uuid = spec.footprint_uuid;
        let padstack_uuid = spec.padstack_uuid;
        let generated =
            generate_ipc7351b_two_terminal_chip(spec).expect("IPC generation should succeed");
        // Historical CLI oracle: schema_version inserted into the serde value.
        let mut expected_padstack =
            serde_json::to_value(&generated.padstack).expect("padstack should serialize");
        expected_padstack["schema_version"] = serde_json::json!(1);
        let mut expected_footprint =
            serde_json::to_value(&generated.footprint).expect("footprint should serialize");
        expected_footprint["schema_version"] = serde_json::json!(1);

        let specs = generated_ipc_footprint_specs("pool", generated)
            .expect("spec composition should succeed");

        assert_eq!(
            specs,
            vec![
                PoolLibraryOperationSpec::Create {
                    target: PoolLibraryObjectTarget::new("pool", "padstacks", padstack_uuid),
                    object: expected_padstack,
                },
                PoolLibraryOperationSpec::Create {
                    target: PoolLibraryObjectTarget::new("pool", "footprints", footprint_uuid),
                    object: expected_footprint,
                },
            ]
        );
    }
}
