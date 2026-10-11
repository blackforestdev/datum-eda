//! PM061 native process resolution. Inputs are explicit; no preference or read-time default.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::board::nominal_geometry::Rational;
use crate::board::quadratic3::Quadratic3;
use crate::error::EngineError;

use super::{
    IpcFootprintBasis, Padstack, PadstackAperture, PadstackMaskPolicy, PadstackPastePolicy,
};

mod faces;
#[cfg(test)]
mod tests;
pub use faces::{ProcessFace, ProcessMembership, ProcessSourceCapability};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConcreteProcessBasis {
    pub mask: PadstackMaskPolicy,
    pub paste: PadstackPastePolicy,
    pub paste_ratio_ppm: i64,
    pub declared_basis: String,
    pub provenance: Value,
}

impl ConcreteProcessBasis {
    pub fn validate(&self) -> Result<(), EngineError> {
        if self.mask == PadstackMaskPolicy::Inherit || self.paste == PadstackPastePolicy::Inherit {
            return Err(error("a concrete basis cannot inherit"));
        }
        if self.declared_basis.trim().is_empty() || self.provenance.is_null() {
            return Err(error(
                "a concrete basis requires declared basis and authored provenance",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessProfile {
    pub uuid: Uuid,
    pub name: String,
    pub basis: ConcreteProcessBasis,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProcessBasisSelector {
    Inline {
        basis: ConcreteProcessBasis,
    },
    IpcBasis,
    ProjectProfile {
        profile_id: Uuid,
        expected_object_revision: u64,
        expected_value_hash: String,
    },
}

/// A profile must arrive with canonical revision/hash evidence from the model.
pub struct PinnedProcessProfile<'a> {
    pub profile: &'a ProcessProfile,
    pub object_revision: u64,
    pub value_hash: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EffectiveProcessAperture {
    Suppressed,
    Circle {
        diameter_nm: Quadratic3,
    },
    Rect {
        width_nm: Quadratic3,
        height_nm: Quadratic3,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectivePadstackProcess {
    pub resolver_version: u32,
    pub mask: EffectiveProcessAperture,
    pub paste: EffectiveProcessAperture,
    // None means raw absence; Some("inherit") remains explicitly authored.
    pub raw_mask_policy: Option<Value>,
    pub raw_paste_policy: Option<Value>,
    pub selected_basis: Option<ProcessBasisSelector>,
    pub resolved_basis: Option<ConcreteProcessBasis>,
    pub standards_basis: Option<String>,
}

pub fn resolve_padstack_process(
    raw_padstack: &Value,
    raw_footprint: &Value,
    profile: Option<PinnedProcessProfile<'_>>,
) -> Result<EffectivePadstackProcess, EngineError> {
    let padstack: Padstack = serde_json::from_value(raw_padstack.clone())
        .map_err(|e| error(&format!("invalid Padstack: {e}")))?;
    let selector: Option<ProcessBasisSelector> = raw_footprint
        .get("process_basis")
        .map(|v| serde_json::from_value(v.clone()))
        .transpose()
        .map_err(|e| error(&format!("invalid Footprint process_basis: {e}")))?;
    validate_selected_profile(selector.as_ref(), profile.as_ref())?;
    let inherits = padstack.mask_policy == PadstackMaskPolicy::Inherit
        || padstack.paste_policy == PadstackPastePolicy::Inherit;
    let basis = if inherits {
        Some(resolve_basis(selector.as_ref(), raw_footprint, profile)?)
    } else {
        None
    };
    let mask = match &padstack.mask_policy {
        PadstackMaskPolicy::Inherit => &basis.as_ref().expect("resolved inherited basis").mask,
        policy => policy,
    };
    let (mask_enabled, mask_margin) = match mask {
        PadstackMaskPolicy::Tented => (false, 0),
        PadstackMaskPolicy::Exposed => (true, 0),
        PadstackMaskPolicy::ExpansionNm(margin) => (true, *margin),
        PadstackMaskPolicy::Inherit => return Err(error("recursive mask inheritance")),
    };
    let (paste, ratio) = match &padstack.paste_policy {
        PadstackPastePolicy::Inherit => {
            let basis = basis.as_ref().expect("resolved inherited basis");
            (&basis.paste, basis.paste_ratio_ppm)
        }
        policy => (policy, 0),
    };
    let (paste_enabled, paste_margin) = match paste {
        PadstackPastePolicy::None => (false, 0),
        PadstackPastePolicy::Aperture => (true, 0),
        PadstackPastePolicy::ExpansionNm(margin) => (true, *margin),
        PadstackPastePolicy::Inherit => return Err(error("recursive paste inheritance")),
    };
    Ok(EffectivePadstackProcess {
        resolver_version: 1,
        mask: effective_aperture(
            padstack.aperture.as_ref(),
            mask_enabled,
            mask_margin,
            0,
            "mask",
        )?,
        paste: effective_aperture(
            padstack.aperture.as_ref(),
            paste_enabled,
            paste_margin,
            ratio,
            "paste",
        )?,
        raw_mask_policy: raw_padstack.get("mask_policy").cloned(),
        raw_paste_policy: raw_padstack.get("paste_policy").cloned(),
        selected_basis: selector,
        resolved_basis: basis,
        standards_basis: raw_footprint
            .get("standards_basis")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}

fn validate_selected_profile(
    selector: Option<&ProcessBasisSelector>,
    profile: Option<&PinnedProcessProfile<'_>>,
) -> Result<(), EngineError> {
    if let Some(ProcessBasisSelector::ProjectProfile {
        profile_id,
        expected_object_revision,
        expected_value_hash,
    }) = selector
    {
        let pin = profile.ok_or_else(|| error("selected project profile is missing"))?;
        if pin.profile.uuid != *profile_id
            || pin.object_revision != *expected_object_revision
            || pin.value_hash != expected_value_hash
        {
            return Err(error("selected project profile pin is stale or mismatched"));
        }
    }
    Ok(())
}

fn resolve_basis(
    selector: Option<&ProcessBasisSelector>,
    footprint: &Value,
    profile: Option<PinnedProcessProfile<'_>>,
) -> Result<ConcreteProcessBasis, EngineError> {
    let basis =
        match selector.ok_or_else(|| error("inherited policy has no Footprint process_basis"))? {
            ProcessBasisSelector::Inline { basis } => basis.clone(),
            ProcessBasisSelector::IpcBasis => {
                let ipc: IpcFootprintBasis = serde_json::from_value(
                    footprint
                        .get("ipc_basis")
                        .ok_or_else(|| error("IPC selector has no structured ipc_basis"))?
                        .clone(),
                )
                .map_err(|e| error(&format!("invalid IPC basis: {e}")))?;
                if ipc.mask_expansion_nm < 0 || ipc.paste_reduction_nm < 0 {
                    return Err(error(
                        "IPC mask expansion and paste reduction must be nonnegative",
                    ));
                }
                ConcreteProcessBasis {
                    mask: PadstackMaskPolicy::ExpansionNm(ipc.mask_expansion_nm),
                    paste: PadstackPastePolicy::ExpansionNm(-ipc.paste_reduction_nm),
                    paste_ratio_ppm: 0,
                    declared_basis: format!("{} {}", ipc.family, ipc.revision),
                    provenance: footprint["ipc_basis"].clone(),
                }
            }
            ProcessBasisSelector::ProjectProfile { .. } => profile
                .ok_or_else(|| error("selected project profile is missing"))?
                .profile
                .basis
                .clone(),
        };
    basis.validate()?;
    Ok(basis)
}

fn effective_aperture(
    copper: Option<&PadstackAperture>,
    enabled: bool,
    margin: i64,
    ratio: i64,
    process: &str,
) -> Result<EffectiveProcessAperture, EngineError> {
    if !enabled {
        return Ok(EffectiveProcessAperture::Suppressed);
    }
    let dimension = |nm: i64| -> Result<Quadratic3, EngineError> {
        if nm <= 0 {
            return Err(error(&format!("{process}: nonpositive copper dimension")));
        }
        let factor = Rational::new(i128::from(ratio) + 1_000_000, 1_000_000)
            .map_err(|_| error(&format!("{process}: ratio arithmetic overflow")))?;
        let result = Rational::integer(i128::from(nm))
            .checked_mul(factor)
            .and_then(|v| v.checked_add(Rational::integer(i128::from(margin) * 2)))
            .map_err(|_| error(&format!("{process}: dimension arithmetic overflow")))?;
        if result.numerator <= 0 {
            return Err(error(&format!("{process}: collapsed aperture")));
        }
        if result
            .compare(Rational::integer(i128::from(i64::MAX)))
            .map_err(|_| error(&format!("{process}: dimension arithmetic overflow")))?
            == std::cmp::Ordering::Greater
        {
            return Err(error(&format!(
                "{process}: dimension exceeds native nanometre domain"
            )));
        }
        Ok(Quadratic3::rational(result))
    };
    match copper.ok_or_else(|| error(&format!("{process}: missing copper aperture")))? {
        PadstackAperture::Circle { diameter_nm } => Ok(EffectiveProcessAperture::Circle {
            diameter_nm: dimension(*diameter_nm)?,
        }),
        PadstackAperture::Rect {
            width_nm,
            height_nm,
        } => Ok(EffectiveProcessAperture::Rect {
            width_nm: dimension(*width_nm)?,
            height_nm: dimension(*height_nm)?,
        }),
    }
}

pub(super) fn error(reason: &str) -> EngineError {
    EngineError::Validation(format!("native process resolution: {reason}"))
}
