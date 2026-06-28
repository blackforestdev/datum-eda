use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::board::BoardText;
use eda_engine::substrate::{DesignModel, Operation};
use uuid::Uuid;

use super::command_project_standards_repairs::{
    NativeProjectStandardsRepairProposalView, create_standards_repair_proposal,
    standards_repair_key,
};

#[derive(Debug, Clone)]
struct SilkClearanceRepair {
    finding_fingerprints: BTreeSet<String>,
    codes: BTreeSet<String>,
}

pub(super) fn append_silk_clearance_repair_proposals(
    root: &Path,
    model: &mut DesignModel,
    project: &super::LoadedNativeProject,
    check_run: &super::NativeProjectCheckRunView,
    views: &mut Vec<NativeProjectStandardsRepairProposalView>,
) -> Result<()> {
    let silk_repairs = collect_silk_clearance_repairs(project, check_run)?;
    for (text_id, repair) in silk_repairs {
        let Some(text_value) = project
            .board
            .texts
            .iter()
            .find(|value| {
                serde_json::from_value::<BoardText>((*value).clone())
                    .map(|text| text.uuid == text_id)
                    .unwrap_or(false)
            })
            .cloned()
        else {
            continue;
        };
        let mut text: BoardText = serde_json::from_value(text_value)
            .context("failed to parse repair target board text")?;
        text.position.y += silk_clearance_repair_offset_nm(&text);

        let finding_fingerprints = repair.finding_fingerprints.into_iter().collect::<Vec<_>>();
        let codes = repair.codes.into_iter().collect::<Vec<_>>();
        let proposal_id = Uuid::new_v5(
            &model.project.project_id,
            standards_repair_key("silk_clearance", text_id, &codes).as_bytes(),
        );
        let operation = Operation::SetBoardText {
            text_id,
            text: serde_json::to_value(&text)
                .expect("native board text serialization must succeed"),
        };
        let readiness = create_standards_repair_proposal(
            root,
            model,
            proposal_id,
            "standards silkscreen-clearance repair proposal",
            format!("repair silkscreen-clearance standards findings for board text {text_id}"),
            vec![operation],
            check_run.check_run_id,
            finding_fingerprints.clone(),
        )?;
        views.push(NativeProjectStandardsRepairProposalView {
            proposal_id,
            repair_kind: "silk_clearance",
            affected_pad: None,
            affected_text: Some(text_id),
            affected_track: None,
            affected_via: None,
            affected_net_class: None,
            affected_zone: None,
            finding_fingerprints,
            codes,
            prepared_against: readiness.prepared_against,
            prepared_against_current_model: readiness.prepared_against_current_model,
            can_apply: readiness.can_apply,
            blocker_codes: readiness.blocker_codes,
            operations: 1,
        });
    }
    Ok(())
}

fn collect_silk_clearance_repairs(
    project: &super::LoadedNativeProject,
    check_run: &super::NativeProjectCheckRunView,
) -> Result<BTreeMap<Uuid, SilkClearanceRepair>> {
    let board_text_ids = project
        .board
        .texts
        .iter()
        .filter_map(|value| serde_json::from_value::<BoardText>(value.clone()).ok())
        .map(|text| text.uuid)
        .collect::<BTreeSet<_>>();
    let mut repairs = BTreeMap::<Uuid, SilkClearanceRepair>::new();
    for finding in &check_run.findings {
        if finding.source != "drc" || finding.code != "silk_clearance_copper" {
            continue;
        }
        let Some(text_id) = finding
            .payload
            .get("objects")
            .and_then(serde_json::Value::as_array)
            .and_then(|objects| {
                objects.iter().find_map(|value| {
                    let id = value
                        .as_str()
                        .and_then(|value| Uuid::parse_str(value).ok())?;
                    board_text_ids.contains(&id).then_some(id)
                })
            })
        else {
            continue;
        };
        let entry = repairs
            .entry(text_id)
            .or_insert_with(|| SilkClearanceRepair {
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

fn silk_clearance_repair_offset_nm(text: &BoardText) -> i64 {
    text.height_nm
        .max(0)
        .saturating_add(text.stroke_width_nm.max(0))
        .saturating_add(500_000)
        .max(1_000_000)
}
