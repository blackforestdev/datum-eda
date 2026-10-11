//! Ordinary native CLI project -> workspace -> canonical paint -> retained GPU capture.
use super::*;

#[test]
#[ignore = "requires DATUM_PROCESS_PROJECT ordinary CLI flow and matching EDA_CLI_BIN"]
fn process_native_ordinary_workspace_capture() {
    for (variable, preserved) in [
        ("DATUM_PROCESS_PROJECT", false),
        ("DATUM_PROCESS_PRESERVED_PROJECT", true),
    ] {
        capture_project(
            std::path::PathBuf::from(std::env::var(variable).unwrap()),
            preserved,
        );
    }
}

fn capture_project(root: std::path::PathBuf, preserved: bool) {
    use datum_gui_protocol::{LiveReviewRequest, load_board_editor_workspace_with_model};
    use eda_engine::substrate::{SourceShardKind, component_pad_source::component_pad_projection};
    let request = LiveReviewRequest {
        project_root: root.clone(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    };
    let before = project_bytes(&root);
    let (mut state, model) = load_board_editor_workspace_with_model(&request).unwrap();
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let capability = eda_engine::substrate::process_operations::capability_from_board(&board)
        .unwrap()
        .unwrap();
    let process_layers = [
        format!("L{}", capability.front.mask_layer.unwrap()),
        format!("L{}", capability.front.paste_layer.unwrap()),
        format!("L{}", capability.back.mask_layer.unwrap()),
        format!("L{}", capability.back.paste_layer.unwrap()),
    ];
    let prefix = if preserved {
        "ordinary-preserved-render"
    } else {
        "ordinary-render"
    };
    let canonical = if preserved {
        board["pads"]
            .as_object()
            .unwrap()
            .values()
            .map(|raw| {
                let mut process =
                    eda_engine::substrate::component_pad_source::process::preserved_placed_process(
                        raw,
                    )
                    .unwrap();
                process.source_capability = Some(capability.clone());
                (process.pad_id, process)
            })
            .collect()
    } else {
        component_pad_projection(&model, &board)
            .unwrap()
            .process_pads
    };
    assert_eq!(state.scene.pads.len(), if preserved { 2 } else { 8 });
    for pad in &state.scene.pads {
        let id = uuid::Uuid::parse_str(&pad.pad_uuid).unwrap();
        let process = pad
            .native_process
            .as_ref()
            .expect("ordinary capable Pad has process authority");
        assert_eq!(process, &canonical[&id]);
        for (kind, aperture) in [
            (PadProcessLayerKind::Mask, &process.effective.mask),
            (PadProcessLayerKind::Paste, &process.effective.paste),
        ] {
            let mut quads = Vec::new();
            paint_native_process(&mut quads, process, kind, [1.0, 0.0, 0.0]);
            assert_eq!(
                quads.is_empty(),
                matches!(
                    aperture,
                    datum_gui_protocol::EffectiveProcessAperture::Suppressed
                )
            );
            assert!(
                quads
                    .iter()
                    .flat_map(|q| q.points)
                    .all(|(x, y)| x.is_finite() && y.is_finite())
            );
        }
    }
    let receipt = root.parent().unwrap().join(format!("{prefix}-scene.json"));
    std::fs::write(
        &receipt,
        eda_engine::ir::serialization::to_json_deterministic(&state.scene).unwrap(),
    )
    .unwrap();
    // The ordinary board has no authored silk/mechanical/outline geometry. Verify
    // its actual combined process order, then supplement only the sorter with an Edge sentinel.
    for layer in &state.scene.layers {
        state.ui.filters.layer_visibility.insert(
            layer.layer_id.clone(),
            process_layers.contains(&layer.layer_id),
        );
    }
    let combined = RetainedScene::try_from_workspace_for_surface(&state, 1000, 700, 1.0).unwrap();
    let order: Vec<_> = combined
        .draw_commands
        .iter()
        .filter_map(|command| match command {
            RetainedDrawCommand::Quads {
                layer_id: Some(id), ..
            } if process_layers.contains(id) => Some(id.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        order,
        [
            process_layers[2].as_str(),
            process_layers[0].as_str(),
            process_layers[3].as_str(),
            process_layers[1].as_str()
        ]
    );
    check_native_sort_control(&state.scene, &process_layers);
    let mut renderer = crate::visual_capture::OffscreenRenderer::new(1000, 700).unwrap();
    for (kind, layer) in [
        ("mask", &process_layers[0]),
        ("paste", &process_layers[1]),
        ("back-mask", &process_layers[2]),
        ("back-paste", &process_layers[3]),
    ] {
        for entry in &state.scene.layers {
            state.ui.filters.layer_visibility.insert(
                entry.layer_id.clone(),
                entry.layer_id.as_str() == layer.as_str(),
            );
        }
        let retained =
            RetainedScene::try_from_workspace_for_surface(&state, 1000, 700, 1.0).unwrap();
        let scene_layer = state
            .scene
            .layers
            .iter()
            .find(|entry| entry.layer_id.as_str() == layer.as_str())
            .unwrap();
        assert!(!matches!(
            scene_layer.name.as_str(),
            "F.Mask" | "B.Mask" | "F.Paste" | "B.Paste"
        ));
        let paste = scene_layer.kind == "paste";
        let layer_number: i32 = layer.strip_prefix('L').unwrap().parse().unwrap();
        let mut visible_pads = 0;
        for pad in &state.scene.pads {
            let process = pad.native_process.as_ref().unwrap();
            let membership = if paste {
                &process.membership.paste_layers
            } else {
                &process.membership.mask_layers
            };
            let spans: Vec<_> = retained
                .presentation
                .quads
                .iter()
                .filter(|span| span.key.as_deref() == Some(pad.object_id.as_str()))
                .collect();
            if !membership.contains(&layer_number) {
                assert!(
                    spans.is_empty(),
                    "hidden/opposite/suppressed Pad {}",
                    pad.pad_uuid
                );
                continue;
            }
            visible_pads += 1;
            let stage = native_process_stage(process, scene_layer).unwrap();
            let mut reference = Vec::new();
            paint_native_process(
                &mut reference,
                process,
                if paste {
                    PadProcessLayerKind::Paste
                } else {
                    PadProcessLayerKind::Mask
                },
                native_process_color(stage),
            );
            let expected: Vec<_> = reference
                .iter()
                .flat_map(|quad| {
                    let vertices = gpu_data::quad_vertices(*quad);
                    vertices[..if quad.points[2] == quad.points[3] {
                        3
                    } else {
                        6
                    }]
                        .to_vec()
                })
                .collect();
            let actual: Vec<_> = spans
                .iter()
                .flat_map(|span| retained.world_vertices[span.range.clone()].iter().copied())
                .collect();
            assert!(
                !actual.is_empty(),
                "native process layer routed into retained painter"
            );
            assert_eq!(
                actual, expected,
                "actual retained vertices/material for {}",
                pad.pad_uuid
            );
        }
        assert_eq!(visible_pads, if preserved { 2 } else { 5 });
        assert!(retained.draw_commands.iter().any(|command| matches!(command,
            RetainedDrawCommand::Quads { layer_id: Some(id), range } if id == layer && !range.is_empty())));
        assert!(
            !retained
                .draw_commands
                .iter()
                .any(|command| matches!(command,
            RetainedDrawCommand::Quads { layer_id: Some(id), .. }
                if process_layers.contains(id) && id != layer))
        );
        let image = renderer.render_workspace(&state, None).unwrap();
        let path = root.parent().unwrap().join(format!("{prefix}-{kind}.png"));
        image.save(path).unwrap();
    }
    assert_eq!(
        project_bytes(&root),
        before,
        "workspace query/capture preserves authored sources and journal"
    );
}

fn project_bytes(
    root: &std::path::Path,
) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
    fn visit(
        root: &std::path::Path,
        at: &std::path::Path,
        files: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>,
    ) {
        for entry in std::fs::read_dir(at).unwrap() {
            let path = entry.unwrap().path();
            let relative = path.strip_prefix(root).unwrap();
            if relative.starts_with(".datum/stage") || relative.starts_with(".datum/lock") {
                continue;
            }
            if path.is_dir() {
                visit(root, &path, files);
            } else if path.is_file() {
                files.insert(relative.to_path_buf(), std::fs::read(path).unwrap());
            }
        }
    }
    let mut files = std::collections::BTreeMap::new();
    visit(root, root, &mut files);
    files
}

/// Comparator supplement over a real ordinary Pad source; the Edge sentinel is
/// not claimed as authored board geometry or a captured product state.
fn check_native_sort_control(scene: &BoardReviewSceneV1, layers: &[String; 4]) {
    let pad = scene
        .pads
        .iter()
        .find(|pad| pad.mask_layer_ids.len() == 2)
        .unwrap();
    let mut scene = scene.clone();
    scene.layers.push(datum_gui_protocol::SceneLayer {
        layer_id: "sort-edge-control".into(),
        name: "Edge.Cuts".into(),
        kind: "mechanical".into(),
        render_order: 100,
        visible_by_default: true,
    });
    let mut commands = vec![RetainedDrawCommand::Strokes {
        layer_id: Some("sort-edge-control".into()),
        range: 0..1,
    }];
    for (index, layer) in [&layers[1], &layers[0], &layers[3], &layers[2]]
        .into_iter()
        .enumerate()
    {
        commands.push(RetainedDrawCommand::Quads {
            layer_id: Some(layer.clone()),
            range: (index * 6) as u32..((index + 1) * 6) as u32,
        });
    }
    let spans = vec![crate::presentation_ranges::Span {
        key: Some(pad.object_id.clone()),
        role: crate::presentation_ranges::Role::Material,
        range: 0..4,
    }];
    scene_retained_access::sort_native_board_commands(&mut commands, &scene, &spans, |_| Ok(()))
        .unwrap();
    let ids: Vec<_> = commands
        .iter()
        .map(|command| match command {
            RetainedDrawCommand::Quads { layer_id, .. }
            | RetainedDrawCommand::Strokes { layer_id, .. } => layer_id.as_deref().unwrap(),
            _ => panic!("unexpected comparator control command"),
        })
        .collect();
    assert_eq!(
        ids,
        [
            layers[2].as_str(),
            layers[0].as_str(),
            layers[3].as_str(),
            layers[1].as_str(),
            "sort-edge-control"
        ]
    );
}
