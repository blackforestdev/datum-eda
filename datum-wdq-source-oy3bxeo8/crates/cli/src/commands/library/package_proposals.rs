use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::substrate::ProjectResolver;
use uuid::Uuid;

use super::library::validate_project_local_pool_path;
use super::proposals::propose_create_pool_library_object_value;
use crate::NativeProjectProposalCreateView;

#[allow(clippy::too_many_arguments)]
pub(crate) fn propose_create_native_project_pool_package(
    root: &Path,
    pool_path: &str,
    package_id: Uuid,
    name: String,
    pad_id: Option<Uuid>,
    padstack_id: Option<Uuid>,
    pad_name: String,
    x_nm: i64,
    y_nm: i64,
    layer: i32,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    validate_project_local_pool_path(pool_path)?;
    let legacy_pad = match (pad_id, padstack_id) {
        (Some(pad_id), Some(padstack_id)) => Some((pad_id, padstack_id)),
        (Some(_), None) | (None, Some(_)) => bail!(
            "legacy package pad compatibility requires both --pad and --padstack; prefer create-pool-footprint for land-pattern pads"
        ),
        (None, None) => None,
    };
    if legacy_pad.is_some() && layer <= 0 {
        bail!("package pad layer must be positive");
    }
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    if let Some((_, padstack_id)) = legacy_pad
        && model
            .objects
            .get(&padstack_id)
            .filter(|object| object.domain == "pool" && object.kind == "padstacks")
            .is_none()
    {
        bail!("missing pool padstack {padstack_id} for package {package_id}");
    }
    let pads = legacy_pad
        .map(|(pad_id, padstack_id)| {
            serde_json::json!({
                pad_id.to_string(): {
                    "uuid": pad_id,
                    "name": pad_name,
                    "position": {"x": x_nm, "y": y_nm},
                    "padstack": padstack_id,
                    "layer": layer
                }
            })
        })
        .unwrap_or_else(|| serde_json::json!({}));
    let object = serde_json::json!({
        "schema_version": 1,
        "uuid": package_id,
        "name": name,
        "pads": pads,
        "courtyard": {"vertices": [], "closed": true},
        "silkscreen": [],
        "models_3d": [],
        "body_height_nm": null,
        "body_height_mounted_nm": null,
        "tags": []
    });
    propose_create_pool_library_object_value(
        root,
        pool_path,
        "packages",
        package_id,
        object,
        proposal_id,
        rationale,
        "create_pool_package_proposal",
        "Create native pool package",
    )
}
