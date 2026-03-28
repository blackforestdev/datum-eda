use std::path::Path;

use anyhow::Result;
use eda_engine::api::CheckReport;

use super::{
    DiagnosticsView, UnroutedView, build_native_project_board, load_native_project,
    summarize_native_board_checks,
};

pub(crate) fn query_native_project_board_diagnostics(root: &Path) -> Result<DiagnosticsView> {
    let project = load_native_project(root)?;
    Ok(DiagnosticsView::Board {
        diagnostics: build_native_project_board(&project)?.diagnostics(),
    })
}

pub(crate) fn query_native_project_board_unrouted(root: &Path) -> Result<UnroutedView> {
    let project = load_native_project(root)?;
    Ok(UnroutedView::Board {
        airwires: build_native_project_board(&project)?.unrouted(),
    })
}

pub(crate) fn query_native_project_board_check(root: &Path) -> Result<CheckReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    let diagnostics = board.diagnostics();
    Ok(CheckReport::Board {
        summary: summarize_native_board_checks(&diagnostics),
        diagnostics,
    })
}
