/// Load a KiCad .kicad_pcb board via the engine import path.
pub(super) fn load_scene_from_kicad_import(
    board_file: &Path,
) -> Result<(BoardReviewSceneV1, PathBuf)> {
    let import_started = std::time::Instant::now();
    let engine_started = std::time::Instant::now();
    let mut engine =
        eda_engine::api::Engine::new().map_err(|e| anyhow::anyhow!("engine init: {e}"))?;
    trace_protocol_timing(format!(
        "kicad engine init {}ms",
        engine_started.elapsed().as_millis()
    ));
    let engine_import_started = std::time::Instant::now();
    let import_report = engine
        .import(board_file)
        .map_err(|e| anyhow::anyhow!("import {}: {e}", board_file.display()))?;
    // Import warnings are fidelity signals (dropped objects, accounting
    // mismatches). They must surface, not vanish with the report.
    for warning in &import_report.warnings {
        eprintln!("datum-import warning [{}]: {warning}", board_file.display());
    }
    trace_protocol_timing(format!(
        "kicad engine import {}ms warnings={}",
        engine_import_started.elapsed().as_millis(),
        import_report.warnings.len()
    ));
    let board_borrow_started = std::time::Instant::now();
    let board = engine
        .board()
        .map_err(|e| anyhow::anyhow!("no board after import: {e}"))?;
    trace_protocol_timing(format!(
        "kicad board borrow {}ms",
        board_borrow_started.elapsed().as_millis()
    ));

    let board_uuid = board.uuid.to_string();
    let project_name = board_file
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "imported".to_string());

    let stackup_started = std::time::Instant::now();
    let stackup = engine
        .get_stackup()
        .map_err(|e| anyhow::anyhow!("stackup: {e}"))?;
    trace_protocol_timing(format!(
        "kicad stackup {}ms",
        stackup_started.elapsed().as_millis()
    ));
    let layer_name_map: std::collections::HashMap<i32, String> = stackup
        .layers
        .iter()
        .map(|l| (l.id, l.name.clone()))
        .collect();
    let _layer_name = |id: i32| -> String {
        layer_name_map
            .get(&id)
            .cloned()
            .unwrap_or_else(|| format!("L{}", id))
    };
    let components_started = std::time::Instant::now();
    let components = engine
        .get_components()
        .map_err(|e| anyhow::anyhow!("components: {e}"))?;
    trace_protocol_timing(format!(
        "kicad components {}ms count={}",
        components_started.elapsed().as_millis(),
        components.len()
    ));

    // Re-borrow board after the method calls above (they borrow &self temporarily).
    let board_reborrow_started = std::time::Instant::now();
    let board = engine.board().map_err(|e| anyhow::anyhow!("board: {e}"))?;
    trace_protocol_timing(format!(
        "kicad board reborrow {}ms",
        board_reborrow_started.elapsed().as_millis()
    ));

    let payload_started = std::time::Instant::now();
    let outline_vertices: Vec<PointNm> = board
        .outline
        .vertices
        .iter()
        .map(|p| PointNm { x: p.x, y: p.y })
        .collect();

    let outline_payload = OutlinePayload {
        vertices: outline_vertices,
        closed: !board.outline.vertices.is_empty(),
    };
    let pad_expansion_setup = ScenePadExpansionSetup {
        pad_to_mask_clearance_nm: board.pad_expansion_setup.pad_to_mask_clearance_nm,
        pad_to_paste_clearance_nm: board.pad_expansion_setup.pad_to_paste_clearance_nm,
        pad_to_paste_ratio_ppm: board.pad_expansion_setup.pad_to_paste_ratio_ppm,
        solder_mask_min_width_nm: board.pad_expansion_setup.solder_mask_min_width_nm,
    };

    let component_payloads: Vec<BoardComponentPayload> = components
        .iter()
        .map(|c| BoardComponentPayload {
            uuid: c.uuid.to_string(),
            reference: c.reference.clone(),
            value: c.value.clone(),
            position: PointNm {
                x: c.position.x,
                y: c.position.y,
            },
            rotation: c.rotation,
            layer: c.layer,
            locked: c.locked,
        })
        .collect();

    let pad_payloads: Vec<BoardPadPayload> = board
        .pads
        .values()
        .map(|p| {
            let shape_str = match p.shape {
                eda_engine::board::PadShape::Circle => "circle",
                eda_engine::board::PadShape::Rect => "rect",
                eda_engine::board::PadShape::Oval => "oval",
                eda_engine::board::PadShape::RoundRect => "roundrect",
            };
            BoardPadPayload {
                uuid: p.uuid.to_string(),
                package: p.package.to_string(),
                name: p.name.clone(),
                net: p.net.map(|n| n.to_string()),
                position: PointNm {
                    x: p.position.x,
                    y: p.position.y,
                },
                layer: p.layer,
                copper_layers: p.copper_layers.clone(),
                shape: shape_str.to_string(),
                diameter: p.diameter,
                width: p.width,
                height: p.height,
                roundrect_rratio_ppm: p.roundrect_rratio_ppm,
                mask_layers: p.mask_layers.clone(),
                paste_layers: p.paste_layers.clone(),
                solder_mask_margin_nm: p.solder_mask_margin_nm,
                solder_paste_margin_nm: p.solder_paste_margin_nm,
                solder_paste_margin_ratio_ppm: p.solder_paste_margin_ratio_ppm,
                drill: if p.drill > 0 { Some(p.drill) } else { None },
                rotation: p.rotation,
            }
        })
        .collect();

    let track_payloads: Vec<BoardTrackPayload> = board
        .tracks
        .values()
        .map(|t| BoardTrackPayload {
            uuid: t.uuid.to_string(),
            net: t.net.to_string(),
            from: PointNm {
                x: t.from.x,
                y: t.from.y,
            },
            to: PointNm {
                x: t.to.x,
                y: t.to.y,
            },
            width: t.width,
            layer: t.layer,
        })
        .collect();

    let via_payloads: Vec<BoardViaPayload> = board
        .vias
        .values()
        .map(|v| BoardViaPayload {
            uuid: v.uuid.to_string(),
            net: v.net.to_string(),
            position: PointNm {
                x: v.position.x,
                y: v.position.y,
            },
            drill: v.drill,
            diameter: v.diameter,
            from_layer: v.from_layer,
            to_layer: v.to_layer,
        })
        .collect();

    let zone_payloads: Vec<BoardZonePayload> = board
        .zones
        .values()
        .map(|z| BoardZonePayload {
            uuid: z.uuid.to_string(),
            net: z.net.to_string(),
            layer: z.layer,
            polygon: OutlinePayload {
                vertices: z
                    .polygon
                    .vertices
                    .iter()
                    .map(|p| PointNm { x: p.x, y: p.y })
                    .collect(),
                closed: true,
            },
        })
        .collect();
    let unrouted_primitives = unrouted_primitives_from_airwires(&board.unrouted());
    let net_display = net_display_from_imported_board(board);
    trace_protocol_timing(format!(
        "kicad payload build {}ms components={} pads={} tracks={} vias={} zones={}",
        payload_started.elapsed().as_millis(),
        component_payloads.len(),
        pad_payloads.len(),
        track_payloads.len(),
        via_payloads.len(),
        zone_payloads.len()
    ));

    let inspect = ProjectInspectPayload {
        project_root: board_file
            .parent()
            .unwrap_or(Path::new("."))
            .display()
            .to_string(),
        project_name,
        project_uuid: board_uuid.clone(),
        board_uuid,
        board_path: board_file.display().to_string(),
    };

    // --- Footprint graphics (silkscreen, fab, courtyard) + board-level
    // Edge.Cuts authored graphics (M7-SCN-007 Option B). Resolve Edge.Cuts to
    // its numeric id from the PCB's own layer table so the scene-level
    // `L{n}` key matches the visibility map for both the outline primitive
    // and the authored board_graphics primitives. KiCad 7 canonically uses
    // id 44; KiCad 9 may renumber — DOA2526 uses id 25 for Edge.Cuts.
    let (
        kicad_graphics,
        kicad_texts,
        mut imported_board_texts,
        mut imported_board_text_geometries,
        mut imported_glyph_mesh_assets,
        board_graphics,
        mut imported_gr_texts,
        mut imported_gr_text_geometries,
        imported_gr_glyph_mesh_assets,
        edge_cuts_layer_key,
    ) = {
        let direct_parse_started = std::time::Instant::now();
        let read_started = std::time::Instant::now();
        let contents = std::fs::read_to_string(board_file)
            .with_context(|| format!("failed to read {}", board_file.display()))?;
        trace_protocol_timing(format!(
            "kicad direct read {}ms bytes={}",
            read_started.elapsed().as_millis(),
            contents.len()
        ));
        let layer_table_started = std::time::Instant::now();
        let layer_table = kicad_parse_layer_table(&contents);
        trace_protocol_timing(format!(
            "kicad layer table parse {}ms layers={}",
            layer_table_started.elapsed().as_millis(),
            layer_table.len()
        ));
        let edge_cuts_key = layer_table
            .get("Edge.Cuts")
            .copied()
            .map(layer_id)
            .unwrap_or_else(|| layer_id(44));
        let footprint_parse_started = std::time::Instant::now();
        let (g, t, bt, btg, gma) =
            extract_kicad_footprint_graphics(&contents, &component_payloads, &layer_table);
        trace_protocol_timing(format!(
            "kicad footprint graphics/text parse {}ms graphics={} texts={} board_texts={} geometries={} glyph_assets={}",
            footprint_parse_started.elapsed().as_millis(),
            g.len(),
            t.len(),
            bt.len(),
            btg.len(),
            gma.len()
        ));
        let board_graphics_started = std::time::Instant::now();
        let bg = extract_kicad_board_graphics(&contents, &inspect.board_uuid, &layer_table);
        trace_protocol_timing(format!(
            "kicad board graphics parse {}ms graphics={}",
            board_graphics_started.elapsed().as_millis(),
            bg.len()
        ));
        let board_text_started = std::time::Instant::now();
        let (gr_texts, gr_geometries, gr_assets) =
            extract_kicad_board_texts(&contents, &layer_table);
        trace_protocol_timing(format!(
            "kicad board text parse {}ms texts={} geometries={} glyph_assets={}",
            board_text_started.elapsed().as_millis(),
            gr_texts.len(),
            gr_geometries.len(),
            gr_assets.len()
        ));
        trace_protocol_timing(format!(
            "kicad direct parse total {}ms",
            direct_parse_started.elapsed().as_millis()
        ));
        (
            g,
            t,
            bt,
            btg,
            gma,
            bg,
            gr_texts,
            gr_geometries,
            gr_assets,
            edge_cuts_key,
        )
    };
    let merge_started = std::time::Instant::now();
    imported_board_texts.append(&mut imported_gr_texts);
    imported_board_text_geometries.append(&mut imported_gr_text_geometries);
    merge_glyph_mesh_assets(
        &mut imported_glyph_mesh_assets,
        imported_gr_glyph_mesh_assets,
    );
    trace_protocol_timing(format!(
        "kicad text merge {}ms board_texts={} geometries={} glyph_assets={}",
        merge_started.elapsed().as_millis(),
        imported_board_texts.len(),
        imported_board_text_geometries.len(),
        imported_glyph_mesh_assets.len()
    ));
    let scene_build_started = std::time::Instant::now();
    let mut scene = build_board_review_scene(
        &inspect,
        outline_payload,
        component_payloads,
        kicad_graphics,
        kicad_texts,
        pad_expansion_setup,
        pad_payloads,
        track_payloads,
        via_payloads,
        zone_payloads,
        board_graphics,
        imported_board_texts,
        imported_board_text_geometries,
        imported_glyph_mesh_assets,
        unrouted_primitives,
        net_display,
        edge_cuts_layer_key,
    );
    trace_protocol_timing(format!(
        "kicad scene build {}ms",
        scene_build_started.elapsed().as_millis()
    ));
    // Replace auto-generated L0/L31 layers with real stackup names
    let layer_replace_started = std::time::Instant::now();
    scene.layers = stackup
        .layers
        .iter()
        .enumerate()
        .map(|(i, l)| SceneLayer {
            layer_id: layer_id(l.id),
            name: l.name.clone(),
            kind: match l.layer_type {
                eda_engine::board::StackupLayerType::Copper => "copper",
                eda_engine::board::StackupLayerType::Silkscreen => "silkscreen",
                eda_engine::board::StackupLayerType::SolderMask => "mask",
                eda_engine::board::StackupLayerType::Paste => "paste",
                eda_engine::board::StackupLayerType::Mechanical => "mechanical",
                eda_engine::board::StackupLayerType::Dielectric => "dielectric",
            }
            .to_string(),
            render_order: i as u32,
            visible_by_default: matches!(l.layer_type, eda_engine::board::StackupLayerType::Copper)
                || l.name.ends_with(".Cu")
                || l.name == "F.Cu"
                || l.name == "B.Cu"
                || l.name == "Edge.Cuts"
                || l.name == "F.SilkS",
        })
        .collect();
    trace_protocol_timing(format!(
        "kicad layer replace {}ms scene_layers={}",
        layer_replace_started.elapsed().as_millis(),
        scene.layers.len()
    ));
    trace_protocol_timing(format!(
        "kicad scene import total {}ms",
        import_started.elapsed().as_millis()
    ));
    Ok((scene, board_file.to_path_buf()))
}
