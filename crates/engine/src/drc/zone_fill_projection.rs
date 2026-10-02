use std::collections::BTreeMap;

use uuid::Uuid;

use crate::board::{Board, Zone};
use crate::rules::ast::RuleType;
use crate::schematic::CheckWaiver;
use crate::substrate::{ZoneFill, zone_fill_copper_projection_zones};

use super::{DrcReport, run_with_clearance_override};

pub fn run_with_zone_fills(
    board: &Board,
    selected_rules: &[RuleType],
    zone_fills: &BTreeMap<Uuid, ZoneFill>,
) -> DrcReport {
    run_with_zone_fills_and_waivers(board, selected_rules, zone_fills, &[])
}

pub fn run_with_zone_fills_and_waivers(
    board: &Board,
    selected_rules: &[RuleType],
    zone_fills: &BTreeMap<Uuid, ZoneFill>,
    waivers: &[CheckWaiver],
) -> DrcReport {
    let clearance =
        if selected_rules.is_empty() || selected_rules.contains(&RuleType::ClearanceCopper) {
            // Raw fill maps do not prove current model/source basis. Keep authored
            // Zone peers visible to the nominal checker instead of erasing them.
            Some(super::checks::run_clearance_checks(board))
        } else {
            None
        };
    projected_report(board, selected_rules, zone_fills, waivers, clearance)
}

pub fn run_with_current_zone_fills_and_waivers(
    board: &Board,
    selected_rules: &[RuleType],
    model: &crate::substrate::DesignModel,
    waivers: &[CheckWaiver],
) -> DrcReport {
    use crate::board::nominal_geometry::GeometryError;
    use crate::substrate::ElectricalSelectionSnapshot;
    let need_nominal = (selected_rules.is_empty()
        || selected_rules.contains(&RuleType::ClearanceCopper))
        && board.tracks.values().any(|t| t.midpoint.is_some());
    let clearance = if need_nominal {
        let snapshot = ElectricalSelectionSnapshot::capture(model).ok();
        let certified = snapshot
            .as_ref()
            .and_then(|s| s.board_source().ok())
            .filter(|s| {
                s.uuid == board.uuid
                    && s.stackup == board.stackup
                    && same(&s.tracks, &board.tracks)
                    && same(&s.pads, &board.pads)
                    && same(&s.vias, &board.vias)
                    && same(&s.zones, &board.zones)
                    && same(&s.nets, &board.nets)
                    && same(&s.net_classes, &board.net_classes)
            })
            .is_some();
        let copper = board
            .zones
            .keys()
            .map(|id| {
                let value = if certified {
                    snapshot
                        .as_ref()
                        .and_then(|s| s.current_zone_fill(*id))
                        .map(|f| Ok(f.islands.clone()))
                        .unwrap_or(Err(GeometryError::UnverifiedFillBasis))
                } else {
                    Err(GeometryError::UnverifiedFillBasis)
                };
                (*id, value)
            })
            .collect();
        Some(super::checks::run_clearance_checks_with_zone_copper(
            board, &copper,
        ))
    } else {
        None
    };
    projected_report(board, selected_rules, &model.zone_fills, waivers, clearance)
}
fn same<T: PartialEq>(a: &BTreeMap<Uuid, T>, b: &std::collections::HashMap<Uuid, T>) -> bool {
    a.len() == b.len() && b.iter().all(|(id, value)| a.get(id) == Some(value))
}
fn projected_report(
    board: &Board,
    selected_rules: &[RuleType],
    zone_fills: &BTreeMap<Uuid, ZoneFill>,
    waivers: &[CheckWaiver],
    clearance: Option<Vec<super::DrcViolation>>,
) -> DrcReport {
    let mut projected = board.clone();
    let authored_zones = projected.zones.values().cloned().collect::<Vec<Zone>>();
    let (projected_zones, _) = zone_fill_copper_projection_zones(&authored_zones, zone_fills);
    projected.zones = projected_zones
        .into_iter()
        .map(|zone| (zone.uuid, zone))
        .collect();
    run_with_clearance_override(&projected, selected_rules, waivers, clearance)
}
