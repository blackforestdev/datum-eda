//! Read-only, exact revision differences for a designer's update review.
//! This performs no graph, rules, publication or engineering approval.
use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::{
    AssetKind, AssetRevisionRef, Dependency, VerifiedAsset, encode_metadata,
    exact_json::{self, ExactValue},
    invalid, payload_reference_slots,
};
use crate::error::EngineError;

pub(crate) struct ReviewInput<'a> {
    pub(crate) asset: &'a VerifiedAsset,
    /// Exact retained bytes, never a serialization of the parsed payload.
    pub(crate) raw_payload: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ChangeCategory {
    Geometry,
    Mapping,
    Designation,
    ElectricalDefinition,
    Property,
    Dependency,
    Provenance,
    CheckBasis,
    RecordedCheckClaim,
    RevisionMetadata,
    Unclassified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReviewUnit {
    Nanometres,
    PartsPerMillion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FieldChange {
    pub(crate) category: ChangeCategory,
    pub(crate) path: Vec<String>,
    pub(crate) label: String,
    /// None means absent, while Some("null") is an authored JSON null.
    /// Numeric text is exact decimal notation, never an f64 display.
    pub(crate) before: Option<String>,
    pub(crate) after: Option<String>,
    pub(crate) unit: Option<ReviewUnit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct DependencyChange {
    pub(crate) role: String,
    pub(crate) payload_path: Vec<String>,
    pub(crate) before: Option<AssetRevisionRef>,
    pub(crate) after: Option<AssetRevisionRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CheckCoverage {
    /// Recorded assertions/basis below are displayed facts, not executed checks.
    NotEvaluated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReviewComparison {
    Revisions,
    /// Existing native content may exist; no prior managed record was supplied.
    FirstManagedRecord,
    RemovedManagedRecord,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct AssetChangeReview {
    pub(crate) kind: AssetKind,
    pub(crate) comparison: ReviewComparison,
    pub(crate) before: Option<AssetRevisionRef>,
    pub(crate) after: Option<AssetRevisionRef>,
    pub(crate) payload_bytes_changed: bool,
    pub(crate) payload_semantics_changed: bool,
    pub(crate) engineering_basis_changed: bool,
    pub(crate) fields: Vec<FieldChange>,
    pub(crate) dependencies: Vec<DependencyChange>,
    pub(crate) provenance: Vec<FieldChange>,
    pub(crate) revision_metadata: Vec<FieldChange>,
    pub(crate) checks: CheckCoverage,
}

/// Compare exact selected records only; callers own closure selection, impact
/// and check execution. Never discover a newer revision or infer adoption.
pub(crate) fn compare(
    before: Option<ReviewInput<'_>>,
    after: Option<ReviewInput<'_>>,
) -> Result<AssetChangeReview, EngineError> {
    let subject = after
        .as_ref()
        .or(before.as_ref())
        .ok_or_else(|| invalid("change review requires an exact subject"))?;
    let kind = subject.asset.revision().kind;
    if let (Some(before), Some(after)) = (&before, &after)
        && (before.asset.reference().asset_id != after.asset.reference().asset_id
            || before.asset.revision().kind != after.asset.revision().kind)
    {
        return Err(invalid(
            "change review cannot pair different asset identities or kinds",
        ));
    }
    let read = |input: &ReviewInput<'_>| {
        input
            .asset
            .revision()
            .payload_ref
            .verify_blob(input.raw_payload)?;
        exact_json::parse(input.raw_payload).map(|parsed| parsed.value)
    };
    let old = before.as_ref().map(read).transpose()?;
    let new = after.as_ref().map(read).transpose()?;
    let mut slots = BTreeSet::new();
    let mut known = BTreeSet::new();
    for input in before.iter().chain(after.iter()) {
        slots.extend(
            payload_reference_slots(kind, input.asset.payload().value())?
                .into_iter()
                .map(|slot| slot.path),
        );
        known.extend(known_paths(kind, input.asset.payload().value())?);
    }
    let fields = differences(old.as_ref(), new.as_ref(), |path| {
        if !known.contains(path) {
            return (
                ChangeCategory::Unclassified,
                format!("Unclassified authored field: {}", readable(path)),
                None,
            );
        }
        let (category, label) = classify(kind, path, &slots);
        (category, label, unit(kind, path))
    })?;
    let metadata = |input: &ReviewInput<'_>, provenance: bool| {
        let bytes = if provenance {
            encode_metadata(input.asset.provenance())?
        } else {
            encode_metadata(input.asset.revision())?
        };
        exact_json::parse(&bytes).map(|parsed| parsed.value)
    };
    let old_provenance = before.as_ref().map(|i| metadata(i, true)).transpose()?;
    let new_provenance = after.as_ref().map(|i| metadata(i, true)).transpose()?;
    let old_record = before.as_ref().map(|i| metadata(i, false)).transpose()?;
    let new_record = after.as_ref().map(|i| metadata(i, false)).transpose()?;
    let dependencies = dependency_changes(
        before
            .as_ref()
            .map(|i| i.asset.revision().dependencies.as_slice())
            .unwrap_or(&[]),
        after
            .as_ref()
            .map(|i| i.asset.revision().dependencies.as_slice())
            .unwrap_or(&[]),
    );
    let payload_semantics_changed = old != new;
    Ok(AssetChangeReview {
        kind,
        comparison: if before.is_none() {
            ReviewComparison::FirstManagedRecord
        } else if after.is_none() {
            ReviewComparison::RemovedManagedRecord
        } else {
            ReviewComparison::Revisions
        },
        before: before.as_ref().map(|i| i.asset.reference().clone()),
        after: after.as_ref().map(|i| i.asset.reference().clone()),
        payload_bytes_changed: before.as_ref().map(|i| i.raw_payload)
            != after.as_ref().map(|i| i.raw_payload),
        payload_semantics_changed,
        engineering_basis_changed: payload_semantics_changed || !dependencies.is_empty(),
        fields,
        dependencies,
        provenance: differences(old_provenance.as_ref(), new_provenance.as_ref(), |path| {
            (
                ChangeCategory::Provenance,
                format!("Provenance: {}", readable(path)),
                None,
            )
        })?,
        revision_metadata: differences(old_record.as_ref(), new_record.as_ref(), |path| {
            (
                ChangeCategory::RevisionMetadata,
                format!("Revision: {}", readable(path)),
                None,
            )
        })?,
        checks: CheckCoverage::NotEvaluated,
    })
}

fn dependency_changes(before: &[Dependency], after: &[Dependency]) -> Vec<DependencyChange> {
    let index = |rows: &[Dependency]| {
        rows.iter()
            .map(|d| ((d.payload_path.clone(), d.role.clone()), d.target.clone()))
            .collect::<BTreeMap<_, _>>()
    };
    let old = index(before);
    let new = index(after);
    old.keys()
        .chain(new.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|key| old.get(*key) != new.get(*key))
        .map(|(path, role)| DependencyChange {
            role: role.clone(),
            payload_path: path.clone(),
            before: old.get(&(path.clone(), role.clone())).cloned(),
            after: new.get(&(path.clone(), role.clone())).cloned(),
        })
        .collect()
}

fn differences(
    before: Option<&ExactValue>,
    after: Option<&ExactValue>,
    classify: impl Fn(&[String]) -> (ChangeCategory, String, Option<ReviewUnit>),
) -> Result<Vec<FieldChange>, EngineError> {
    fn walk(
        before: Option<&ExactValue>,
        after: Option<&ExactValue>,
        path: &mut Vec<String>,
        out: &mut Vec<FieldChange>,
        classify: &impl Fn(&[String]) -> (ChangeCategory, String, Option<ReviewUnit>),
    ) -> Result<(), EngineError> {
        if before == after {
            return Ok(());
        }
        match (before, after) {
            (Some(ExactValue::Object(a)), Some(ExactValue::Object(b))) => {
                for key in a.keys().chain(b.keys()).collect::<BTreeSet<_>>() {
                    path.push(key.clone());
                    walk(a.get(key), b.get(key), path, out, classify)?;
                    path.pop();
                }
                return Ok(());
            }
            (Some(ExactValue::Array(a)), Some(ExactValue::Array(b))) => {
                for index in 0..a.len().max(b.len()) {
                    path.push(index.to_string());
                    walk(a.get(index), b.get(index), path, out, classify)?;
                    path.pop();
                }
                return Ok(());
            }
            (None, Some(ExactValue::Object(rows))) | (Some(ExactValue::Object(rows)), None)
                if !rows.is_empty() =>
            {
                for (key, value) in rows {
                    path.push(key.clone());
                    walk(
                        before.map(|_| value),
                        after.map(|_| value),
                        path,
                        out,
                        classify,
                    )?;
                    path.pop();
                }
                return Ok(());
            }
            (None, Some(ExactValue::Array(rows))) | (Some(ExactValue::Array(rows)), None)
                if !rows.is_empty() =>
            {
                for (index, value) in rows.iter().enumerate() {
                    path.push(index.to_string());
                    walk(
                        before.map(|_| value),
                        after.map(|_| value),
                        path,
                        out,
                        classify,
                    )?;
                    path.pop();
                }
                return Ok(());
            }
            _ => {}
        }
        let (category, label, unit) = classify(path);
        out.push(FieldChange {
            category,
            path: path.clone(),
            label,
            before: before.map(display).transpose()?,
            after: after.map(display).transpose()?,
            unit,
        });
        Ok(())
    }
    let mut out = Vec::new();
    walk(before, after, &mut Vec::new(), &mut out, &classify)?;
    Ok(out)
}

fn display(value: &ExactValue) -> Result<String, EngineError> {
    Ok(match value {
        ExactValue::Null => "null".into(),
        ExactValue::Bool(value) => value.to_string(),
        ExactValue::String(value) => serde_json::to_string(value)?,
        ExactValue::Number {
            negative,
            digits,
            exponent,
        } => {
            let sign = if *negative { "-" } else { "" };
            if *exponent == 0 {
                format!("{sign}{digits}")
            } else {
                format!("{sign}{digits}e{exponent}")
            }
        }
        ExactValue::Array(rows) => format!(
            "[{}]",
            rows.iter()
                .map(display)
                .collect::<Result<Vec<_>, _>>()?
                .join(",")
        ),
        ExactValue::Object(rows) => format!(
            "{{{}}}",
            rows.iter()
                .map(|(key, value)| Ok(format!(
                    "{}:{}",
                    serde_json::to_string(key)?,
                    display(value)?
                )))
                .collect::<Result<Vec<_>, EngineError>>()?
                .join(",")
        ),
    })
}

fn readable(path: &[String]) -> String {
    path.join(" / ").replace('_', " ")
}

/// Native serialization identifies owned field paths only. Its values are never
/// compared, hashed or displayed; the exact retained tree remains authoritative.
fn known_paths(
    kind: AssetKind,
    raw: &serde_json::Value,
) -> Result<BTreeSet<Vec<String>>, EngineError> {
    macro_rules! shape {
        ($ty:ty) => {
            serde_json::to_value(serde_json::from_value::<$ty>(raw.clone())?)?
        };
    }
    let shape = match kind {
        AssetKind::Unit => shape!(crate::pool::Unit),
        AssetKind::Symbol => shape!(crate::pool::Symbol),
        AssetKind::Entity => shape!(crate::pool::Entity),
        AssetKind::Part => shape!(crate::pool::Part),
        AssetKind::Package => shape!(crate::pool::Package),
        AssetKind::Footprint => shape!(crate::pool::Footprint),
        AssetKind::Padstack => shape!(crate::pool::Padstack),
        AssetKind::PinPadMap => shape!(crate::pool::PinPadMap),
        AssetKind::ModelAttachment => shape!(crate::pool::ModelAttachment),
    };
    fn walk(value: &serde_json::Value, path: &mut Vec<String>, out: &mut BTreeSet<Vec<String>>) {
        out.insert(path.clone());
        match value {
            serde_json::Value::Object(rows) => {
                for (key, value) in rows {
                    path.push(key.clone());
                    walk(value, path, out);
                    path.pop();
                }
            }
            serde_json::Value::Array(rows) => {
                for (index, value) in rows.iter().enumerate() {
                    path.push(index.to_string());
                    walk(value, path, out);
                    path.pop();
                }
            }
            _ => {}
        }
    }
    let mut out = BTreeSet::new();
    walk(&shape, &mut Vec::new(), &mut out);
    // Process selectors are owned by the shared process schema, adjacent to
    // Footprint rather than fields on its legacy Rust struct. Classify their
    // declared shape without resolving a profile or evaluating process rules.
    if kind == AssetKind::Footprint
        && let Some(value) = raw.get("process_basis")
        && let Ok(selector) =
            serde_json::from_value::<crate::pool::process::ProcessBasisSelector>(value.clone())
    {
        walk(
            &serde_json::to_value(selector)?,
            &mut vec!["process_basis".into()],
            &mut out,
        );
    }
    // The native SymbolPinAnchor deserializer explicitly owns these legacy
    // aliases, while serialization folds them into style. Keep that distinction.
    if kind == AssetKind::Symbol
        && let Some(rows) = raw.get("pin_anchors").and_then(serde_json::Value::as_array)
    {
        for (index, row) in rows.iter().enumerate() {
            for alias in ["orientation", "length_nm", "decoration"] {
                if row.get(alias).is_some() {
                    out.insert(vec!["pin_anchors".into(), index.to_string(), alias.into()]);
                }
            }
        }
    }
    Ok(out)
}

fn classify(
    kind: AssetKind,
    path: &[String],
    slots: &BTreeSet<Vec<String>>,
) -> (ChangeCategory, String) {
    use ChangeCategory as C;
    let root = path.first().map(String::as_str).unwrap_or("");
    let category = if slots.contains(path) {
        C::Dependency
    } else if matches!(
        (kind, root),
        (AssetKind::Symbol | AssetKind::Footprint, "standards_basis")
            | (
                AssetKind::Footprint,
                "ipc_basis" | "process_basis" | "process_aperture_policy"
            )
            | (AssetKind::Symbol, "style_profile_assertions")
    ) {
        C::CheckBasis
    } else if kind == AssetKind::Symbol && root == "check_state" {
        C::RecordedCheckClaim
    } else if kind == AssetKind::Symbol && root == "provenance" {
        C::Provenance
    } else {
        let p: Vec<_> = path.iter().map(String::as_str).collect();
        match (kind, root, p.as_slice()) {
            (AssetKind::Symbol, _, ["pin_anchors", _, "number"])
            | (AssetKind::Footprint | AssetKind::Package, _, ["pads", _, "name"]) => C::Designation,
            (AssetKind::Symbol, _, ["pin_anchors", _, "pin"]) => C::ElectricalDefinition,
            (AssetKind::Symbol, _, ["fields", _, "position", ..]) => C::Geometry,
            (AssetKind::PinPadMap, "mappings" | "part" | "footprint", _)
            | (AssetKind::Part, "pad_map", _) => C::Mapping,
            (AssetKind::Unit, "pins", _) | (AssetKind::Entity, "gates", _) => {
                C::ElectricalDefinition
            }
            (AssetKind::Symbol, "drawings" | "pin_anchors", _)
            | (
                AssetKind::Footprint,
                "pads" | "courtyard" | "silkscreen" | "fab" | "assembly" | "mechanical",
                _,
            )
            | (AssetKind::Package, "pads", _)
            | (
                AssetKind::Padstack,
                "aperture" | "drill_nm" | "plated" | "layer_span" | "mask_policy" | "paste_policy"
                | "annular_ring_nm" | "thermal" | "antipad",
                _,
            ) => C::Geometry,
            (
                _,
                "name" | "manufacturer" | "prefix" | "mpn" | "value" | "description" | "datasheet"
                | "tags" | "parametric" | "orderable_mpns" | "lifecycle" | "fields",
                _,
            ) => C::Property,
            _ => C::Unclassified,
        }
    };
    let heading = match category {
        C::Geometry => "Geometry",
        C::Mapping => "Pin/pad mapping",
        C::Designation => "Designation",
        C::ElectricalDefinition => "Electrical definition",
        C::Property => "Property",
        C::Dependency => "Dependency",
        C::Provenance => "Recorded provenance",
        C::CheckBasis => "Recorded check basis",
        C::RecordedCheckClaim => "Recorded check claim",
        C::RevisionMetadata => "Revision",
        C::Unclassified => "Unclassified authored field",
    };
    (category, format!("{heading}: {}", readable(path)))
}

fn unit(kind: AssetKind, path: &[String]) -> Option<ReviewUnit> {
    use ReviewUnit as U;
    let p: Vec<_> = path.iter().map(String::as_str).collect();
    if let [root, _, drawing @ ..] = p.as_slice()
        && matches!(
            (kind, *root),
            (AssetKind::Symbol, "drawings")
                | (
                    AssetKind::Footprint,
                    "silkscreen" | "fab" | "assembly" | "mechanical"
                )
        )
        && matches!(
            drawing,
            ["Line" | "Rect" | "Circle" | "Polygon" | "Arc", "width"]
                | ["Circle", "radius"]
                | ["Line", "from" | "to", "x" | "y"]
                | ["Rect", "min" | "max", "x" | "y"]
                | ["Circle", "center", "x" | "y"]
                | ["Text", "position", "x" | "y"]
                | ["Polygon", "polygon", "vertices", _, "x" | "y"]
        )
    {
        return Some(U::Nanometres);
    }
    match (kind, p.as_slice()) {
        (AssetKind::Padstack, ["drill_nm" | "annular_ring_nm"])
        | (AssetKind::Padstack, ["thermal", "spoke_width_nm" | "gap_nm"])
        | (AssetKind::Padstack, ["antipad", "clearance_nm"])
        | (AssetKind::Padstack, ["mask_policy" | "paste_policy", "expansion_nm"]) => {
            Some(U::Nanometres)
        }
        (AssetKind::Padstack, ["aperture", "circle", "diameter_nm"])
        | (AssetKind::Padstack, ["antipad", "aperture", "circle", "diameter_nm"])
        | (AssetKind::Padstack, ["aperture", "rect" | "round_rect", "width_nm" | "height_nm"])
        | (
            AssetKind::Padstack,
            [
                "antipad",
                "aperture",
                "rect" | "round_rect",
                "width_nm" | "height_nm",
            ],
        ) => Some(U::Nanometres),
        (AssetKind::Padstack, ["aperture", "round_rect", "rratio_ppm"]) => Some(U::PartsPerMillion),
        (AssetKind::Footprint | AssetKind::Package, ["pads", _, "position", "x" | "y"])
        | (AssetKind::Symbol, ["pin_anchors", _, "position", "x" | "y"])
        | (AssetKind::Symbol, ["pin_anchors", _, "style", "length_nm"])
        | (AssetKind::Symbol, ["pin_anchors", _, "length_nm"]) => Some(U::Nanometres),
        (AssetKind::Symbol, ["fields", _, "position", "x" | "y"]) => Some(U::Nanometres),
        (AssetKind::Footprint, ["courtyard", "vertices", _, "x" | "y"]) => Some(U::Nanometres),
        _ => None,
    }
}

#[cfg(test)]
#[path = "change_review_tests.rs"]
mod tests;
