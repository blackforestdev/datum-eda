use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Result, bail};
use eda_engine::board::{StackupLayer, StackupLayerType};

use super::{
    LoadedNativeProject, NativeProjectBoardStackupMutationReportView, NativeStackup,
    load_native_project, query_native_project_board_stackup, write_canonical_json,
};

pub(crate) fn default_native_project_stackup() -> Vec<StackupLayer> {
    vec![
        StackupLayer {
            id: 1,
            name: "Top Copper".to_string(),
            layer_type: StackupLayerType::Copper,
            thickness_nm: 35_000,
        },
        StackupLayer {
            id: 2,
            name: "Top Mask".to_string(),
            layer_type: StackupLayerType::SolderMask,
            thickness_nm: 10_000,
        },
        StackupLayer {
            id: 3,
            name: "Top Silk".to_string(),
            layer_type: StackupLayerType::Silkscreen,
            thickness_nm: 10_000,
        },
        StackupLayer {
            id: 4,
            name: "Top Paste".to_string(),
            layer_type: StackupLayerType::Paste,
            thickness_nm: 10_000,
        },
        StackupLayer {
            id: 41,
            name: "Mechanical 41".to_string(),
            layer_type: StackupLayerType::Mechanical,
            thickness_nm: 0,
        },
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
    let mut project = load_native_project(root)?;
    let merged = merge_default_top_stackup(&project)?;
    project.board.stackup = NativeStackup {
        layers: merged
            .into_iter()
            .map(|layer| {
                serde_json::to_value(layer)
                    .expect("native board stackup serialization must succeed")
            })
            .collect(),
    };
    let layer_count = project.board.stackup.layers.len();
    write_canonical_json(&project.board_path, &project.board)?;
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
