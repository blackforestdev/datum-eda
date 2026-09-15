use std::collections::BTreeMap;

use uuid::Uuid;

use crate::board::{Board, Zone};
use crate::rules::ast::RuleType;
use crate::schematic::CheckWaiver;
use crate::substrate::{ZoneFill, zone_fill_copper_projection_zones};

use super::{DrcReport, run_with_waivers};

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
    let mut projected = board.clone();
    let authored_zones = projected.zones.values().cloned().collect::<Vec<Zone>>();
    let (projected_zones, _) = zone_fill_copper_projection_zones(&authored_zones, zone_fills);
    projected.zones = projected_zones
        .into_iter()
        .map(|zone| (zone.uuid, zone))
        .collect();
    run_with_waivers(&projected, selected_rules, waivers)
}
