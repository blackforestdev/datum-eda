use std::path::Path;

use anyhow::Result;
use eda_engine::api::CheckReport;
use eda_engine::board::Board;
use eda_engine::drc::DrcReport;
use eda_engine::rules::ast::RuleType;

#[path = "zone_fill_projection.rs"]
mod zone_fill_projection;
use zone_fill_projection::zone_fill_copper_projection_zones;

use crate::{
    DiagnosticsView, UnroutedView, build_native_project_board, build_native_project_schematic,
    load_native_project_with_resolved_board_and_model, summarize_native_board_checks,
};

pub(crate) fn query_native_project_board_diagnostics(root: &Path) -> Result<DiagnosticsView> {
    Ok(DiagnosticsView::Board {
        diagnostics: build_native_project_projected_board(root)?.diagnostics(),
    })
}

pub(crate) fn query_native_project_board_unrouted(root: &Path) -> Result<UnroutedView> {
    Ok(UnroutedView::Board {
        airwires: build_native_project_projected_board(root)?.unrouted(),
    })
}

pub(crate) fn query_native_project_board_check(root: &Path) -> Result<CheckReport> {
    let board = build_native_project_projected_board(root)?;
    let diagnostics = board.diagnostics();
    Ok(CheckReport::Board {
        summary: summarize_native_board_checks(&diagnostics),
        diagnostics,
    })
}

pub(crate) fn query_native_project_drc_with_rules(
    root: &Path,
    rules: &[RuleType],
) -> Result<DrcReport> {
    let (project, model) = load_native_project_with_resolved_board_and_model(root)?;
    let board = build_native_project_board(&project)?;
    let schematic = build_native_project_schematic(&project)?;
    Ok(eda_engine::drc::run_with_zone_fills_and_waivers(
        &board,
        rules,
        &model.zone_fills,
        &schematic.waivers,
    ))
}

fn build_native_project_projected_board(root: &Path) -> Result<Board> {
    let (project, model) = load_native_project_with_resolved_board_and_model(root)?;
    let mut board = build_native_project_board(&project)?;
    let authored_zones = board.zones.values().cloned().collect::<Vec<_>>();
    let (projected_zones, _) =
        zone_fill_copper_projection_zones(&authored_zones, &model.zone_fills);
    board.zones = projected_zones
        .into_iter()
        .map(|zone| (zone.uuid, zone))
        .collect();
    Ok(board)
}
