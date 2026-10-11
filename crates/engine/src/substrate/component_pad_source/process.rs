//! Frozen PM061 process inputs and exact placed projection. No live read re-resolution.
pub mod inspection;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::{ComponentPadSource, invalid, source_value};

use crate::board::quadratic3::ExactPointNm;
use crate::board::{PlacedPackage, Stackup};
use crate::pool::process::{
    EffectivePadstackProcess, PinnedProcessProfile, ProcessBasisSelector, ProcessMembership,
    ProcessProfile, ProcessSourceCapability, resolve_padstack_process,
};
use crate::substrate::{
    DesignModel, EngineError, ObjectRevision, SourceShardKind,
    process_operations::capability_from_board,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenProfilePin {
    pub project_id: Uuid,
    pub profile: ProcessProfile,
    pub object_revision: ObjectRevision,
    pub value_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenProcessSnapshot {
    pub version: u32,
    pub effective: EffectivePadstackProcess,
    pub membership: ProcessMembership,
    pub capability: ProcessSourceCapability,
    pub stackup: Stackup,
    pub stackup_value_hash: String,
    pub profile: Option<FrozenProfilePin>,
    pub raw_padstack: Value,
    pub raw_footprint: Value,
    pub transform_basis: PlacedPackage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacedProcessGeometry {
    pub pad_id: Uuid,
    /// Captured physical face associations; preserved raw inputs attach their adopted board capability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_capability: Option<ProcessSourceCapability>,
    pub center_nm: ExactPointNm,
    pub rotation: i32,
    pub reflected: bool,
    pub effective: EffectivePadstackProcess,
    pub membership: ProcessMembership,
}

pub(crate) fn validate_update_policy(
    _model: &DesignModel,
    batch: &crate::substrate::OperationBatch,
    kind: crate::substrate::TransactionKind,
    accepted_proposal: bool,
) -> Result<(), EngineError> {
    use crate::substrate::Operation;
    if kind != crate::substrate::TransactionKind::Normal || accepted_proposal {
        return Ok(());
    }
    for op in &batch.operations {
        let Operation::SetBoardPackagePackage {
            package_id,
            previous_materialized,
            materialized,
            ..
        } = op
        else {
            continue;
        };
        let pads = |value: &Value| -> Result<Vec<ComponentPadSource>, EngineError> {
            Ok(serde_json::from_value(
                value
                    .get("component_pads")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!([])),
            )?)
        };
        let old = pads(previous_materialized)?;
        let new = pads(materialized)?;
        let summary = |pads: &[ComponentPadSource]| -> Result<Value, EngineError> {
            let mut records = std::collections::BTreeMap::new();
            for pad in pads {
                if let Some(snapshot) = pad
                    .source_binding
                    .as_ref()
                    .and_then(|b| b.process_snapshot.as_ref())
                {
                    records.insert(pad.uuid, serde_json::json!({"version": snapshot.version,
                        "effective": snapshot.effective, "membership": snapshot.membership,
                        "profile": snapshot.profile, "stackup_value_hash": snapshot.stackup_value_hash}));
                }
            }
            Ok(serde_json::to_value(records)?)
        };
        if summary(&old)? != summary(&new)? {
            return Err(invalid(format!(
                "placement {package_id}: process-changing update requires a reviewed accepted proposal"
            )));
        }
    }
    Ok(())
}

/// Preserved placed input has its own explicit membership and authored dimensions.
/// It does not inherit native library or project-profile policy.
pub fn preserved_placed_process(raw: &Value) -> Result<PlacedProcessGeometry, EngineError> {
    use crate::board::{PadShape, PlacedPad};
    let pad: PlacedPad = serde_json::from_value(raw.clone())?;
    let membership = ProcessMembership {
        mask_layers: serde_json::from_value(
            raw.get("mask_layers")
                .ok_or_else(|| {
                    invalid(format!(
                        "Pad {}: missing explicit preserved mask_layers membership",
                        pad.uuid
                    ))
                })?
                .clone(),
        )?,
        paste_layers: serde_json::from_value(
            raw.get("paste_layers")
                .ok_or_else(|| {
                    invalid(format!(
                        "Pad {}: missing explicit preserved paste_layers membership",
                        pad.uuid
                    ))
                })?
                .clone(),
        )?,
    };
    let require = |field: &str| -> Result<(), EngineError> {
        if raw.get(field).is_none() {
            return Err(invalid(format!(
                "Pad {}: enabled preserved process requires explicit {field}",
                pad.uuid
            )));
        }
        Ok(())
    };
    if !membership.mask_layers.is_empty() {
        require("solder_mask_margin_nm")?;
    }
    if !membership.paste_layers.is_empty() {
        require("solder_paste_margin_nm")?;
        require("solder_paste_margin_ratio_ppm")?;
    }
    let enabled = !membership.mask_layers.is_empty() || !membership.paste_layers.is_empty();
    if enabled {
        require("shape")?;
        match pad.shape {
            PadShape::Circle => require("diameter")?,
            PadShape::Rect => {
                require("width")?;
                require("height")?;
            }
            _ => {
                return Err(invalid(format!(
                    "Pad {}: unsupported retained process shape",
                    pad.uuid
                )));
            }
        }
    }
    let aperture = match (enabled, pad.shape) {
        (false, _) => Value::Null,
        (true, PadShape::Circle) => serde_json::json!({"circle": {"diameter_nm": pad.diameter}}),
        (true, PadShape::Rect) => {
            serde_json::json!({"rect": {"width_nm": pad.width, "height_nm": pad.height}})
        }
        _ => {
            return Err(invalid(format!(
                "Pad {}: unsupported retained process shape",
                pad.uuid
            )));
        }
    };
    let mask = if membership.mask_layers.is_empty() {
        serde_json::json!("tented")
    } else {
        serde_json::json!({"expansion_nm": pad.solder_mask_margin_nm})
    };
    let paste = if membership.paste_layers.is_empty() {
        serde_json::json!("none")
    } else {
        serde_json::json!("inherit")
    };
    let source = serde_json::json!({"uuid": pad.uuid, "name": pad.name, "aperture": aperture,
        "mask_policy": mask, "paste_policy": paste});
    let footprint = serde_json::json!({"process_basis": {"kind": "inline", "basis": {
        "mask": "tented", "paste": {"expansion_nm": pad.solder_paste_margin_nm},
        "paste_ratio_ppm": pad.solder_paste_margin_ratio_ppm,
        "declared_basis": "preserved authored placed input", "provenance": raw,
    }}});
    let effective = resolve_padstack_process(&source, &footprint, None)
        .map_err(|e| invalid(format!("preserved Pad {}: {e}", pad.uuid)))?;
    use crate::board::{nominal_geometry::Rational, quadratic3::Quadratic3};
    Ok(PlacedProcessGeometry {
        pad_id: pad.uuid,
        source_capability: None,
        center_nm: [
            Quadratic3::rational(Rational::integer(i128::from(pad.position.x))),
            Quadratic3::rational(Rational::integer(i128::from(pad.position.y))),
        ],
        rotation: pad.rotation,
        reflected: false,
        effective,
        membership,
    })
}

fn hash(value: &Value) -> Result<String, EngineError> {
    Ok(crate::substrate::sha256_hex(
        crate::ir::serialization::to_json_deterministic(value)?.as_bytes(),
    ))
}

fn stackup_hash(
    stackup: &Stackup,
    capability: &ProcessSourceCapability,
) -> Result<String, EngineError> {
    hash(&serde_json::json!({"stackup": stackup, "capability": capability}))
}

fn pinned<'a>(profile: &'a Option<FrozenProfilePin>) -> Option<PinnedProcessProfile<'a>> {
    profile.as_ref().map(|p| PinnedProcessProfile {
        profile: &p.profile,
        object_revision: p.object_revision.0,
        value_hash: &p.value_hash,
    })
}

pub(crate) fn validate_dependency_layers(
    board: &Value,
    snapshot: &FrozenProcessSnapshot,
    pad: &ComponentPadSource,
) -> Result<(), EngineError> {
    let capability = capability_from_board(board)?
        .ok_or_else(|| invalid("process snapshot requires retained process capability"))?;
    let stackup: Stackup = serde_json::from_value(board["stackup"].clone())?;
    capability.validate(&stackup)?;
    let pose: PlacedPackage = serde_json::from_value(
        board["packages"][snapshot.transform_basis.uuid.to_string()].clone(),
    )?;
    let physical = super::projection::project_pad(pad, &pose, &snapshot.stackup)?;
    let current_pose = snapshot.project(pad, &pose)?;
    let layers: std::collections::BTreeSet<_> = snapshot
        .membership
        .mask_layers
        .iter()
        .chain(&snapshot.membership.paste_layers)
        .chain(&current_pose.membership.mask_layers)
        .chain(&current_pose.membership.paste_layers)
        .chain(&physical.terminal().copper_layers)
        .copied()
        .collect();
    for id in layers {
        let old = snapshot
            .stackup
            .layers
            .iter()
            .find(|l| l.id == id)
            .ok_or_else(|| invalid("frozen process layer is absent from frozen stackup"))?;
        if !stackup
            .layers
            .iter()
            .any(|l| l.id == id && l.layer_type == old.layer_type)
        {
            return Err(invalid("stackup edit would orphan a frozen process layer"));
        }
    }
    Ok(())
}

pub(crate) fn capture(
    model: &DesignModel,
    board: &Value,
    pad: &ComponentPadSource,
    placement: &PlacedPackage,
) -> Result<Option<FrozenProcessSnapshot>, EngineError> {
    let Some(capability) = capability_from_board(board)? else {
        return Ok(None);
    };
    let binding = pad
        .source_binding
        .as_ref()
        .ok_or_else(|| invalid("missing source binding for process materialization"))?;
    let raw_padstack = source_value(model, &binding.padstack)?;
    let raw_footprint = source_value(model, &binding.land_pattern)?;
    let selector: Option<ProcessBasisSelector> = raw_footprint
        .get("process_basis")
        .map(|v| serde_json::from_value(v.clone()))
        .transpose()?;
    let profile = if let Some(ProcessBasisSelector::ProjectProfile { profile_id, .. }) = selector {
        let profiles = crate::api::native_write::process::profiles(model)?
            .ok_or_else(|| invalid("selected process profile collection is absent"))?;
        let profile = profiles
            .get(&profile_id)
            .cloned()
            .ok_or_else(|| invalid(format!("selected process profile {profile_id} is absent")))?;
        let object = model.objects.get(&profile_id).ok_or_else(|| {
            invalid(format!(
                "selected process profile {profile_id} lacks canonical identity"
            ))
        })?;
        let value_hash = hash(&serde_json::to_value(&profile)?)?;
        Some(FrozenProfilePin {
            project_id: model.project.project_id,
            profile,
            object_revision: object.object_revision,
            value_hash,
        })
    } else {
        None
    };
    let effective = resolve_padstack_process(&raw_padstack, &raw_footprint, pinned(&profile))
        .map_err(|e| {
            invalid(format!(
                "placement {} Pad {} Footprint {} Padstack {}: {e}",
                placement.uuid,
                pad.uuid,
                binding.land_pattern.object_id,
                binding.padstack.object_id
            ))
        })?;
    let stackup: Stackup = serde_json::from_value(board["stackup"].clone())?;
    let projected = super::projection::project_pad(pad, placement, &stackup)?;
    let membership =
        capability.membership(&stackup, &projected.terminal().copper_layers, &effective)?;
    let stackup_value_hash = stackup_hash(&stackup, &capability)?;
    Ok(Some(FrozenProcessSnapshot {
        version: 1,
        effective,
        membership,
        capability,
        stackup,
        stackup_value_hash,
        profile,
        raw_padstack,
        raw_footprint,
        transform_basis: placement.clone(),
    }))
}

impl FrozenProcessSnapshot {
    pub fn validate(&self, pad: &ComponentPadSource) -> Result<(), EngineError> {
        if self.version != 1 || self.effective.resolver_version != 1 {
            return Err(invalid(
                "unsupported frozen process snapshot/resolver version",
            ));
        }
        let binding = pad
            .source_binding
            .as_ref()
            .ok_or_else(|| invalid("process snapshot lacks binding"))?;
        if self.transform_basis.uuid != binding.package_id
            || hash(&self.raw_padstack)? != binding.padstack.owner_value_hash
            || hash(&self.raw_footprint)? != binding.land_pattern.owner_value_hash
            || self.raw_padstack["uuid"] != serde_json::json!(binding.padstack.object_id)
            || self.raw_footprint["uuid"] != serde_json::json!(binding.land_pattern.object_id)
            || self.raw_padstack["aperture"]
                != serde_json::to_value(&binding.frozen_geometry.aperture)?
        {
            return Err(invalid(
                "frozen process source/transform correspondence mismatch",
            ));
        }
        if let Some(profile) = &self.profile {
            if profile.project_id != binding.project_id
                || hash(&serde_json::to_value(&profile.profile)?)? != profile.value_hash
            {
                return Err(invalid("frozen profile pin mismatch"));
            }
        }
        let expected = resolve_padstack_process(
            &self.raw_padstack,
            &self.raw_footprint,
            pinned(&self.profile),
        )?;
        if expected != self.effective
            || stackup_hash(&self.stackup, &self.capability)? != self.stackup_value_hash
        {
            return Err(invalid(
                "frozen process geometry/input or stackup pin mismatch",
            ));
        }
        let physical = super::projection::project_pad(pad, &self.transform_basis, &self.stackup)?;
        if self.capability.membership(
            &self.stackup,
            &physical.terminal().copper_layers,
            &self.effective,
        )? != self.membership
        {
            return Err(invalid("frozen process membership mismatch"));
        }
        Ok(())
    }

    pub fn check_current(&self, model: &DesignModel) -> Result<(), EngineError> {
        let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
        let capability = capability_from_board(&board)?
            .ok_or_else(|| invalid("current process capability is absent"))?;
        let stackup: Stackup = serde_json::from_value(board["stackup"].clone())?;
        if stackup_hash(&stackup, &capability)? != self.stackup_value_hash {
            return Err(invalid("frozen process stackup/association pin is stale"));
        }
        if let Some(pin) = &self.profile {
            let profiles = crate::api::native_write::process::profiles(model)?
                .ok_or_else(|| invalid("frozen process profile is absent"))?;
            let current = profiles
                .get(&pin.profile.uuid)
                .ok_or_else(|| invalid("frozen process profile is deleted"))?;
            if model
                .objects
                .get(&pin.profile.uuid)
                .is_none_or(|o| o.object_revision != pin.object_revision)
                || hash(&serde_json::to_value(current)?)? != pin.value_hash
            {
                return Err(invalid("frozen process profile pin is stale"));
            }
        }
        Ok(())
    }

    pub fn project(
        &self,
        pad: &ComponentPadSource,
        placement: &PlacedPackage,
    ) -> Result<PlacedProcessGeometry, EngineError> {
        let projected = super::projection::project_pad(pad, placement, &self.stackup)?;
        Ok(PlacedProcessGeometry {
            pad_id: pad.uuid,
            source_capability: Some(self.capability.clone()),
            center_nm: projected.center(),
            rotation: placement.rotation,
            reflected: placement.layer != self.capability.front.copper_layer,
            effective: self.effective.clone(),
            membership: self.capability.membership(
                &self.stackup,
                &projected.terminal().copper_layers,
                &self.effective,
            )?,
        })
    }
}

#[cfg(test)]
mod tests;
