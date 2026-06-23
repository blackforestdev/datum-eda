use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::board::{Net, NetClass, PlacedPad, Track, Via};
use eda_engine::substrate::{
    CommitProvenance, CommitSource, DesignModel, Operation, OperationBatch, ProjectResolver,
    ProposalCreateRequest, ProposalSource, create_draft_proposal_from_batch,
    validate_proposal_apply,
};
use serde::Serialize;
use uuid::Uuid;

use super::command_project_native_inspect::NativeProjectCheckFindingView;
use super::{load_native_project_with_resolved_board, query_native_project_check_run};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectStandardsRepairProposalView {
    pub(crate) proposal_id: Uuid,
    pub(crate) repair_kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) affected_pad: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) affected_track: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) affected_via: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) affected_net_class: Option<Uuid>,
    pub(crate) finding_fingerprints: Vec<String>,
    pub(crate) codes: Vec<String>,
    pub(crate) prepared_against: String,
    pub(crate) prepared_against_current_model: bool,
    pub(crate) can_apply: bool,
    pub(crate) blocker_codes: Vec<String>,
    pub(crate) operations: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectStandardsRepairProposalReportView {
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) check_run_id: Uuid,
    pub(crate) proposal_count: usize,
    pub(crate) proposals: Vec<NativeProjectStandardsRepairProposalView>,
}

#[derive(Debug, Clone)]
struct PadRepair {
    finding_fingerprints: BTreeSet<String>,
    codes: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct TrackRepair {
    finding_fingerprints: BTreeSet<String>,
    codes: BTreeSet<String>,
    required_width: i64,
    net_class_id: Uuid,
}

#[derive(Debug, Clone)]
struct ViaRepair {
    finding_fingerprints: BTreeSet<String>,
    codes: BTreeSet<String>,
    via_drill: Option<i64>,
    via_annular: Option<i64>,
    net_class_id: Uuid,
}

#[derive(Debug, Clone, Default)]
struct DimensionGeometryRepairs {
    tracks: BTreeMap<Uuid, TrackRepair>,
    vias: BTreeMap<Uuid, ViaRepair>,
}

pub(crate) fn generate_native_project_standards_repair_proposals(
    root: &Path,
) -> Result<NativeProjectStandardsRepairProposalReportView> {
    let check_run = query_native_project_check_run(root)?;
    let project = load_native_project_with_resolved_board(root)?;
    let mut model = ProjectResolver::new(root).resolve()?;
    let repairs = collect_process_aperture_repairs(&check_run)?;
    let required_mask_expansion = project
        .board
        .pad_expansion_setup
        .pad_to_mask_clearance_nm
        .max(0);
    let required_paste_reduction =
        (-project.board.pad_expansion_setup.pad_to_paste_clearance_nm).max(0);

    let mut views = Vec::new();
    for (pad_id, repair) in repairs {
        let Some(pad_value) = project.board.pads.get(&pad_id.to_string()).cloned() else {
            continue;
        };
        let mut pad: PlacedPad =
            serde_json::from_value(pad_value).context("failed to parse repair target pad")?;
        let mut changed = false;
        if repair
            .codes
            .iter()
            .any(|code| code.starts_with("pad_mask_"))
            && required_mask_expansion > 0
            && pad.solder_mask_margin_nm < required_mask_expansion
        {
            pad.solder_mask_margin_nm = required_mask_expansion;
            changed = true;
        }
        if repair
            .codes
            .iter()
            .any(|code| code.starts_with("pad_paste_"))
            && required_paste_reduction > 0
            && pad.solder_paste_margin_nm > -required_paste_reduction
        {
            pad.solder_paste_margin_nm = -required_paste_reduction;
            changed = true;
        }
        if !changed {
            continue;
        }

        let finding_fingerprints = repair.finding_fingerprints.into_iter().collect::<Vec<_>>();
        let codes = repair.codes.into_iter().collect::<Vec<_>>();
        let proposal_id = Uuid::new_v5(
            &model.project.project_id,
            standards_repair_key("process_aperture", pad_id, &codes).as_bytes(),
        );
        let operation = Operation::SetBoardPad {
            pad_id,
            pad: serde_json::to_value(&pad).expect("native board pad serialization must succeed"),
        };
        let readiness = create_standards_repair_proposal(
            root,
            &mut model,
            proposal_id,
            "standards process-aperture repair proposal",
            format!("repair process-aperture standards findings for pad {pad_id}"),
            vec![operation],
            check_run.check_run_id,
            finding_fingerprints.clone(),
        )?;
        views.push(NativeProjectStandardsRepairProposalView {
            proposal_id,
            repair_kind: "process_aperture",
            affected_pad: Some(pad_id),
            affected_track: None,
            affected_via: None,
            affected_net_class: None,
            finding_fingerprints,
            codes,
            prepared_against: readiness.prepared_against,
            prepared_against_current_model: readiness.prepared_against_current_model,
            can_apply: readiness.can_apply,
            blocker_codes: readiness.blocker_codes,
            operations: 1,
        });
    }

    let dimension_repairs = collect_dimension_geometry_repairs(&project, &check_run)?;
    for (track_id, repair) in dimension_repairs.tracks {
        let Some(track_value) = project.board.tracks.get(&track_id.to_string()).cloned() else {
            continue;
        };
        let mut track: Track =
            serde_json::from_value(track_value).context("failed to parse repair target track")?;
        if track.width >= repair.required_width {
            continue;
        }
        track.width = repair.required_width;

        let finding_fingerprints = repair.finding_fingerprints.into_iter().collect::<Vec<_>>();
        let codes = repair.codes.into_iter().collect::<Vec<_>>();
        let proposal_id = Uuid::new_v5(
            &model.project.project_id,
            standards_repair_key("track_geometry", track_id, &codes).as_bytes(),
        );
        let operation = Operation::SetBoardTrack {
            track_id,
            track: serde_json::to_value(&track)
                .expect("native board track serialization must succeed"),
        };
        let readiness = create_standards_repair_proposal(
            root,
            &mut model,
            proposal_id,
            "standards track-width repair proposal",
            format!("repair track-width standards findings for track {track_id}"),
            vec![operation],
            check_run.check_run_id,
            finding_fingerprints.clone(),
        )?;
        views.push(NativeProjectStandardsRepairProposalView {
            proposal_id,
            repair_kind: "track_geometry",
            affected_pad: None,
            affected_track: Some(track_id),
            affected_via: None,
            affected_net_class: Some(repair.net_class_id),
            finding_fingerprints,
            codes,
            prepared_against: readiness.prepared_against,
            prepared_against_current_model: readiness.prepared_against_current_model,
            can_apply: readiness.can_apply,
            blocker_codes: readiness.blocker_codes,
            operations: 1,
        });
    }

    for (via_id, repair) in dimension_repairs.vias {
        let Some(via_value) = project.board.vias.get(&via_id.to_string()).cloned() else {
            continue;
        };
        let mut via: Via =
            serde_json::from_value(via_value).context("failed to parse repair target via")?;
        let original = via.clone();
        if let Some(drill) = repair.via_drill {
            if via.drill < drill {
                via.drill = drill;
            }
        }
        if let Some(annular) = repair.via_annular {
            let required_diameter = via.drill + (annular * 2);
            if via.diameter < required_diameter {
                via.diameter = required_diameter;
            }
        }
        if via == original {
            continue;
        }

        let finding_fingerprints = repair.finding_fingerprints.into_iter().collect::<Vec<_>>();
        let codes = repair.codes.into_iter().collect::<Vec<_>>();
        let proposal_id = Uuid::new_v5(
            &model.project.project_id,
            standards_repair_key("via_geometry", via_id, &codes).as_bytes(),
        );
        let operation = Operation::SetBoardVia {
            via_id,
            via: serde_json::to_value(&via).expect("native board via serialization must succeed"),
        };
        let readiness = create_standards_repair_proposal(
            root,
            &mut model,
            proposal_id,
            "standards via-geometry repair proposal",
            format!("repair via-geometry standards findings for via {via_id}"),
            vec![operation],
            check_run.check_run_id,
            finding_fingerprints.clone(),
        )?;
        views.push(NativeProjectStandardsRepairProposalView {
            proposal_id,
            repair_kind: "via_geometry",
            affected_pad: None,
            affected_track: None,
            affected_via: Some(via_id),
            affected_net_class: Some(repair.net_class_id),
            finding_fingerprints,
            codes,
            prepared_against: readiness.prepared_against,
            prepared_against_current_model: readiness.prepared_against_current_model,
            can_apply: readiness.can_apply,
            blocker_codes: readiness.blocker_codes,
            operations: 1,
        });
    }

    Ok(NativeProjectStandardsRepairProposalReportView {
        action: "generate_standards_repair_proposals",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        check_run_id: check_run.check_run_id,
        proposal_count: views.len(),
        proposals: views,
    })
}

fn standards_repair_key(repair_kind: &str, target_id: Uuid, codes: &[String]) -> String {
    format!(
        "datum-eda:standards-repair:{repair_kind}:{target_id}:{}",
        codes.join("|")
    )
}

#[derive(Debug, Clone)]
struct StandardsRepairProposalReadiness {
    prepared_against: String,
    prepared_against_current_model: bool,
    can_apply: bool,
    blocker_codes: Vec<String>,
}

fn create_standards_repair_proposal(
    root: &Path,
    model: &mut DesignModel,
    proposal_id: Uuid,
    reason: &'static str,
    rationale: String,
    operations: Vec<Operation>,
    check_run_id: Uuid,
    finding_fingerprints: Vec<String>,
) -> Result<StandardsRepairProposalReadiness> {
    validate_repair_fingerprints(&finding_fingerprints)?;
    if model.proposals.contains_key(&proposal_id) {
        return standards_repair_readiness(model, proposal_id);
    }
    let batch = OperationBatch {
        batch_id: Uuid::new_v5(&model.project.project_id, proposal_id.as_bytes()),
        expected_model_revision: Some(model.model_revision.clone()),
        provenance: CommitProvenance {
            actor: "datum-eda-cli".to_string(),
            source: CommitSource::Cli,
            reason: reason.to_string(),
        },
        operations,
    };
    create_draft_proposal_from_batch(
        model,
        root,
        ProposalCreateRequest {
            proposal_id: Some(proposal_id),
            batch,
            rationale,
            source: ProposalSource::Check,
            checks_run: vec![check_run_id],
            finding_fingerprints,
        },
    )?;
    standards_repair_readiness(model, proposal_id)
}

fn validate_repair_fingerprints(finding_fingerprints: &[String]) -> Result<()> {
    if finding_fingerprints.is_empty() {
        anyhow::bail!("standards repair proposal requires at least one finding fingerprint");
    }
    for fingerprint in finding_fingerprints {
        if !is_sha256_fingerprint(fingerprint) {
            anyhow::bail!(
                "standards repair proposal fingerprint `{fingerprint}` must be a sha256:<64 lowercase hex> value"
            );
        }
    }
    Ok(())
}

fn is_sha256_fingerprint(value: &str) -> bool {
    let Some(digest) = value.strip_prefix("sha256:") else {
        return false;
    };
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

fn standards_repair_readiness(
    model: &DesignModel,
    proposal_id: Uuid,
) -> Result<StandardsRepairProposalReadiness> {
    let validation = validate_proposal_apply(model, proposal_id)?;
    Ok(StandardsRepairProposalReadiness {
        prepared_against: model
            .proposals
            .get(&proposal_id)
            .map(|proposal| proposal.prepared_against.0.clone())
            .unwrap_or_else(|| model.model_revision.0.clone()),
        prepared_against_current_model: validation.prepared_against_current_model,
        can_apply: validation.can_apply,
        blocker_codes: validation
            .blockers
            .iter()
            .map(|blocker| blocker.code.clone())
            .collect(),
    })
}

fn collect_process_aperture_repairs(
    check_run: &super::NativeProjectCheckRunView,
) -> Result<BTreeMap<Uuid, PadRepair>> {
    let mut repairs = BTreeMap::<Uuid, PadRepair>::new();
    for finding in &check_run.findings {
        if finding.source != "drc" || !is_process_aperture_repair_code(&finding.code) {
            continue;
        }
        let Some(pad_id) = finding
            .payload
            .get("objects")
            .and_then(serde_json::Value::as_array)
            .and_then(|objects: &Vec<serde_json::Value>| objects.first())
            .and_then(serde_json::Value::as_str)
            .and_then(|value| Uuid::parse_str(value).ok())
        else {
            continue;
        };
        let entry = repairs.entry(pad_id).or_insert_with(|| PadRepair {
            finding_fingerprints: BTreeSet::new(),
            codes: BTreeSet::new(),
        });
        entry
            .finding_fingerprints
            .insert(finding.fingerprint.clone());
        entry.codes.insert(finding.code.clone());
    }
    Ok(repairs)
}

fn is_process_aperture_repair_code(code: &str) -> bool {
    matches!(
        code,
        "pad_mask_expansion_missing"
            | "pad_mask_expansion_below_rule"
            | "pad_paste_reduction_missing"
            | "pad_paste_reduction_below_rule"
    )
}

fn collect_dimension_geometry_repairs(
    project: &super::LoadedNativeProject,
    check_run: &super::NativeProjectCheckRunView,
) -> Result<DimensionGeometryRepairs> {
    let mut repairs = DimensionGeometryRepairs::default();
    for finding in &check_run.findings {
        if finding.source != "drc" || !is_dimension_rule_repair_code(&finding.code) {
            continue;
        }
        let Some(object_id) = first_finding_object_id(finding) else {
            continue;
        };
        match finding.code.as_str() {
            "track_width_below_min" => {
                let Some(track_value) = project.board.tracks.get(&object_id.to_string()).cloned()
                else {
                    continue;
                };
                let track: Track = serde_json::from_value(track_value)
                    .context("failed to parse track dimension repair target")?;
                let Some((net_class_id, net_class)) = net_class_for_net(project, track.net)? else {
                    continue;
                };
                let entry = repairs
                    .tracks
                    .entry(track.uuid)
                    .or_insert_with(|| TrackRepair {
                        finding_fingerprints: BTreeSet::new(),
                        codes: BTreeSet::new(),
                        required_width: net_class.track_width,
                        net_class_id,
                    });
                entry
                    .finding_fingerprints
                    .insert(finding.fingerprint.clone());
                entry.codes.insert(finding.code.clone());
                entry.required_width = entry.required_width.max(net_class.track_width);
            }
            "via_hole_out_of_range" | "via_annular_below_min" => {
                let Some(via_value) = project.board.vias.get(&object_id.to_string()).cloned()
                else {
                    continue;
                };
                let via: Via = serde_json::from_value(via_value)
                    .context("failed to parse via dimension repair target")?;
                let Some((net_class_id, net_class)) = net_class_for_net(project, via.net)? else {
                    continue;
                };
                let repair = repairs.vias.entry(via.uuid).or_insert_with(|| ViaRepair {
                    finding_fingerprints: BTreeSet::new(),
                    codes: BTreeSet::new(),
                    via_drill: None,
                    via_annular: None,
                    net_class_id,
                });
                repair
                    .finding_fingerprints
                    .insert(finding.fingerprint.clone());
                repair.codes.insert(finding.code.clone());
                if finding.code == "via_hole_out_of_range" {
                    repair.via_drill = Some(net_class.via_drill);
                }
                if finding.code == "via_annular_below_min" {
                    repair.via_annular = Some((net_class.via_diameter - net_class.via_drill) / 2);
                }
            }
            _ => {}
        }
    }
    Ok(repairs)
}

fn first_finding_object_id(finding: &NativeProjectCheckFindingView) -> Option<Uuid> {
    finding
        .payload
        .get("objects")
        .and_then(serde_json::Value::as_array)
        .and_then(|objects: &Vec<serde_json::Value>| objects.first())
        .and_then(serde_json::Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
}

fn net_class_for_net(
    project: &super::LoadedNativeProject,
    net_id: Uuid,
) -> Result<Option<(Uuid, NetClass)>> {
    let Some(net_value) = project.board.nets.get(&net_id.to_string()).cloned() else {
        return Ok(None);
    };
    let net: Net =
        serde_json::from_value(net_value).context("failed to parse repair target net")?;
    let Some(class_value) = project
        .board
        .net_classes
        .get(&net.class.to_string())
        .cloned()
    else {
        return Ok(None);
    };
    let net_class: NetClass =
        serde_json::from_value(class_value).context("failed to parse repair target net class")?;
    Ok(Some((net.class, net_class)))
}

fn is_dimension_rule_repair_code(code: &str) -> bool {
    matches!(
        code,
        "track_width_below_min" | "via_hole_out_of_range" | "via_annular_below_min"
    )
}
