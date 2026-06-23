use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Result, bail};
use eda_engine::board::{StackupLayer, StackupLayerType};
use eda_engine::substrate::Operation;

use super::{
    LoadedNativeProject, NativeProjectBoardStackupMutationReportView,
    command_project_board_layout::commit_board_layout_operation, load_native_project,
    query_native_project_board_stackup,
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
    let project = load_native_project(root)?;
    let merged = merge_default_top_stackup(&project)?;
    let stackup = serde_json::json!({
        "layers": merged
            .into_iter()
            .map(|layer| {
                serde_json::to_value(layer)
                    .expect("native board stackup serialization must succeed")
            })
            .collect::<Vec<_>>(),
    });
    let layer_count = stackup["layers"].as_array().map_or(0, Vec::len);
    commit_board_layout_operation(
        root,
        "add default top stackup",
        Operation::SetBoardStackup {
            board_id: project.board.uuid,
            stackup,
        },
    )?;
    let project = load_native_project(root)?;
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
