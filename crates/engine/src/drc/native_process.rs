//! Capable native checks consume canonical frozen/preserved authority; legacy checks stay separate.
use super::{DrcLocation, DrcSeverity, DrcViolation};
use crate::rules::ast::RuleType;
use crate::substrate::{
    DesignModel, SourceShardKind,
    component_pad_source::{
        ComponentPadMaterializationStatus, component_pad_projection,
        process::{PlacedProcessGeometry, preserved_placed_process},
    },
    process_operations::capability_from_board,
};
use uuid::Uuid;

pub(super) fn checks(model: &DesignModel, rules: &[RuleType]) -> Option<Vec<DrcViolation>> {
    if !rules.is_empty() && !rules.contains(&RuleType::ProcessAperture) {
        return None;
    }
    let board = match model.materialized_source_shard_value(SourceShardKind::BoardRoot) {
        Ok(board) => board,
        Err(_) => return None,
    };
    let board_id = board
        .get("uuid")
        .cloned()
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or(model.project.project_id);
    match capability_from_board(&board) {
        Ok(None) => return None,
        Err(error) => {
            return Some(vec![finding(
                board_id,
                DrcSeverity::Error,
                "native_process_unresolved",
                error.to_string(),
                None,
            )]);
        }
        Ok(Some(capability)) => {
            let valid = serde_json::from_value::<crate::board::Stackup>(board["stackup"].clone())
                .map_err(crate::error::EngineError::from)
                .and_then(|stackup| capability.validate(&stackup));
            if let Err(error) = valid {
                return Some(vec![finding(
                    board_id,
                    DrcSeverity::Error,
                    "native_process_unresolved",
                    error.to_string(),
                    None,
                )]);
            }
        }
    }
    let mut findings = Vec::new();
    match component_pad_projection(model, &board) {
        Err(error) => findings.push(finding(
            board_id,
            DrcSeverity::Error,
            "native_process_unresolved",
            error.to_string(),
            None,
        )),
        Ok(projection) => {
            for (id, reason) in &projection.unavailable {
                findings.push(finding(
                    *id,
                    DrcSeverity::Error,
                    "native_process_unresolved",
                    reason.clone(),
                    None,
                ));
            }
            for id in &projection.unadopted_packages {
                findings.push(finding(
                    *id,
                    DrcSeverity::Error,
                    "native_process_unresolved",
                    "placement requires explicit process materialization".into(),
                    None,
                ));
            }
            for id in projection.query_pads.keys() {
                if let Some(process) = projection.process_pads.get(id) {
                    findings.push(standards_finding(process));
                    if projection.statuses.get(id)
                        == Some(&ComponentPadMaterializationStatus::Stale)
                    {
                        findings.push(finding(*id, DrcSeverity::Warning, "native_process_snapshot_stale",
                            "Frozen process inputs are stale; inspect exact current differences and prepare a reviewed update".into(), location(process)));
                    }
                } else {
                    findings.push(finding(
                        *id,
                        DrcSeverity::Error,
                        "native_process_unresolved",
                        "placed Pad lacks frozen process authority".into(),
                        None,
                    ));
                }
            }
        }
    }
    for raw in board
        .get("pads")
        .and_then(serde_json::Value::as_object)
        .into_iter()
        .flat_map(|p| p.values())
    {
        match preserved_placed_process(raw) {
            Ok(process) => findings.push(standards_finding(&process)),
            Err(error) => findings.push(finding(
                raw.get("uuid")
                    .cloned()
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or(model.project.project_id),
                DrcSeverity::Error,
                "native_process_unresolved",
                error.to_string(),
                None,
            )),
        }
    }
    Some(findings)
}

fn location(process: &PlacedProcessGeometry) -> Option<DrcLocation> {
    Some(DrcLocation {
        x_nm: process.center_nm[0].integer_nm().ok()?,
        y_nm: process.center_nm[1].integer_nm().ok()?,
        layer: None,
    })
}

fn standards_finding(process: &PlacedProcessGeometry) -> DrcViolation {
    let evidence = process
        .effective
        .standards_basis
        .as_deref()
        .unwrap_or("unknown");
    finding(
        process.pad_id,
        DrcSeverity::Warning,
        "native_process_standards_unqualified",
        format!(
            "Exact process geometry is resolved; standards basis {evidence:?} is not qualified by this geometry check"
        ),
        location(process),
    )
}

fn finding(
    id: Uuid,
    severity: DrcSeverity,
    code: &str,
    message: String,
    location: Option<DrcLocation>,
) -> DrcViolation {
    DrcViolation {
        id: Uuid::new_v5(&id, format!("{code}:{message}").as_bytes()),
        code: code.into(),
        rule_type: RuleType::ProcessAperture,
        severity,
        message,
        location,
        objects: vec![id],
        fingerprint: None,
        standards_basis: None,
        rule_revision: Some("pm061.geometry.v1".into()),
        import_key: None,
        waived: false,
    }
}
