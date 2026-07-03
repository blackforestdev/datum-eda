use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::api::native_write::board_routing::build_set_board_track;
use eda_engine::board::{Net, NetClass, Track};
use eda_engine::substrate::DesignModel;
use uuid::Uuid;

use super::command_project_standards_repairs::{
    NativeProjectStandardsRepairProposalView, create_standards_repair_proposal,
    standards_repair_proposal_id, standards_repair_provenance,
};

#[derive(Debug, Clone)]
struct CopperClearanceRepair {
    moved_track_id: Uuid,
    fixed_track_id: Uuid,
    repaired_track: Track,
    finding_fingerprint: String,
}

pub(super) fn append_copper_clearance_repair_proposals(
    root: &Path,
    model: &mut DesignModel,
    project: &super::LoadedNativeProject,
    check_run: &super::NativeProjectCheckRunView,
    views: &mut Vec<NativeProjectStandardsRepairProposalView>,
) -> Result<()> {
    for repair in collect_copper_clearance_repairs(project, check_run)? {
        let codes = vec!["clearance_copper".to_string()];
        let proposal_id =
            standards_repair_proposal_id(model, "copper_clearance", repair.moved_track_id, &codes);
        let prepared = build_set_board_track(
            model,
            standards_repair_provenance("standards copper-clearance repair proposal"),
            &repair.repaired_track,
        )?;
        let readiness = create_standards_repair_proposal(
            root,
            model,
            proposal_id,
            prepared,
            format!(
                "repair copper-clearance standards finding between tracks {} and {}",
                repair.moved_track_id, repair.fixed_track_id
            ),
            check_run.check_run_id,
            vec![repair.finding_fingerprint.clone()],
        )?;
        views.push(NativeProjectStandardsRepairProposalView {
            proposal_id,
            repair_kind: "copper_clearance",
            affected_pad: None,
            affected_text: None,
            affected_track: Some(repair.moved_track_id),
            affected_via: None,
            affected_net_class: None,
            affected_zone: None,
            finding_fingerprints: vec![repair.finding_fingerprint],
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

fn collect_copper_clearance_repairs(
    project: &super::LoadedNativeProject,
    check_run: &super::NativeProjectCheckRunView,
) -> Result<Vec<CopperClearanceRepair>> {
    let mut repairs = Vec::new();
    let mut moved_tracks = BTreeSet::<Uuid>::new();
    for finding in &check_run.findings {
        if finding.source != "drc" || finding.code != "clearance_copper" {
            continue;
        }
        let Some((fixed_track_id, moved_track_id)) = clearance_track_pair(project, finding) else {
            continue;
        };
        if !moved_tracks.insert(moved_track_id) {
            continue;
        }
        let Some(fixed_track) = project
            .board
            .tracks
            .get(&fixed_track_id.to_string())
            .cloned()
        else {
            continue;
        };
        let Some(moved_track) = project
            .board
            .tracks
            .get(&moved_track_id.to_string())
            .cloned()
        else {
            continue;
        };
        let fixed_track: Track = serde_json::from_value(fixed_track)
            .context("failed to parse fixed clearance repair track")?;
        let moved_track: Track = serde_json::from_value(moved_track)
            .context("failed to parse moved clearance repair track")?;
        let Some(repaired_track) =
            repaired_parallel_clearance_track(project, &fixed_track, moved_track)?
        else {
            continue;
        };
        repairs.push(CopperClearanceRepair {
            moved_track_id,
            fixed_track_id,
            repaired_track,
            finding_fingerprint: finding.fingerprint.clone(),
        });
    }
    Ok(repairs)
}

fn clearance_track_pair(
    project: &super::LoadedNativeProject,
    finding: &super::command_project_native_inspect::NativeProjectCheckFindingView,
) -> Option<(Uuid, Uuid)> {
    let track_ids = finding
        .payload
        .get("objects")
        .and_then(serde_json::Value::as_array)?
        .iter()
        .filter_map(|value| value.as_str())
        .filter_map(|value| Uuid::parse_str(value).ok())
        .filter(|uuid| project.board.tracks.contains_key(&uuid.to_string()))
        .collect::<Vec<_>>();
    if track_ids.len() != 2 {
        return None;
    }
    let mut sorted = track_ids;
    sorted.sort();
    Some((sorted[0], sorted[1]))
}

fn repaired_parallel_clearance_track(
    project: &super::LoadedNativeProject,
    fixed: &Track,
    mut moved: Track,
) -> Result<Option<Track>> {
    if fixed.layer != moved.layer || fixed.net == moved.net {
        return Ok(None);
    }
    let required = required_clearance_nm(project, fixed.net, moved.net)?;
    if required <= 0 {
        return Ok(None);
    }
    let edge_distance = parallel_track_edge_distance(fixed, &moved);
    if edge_distance >= required {
        return Ok(None);
    }
    let delta = required - edge_distance;
    if fixed.from.y == fixed.to.y
        && moved.from.y == moved.to.y
        && ranges_overlap(fixed.from.x, fixed.to.x, moved.from.x, moved.to.x)
    {
        let direction = if moved.from.y >= fixed.from.y { 1 } else { -1 };
        moved.from.y += delta * direction;
        moved.to.y += delta * direction;
        return Ok(Some(moved));
    }
    if fixed.from.x == fixed.to.x
        && moved.from.x == moved.to.x
        && ranges_overlap(fixed.from.y, fixed.to.y, moved.from.y, moved.to.y)
    {
        let direction = if moved.from.x >= fixed.from.x { 1 } else { -1 };
        moved.from.x += delta * direction;
        moved.to.x += delta * direction;
        return Ok(Some(moved));
    }
    Ok(None)
}

fn parallel_track_edge_distance(a: &Track, b: &Track) -> i64 {
    if a.from.y == a.to.y && b.from.y == b.to.y {
        return (a.from.y - b.from.y).abs() - ((a.width + b.width) / 2);
    }
    if a.from.x == a.to.x && b.from.x == b.to.x {
        return (a.from.x - b.from.x).abs() - ((a.width + b.width) / 2);
    }
    i64::MAX
}

fn ranges_overlap(a0: i64, a1: i64, b0: i64, b1: i64) -> bool {
    let (a_min, a_max) = if a0 <= a1 { (a0, a1) } else { (a1, a0) };
    let (b_min, b_max) = if b0 <= b1 { (b0, b1) } else { (b1, b0) };
    a_min <= b_max && b_min <= a_max
}

fn required_clearance_nm(
    project: &super::LoadedNativeProject,
    net_a: Uuid,
    net_b: Uuid,
) -> Result<i64> {
    Ok(net_class_clearance_nm(project, net_a)?.max(net_class_clearance_nm(project, net_b)?))
}

fn net_class_clearance_nm(project: &super::LoadedNativeProject, net_id: Uuid) -> Result<i64> {
    let Some(net_value) = project.board.nets.get(&net_id.to_string()).cloned() else {
        return Ok(0);
    };
    let net: Net =
        serde_json::from_value(net_value).context("failed to parse clearance repair net")?;
    let Some(class_value) = project
        .board
        .net_classes
        .get(&net.class.to_string())
        .cloned()
    else {
        return Ok(0);
    };
    let net_class: NetClass = serde_json::from_value(class_value)
        .context("failed to parse clearance repair net class")?;
    Ok(net_class.clearance)
}
