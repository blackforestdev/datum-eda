use crate::*;
use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Result, bail};
use eda_engine::api::native_write::board_layout::build_set_board_stackup;
use eda_engine::board::{StackupLayer, StackupLayerType};

use super::layout::commit_board_layout_write;
use crate::{
    LoadedNativeProject, NativeProjectBoardStackupMutationReportView,
    load_native_project_with_resolved_board, query_native_project_board_stackup,
};

pub(crate) fn default_native_project_stackup() -> Vec<StackupLayer> {
    vec![
        StackupLayer::new(1, "Top Copper", StackupLayerType::Copper, 35_000),
        StackupLayer::new(2, "Top Mask", StackupLayerType::SolderMask, 10_000),
        StackupLayer::new(3, "Top Silk", StackupLayerType::Silkscreen, 10_000),
        StackupLayer::new(4, "Top Paste", StackupLayerType::Paste, 10_000),
        StackupLayer::new(41, "Mechanical 41", StackupLayerType::Mechanical, 0),
    ]
}

pub(crate) fn default_native_project_stackup_layers() -> Vec<serde_json::Value> {
    default_native_project_stackup()
        .into_iter()
        .map(|layer| {
            serde_json::to_value(layer).expect("native stackup layer serialization must succeed")
        })
        .collect()
}

pub(crate) fn add_native_project_default_top_stackup(
    root: &Path,
) -> Result<NativeProjectBoardStackupMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let merged = merge_default_top_stackup(&project)?;
    let layer_count = merged.len();
    commit_board_layout_write(root, "add default top stackup", |model, provenance| {
        build_set_board_stackup(model, provenance, project.board.uuid, &merged)
    })?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(NativeProjectBoardStackupMutationReportView {
        action: "add_default_top_stackup".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        layer_count,
    })
}

fn merge_default_top_stackup(project: &LoadedNativeProject) -> Result<Vec<StackupLayer>> {
    let defaults = default_native_project_stackup();
    let existing = query_native_project_board_stackup(&project.root)?;
    let mut merged = BTreeMap::new();
    for layer in existing {
        merged.insert(layer.id, layer);
    }

    for default in defaults {
        match merged.get(&default.id) {
            Some(existing_layer) => {
                if existing_layer != &default {
                    bail!(
                        "cannot add default top stackup: layer id {} already exists with conflicting definition",
                        default.id
                    );
                }
            }
            None => {
                merged.insert(default.id, default);
            }
        }
    }

    Ok(merged.into_values().collect())
}

// Phase 5: exec-layer dissolution — variant run() impls (the former
// command_exec destructure-and-forward glue, now inherent methods on the
// clap args structs).

impl ProjectSetBoardStackupArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, layers } = self;
        let stackup_layers = parse_native_stackup_layers(&layers)?;
        let report = set_native_project_board_stackup(&path, stackup_layers)?;
        let output = render_report(
            format,
            &report,
            render_native_project_board_stackup_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectAddDefaultTopStackupArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path } = self;
        let report = add_native_project_default_top_stackup(&path)?;
        let output = render_report(
            format,
            &report,
            render_native_project_board_stackup_mutation_text,
        );
        Ok((output, 0))
    }
}
