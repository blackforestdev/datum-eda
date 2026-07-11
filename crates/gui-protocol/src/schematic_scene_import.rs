use anyhow::{Context, Result};
use eda_engine::board::BoardText;
use eda_engine::ir::geometry::Point;
use eda_engine::schematic::{
    Bus, BusEntry, HierarchicalPort, LabelKind, NetLabel, PlacedSymbol, PortDirection, Schematic,
    SchematicPrimitive, Sheet, SymbolPin,
};
use eda_engine::text::{
    TextFamilyId, TextFamilySource, TextHAlign, TextRenderIntent, TextStyleId, TextVAlign,
    default_stroke_width_nm,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::*;

// Per-net-role schematic layers (P2.2c colour fidelity). The projection no
// longer smuggles every graphic onto one board silk layer (`F.SilkS`, which
// forced a single off-white); each element carries a schematic-specific layer
// whose NAME (`Schematic.*`) the renderer maps to the prototype token colour
// (`docs/gui/prototypes/schematic-editor.html`). Ids sit in a private high band
// (`L20x`/`L21x`) that never collides with real KiCad board layers (0..=49).
// Graphic layers use the `L<int>` string form; text layers use the matching
// integer, because `BoardText.layer` (i32) is rendered to `layer_id` as
// `format!("L{int}")` — so both graphics and text resolve through the same
// name lookup.
const SCHEMATIC_WIRE_LAYER: &str = "L200";
const SCHEMATIC_SYMBOL_LAYER: &str = "L201";
const SCHEMATIC_JUNCTION_LAYER: &str = "L202";
const SCHEMATIC_NOCONNECT_LAYER: &str = "L203";
// P2.2e typed-object layers. Buses render gold (`--bus`), power flags/stacks grey
// (`--pwr`), global/hierarchical net-label tags blue (`--info`). Same private high
// band (`L20x`) as the P2.2c net-role layers, mapped to prototype tokens by the
// renderer's `schematic_layer_world_color`.
const SCHEMATIC_BUS_LAYER: &str = "L204";
const SCHEMATIC_POWER_LAYER: &str = "L205";
const SCHEMATIC_GLOBAL_LABEL_LAYER: &str = "L206";
const SCHEMATIC_ANNOTATION_LAYER: &str = "L214";
const SCHEMATIC_REFDES_TEXT_LAYER_INT: i32 = 210;
const SCHEMATIC_VALUE_TEXT_LAYER_INT: i32 = 211;
const SCHEMATIC_PIN_NAME_TEXT_LAYER_INT: i32 = 212;
const SCHEMATIC_PIN_NUMBER_TEXT_LAYER_INT: i32 = 213;
const SCHEMATIC_ANNOTATION_TEXT_LAYER_INT: i32 = 214;
// P2.2f removed the schematic sheet frame: there is no longer an `Edge.Cuts`
// (`L44`) padded-bounds outline primitive in the schematic projection. A proper
// title-block frame is future work.
const SCHEMATIC_STROKE_NM: i64 = 120_000;
const SYMBOL_HALF_WIDTH_NM: i64 = 1_600_000;
const SYMBOL_HALF_HEIGHT_NM: i64 = 900_000;
const JUNCTION_RADIUS_NM: i64 = 180_000;
// Symbol projection fidelity (P2.2b). All nm in the schematic's own coordinate
// space; pins carry absolute positions (rotation baked in at import), so the
// body is derived from the pin envelope and pin lines connect body edge ->
// terminal (where wires already meet).
const PIN_STROKE_NM: i64 = 100_000;
const PIN_TERMINAL_RADIUS_NM: i64 = 90_000;
const MIN_BODY_HALF_NM: i64 = 500_000;
const MIN_PIN_STUB_NM: i64 = 300_000;
const MAX_PIN_STUB_NM: i64 = 2_000_000;
const REFDES_HEIGHT_NM: i64 = 900_000;
const VALUE_HEIGHT_NM: i64 = 720_000;
const PIN_NAME_HEIGHT_NM: i64 = 520_000;
const PIN_NUMBER_HEIGHT_NM: i64 = 420_000;
const LABEL_TEXT_HEIGHT_NM: i64 = 640_000;
const SYMBOL_TEXT_GAP_NM: i64 = 320_000;
const PIN_NAME_INSET_NM: i64 = 260_000;
const PIN_NUMBER_OUTSET_NM: i64 = 180_000;
const LABEL_HALF_WIDTH_NM: i64 = 1_100_000;
const LABEL_HALF_HEIGHT_NM: i64 = 300_000;
const NOCONNECT_HALF_NM: i64 = 260_000;
const SHEET_INSTANCE_HALF_WIDTH_NM: i64 = 2_200_000;
const SHEET_INSTANCE_HALF_HEIGHT_NM: i64 = 1_400_000;
const SCHEMATIC_BOUNDS_PAD_NM: i64 = 5_000_000;
// P2.2e geometry (schematic nm). Buses are thicker than wires (prototype 3.2 vs
// 1.5 stroke). Bus entries with no imported size fall back to a 2.54mm 45° stub.
const BUS_STROKE_NM: i64 = 320_000;
const BUS_ENTRY_STROKE_NM: i64 = 150_000;
const BUS_ENTRY_DEFAULT_STUB_NM: i64 = 2_540_000;
// Power flag/stack geometry. Rail = stem + one bar; ground = stem + three
// shrinking bars. All derived from the symbol origin (power symbols skeleton-
// import with no pins), rail extending "up" (-y) and ground "down" (+y).
const POWER_STROKE_NM: i64 = 140_000;
const POWER_STEM_NM: i64 = 1_400_000;
const POWER_RAIL_BAR_HALF_NM: i64 = 900_000;
const POWER_GND_BAR0_HALF_NM: i64 = 900_000;
const POWER_GND_BAR1_HALF_NM: i64 = 560_000;
const POWER_GND_BAR2_HALF_NM: i64 = 240_000;
const POWER_GND_BAR_GAP_NM: i64 = 360_000;
const POWER_LABEL_HEIGHT_NM: i64 = 560_000;
const POWER_LABEL_GAP_NM: i64 = 300_000;
// Global/hierarchical label pentagon tag: a flat body with one pointed end at the
// net attach point (KiCad global-label shape). Body extends +x from the anchor.
const GLOBAL_LABEL_STROKE_NM: i64 = 130_000;
const GLOBAL_LABEL_HALF_HEIGHT_NM: i64 = 420_000;
const GLOBAL_LABEL_NOTCH_NM: i64 = 420_000;
const GLOBAL_LABEL_CHAR_NM: i64 = 560_000;
const GLOBAL_LABEL_PAD_NM: i64 = 500_000;

pub fn load_kicad_schematic_workspace_state(schematic_file: &Path) -> Result<ReviewWorkspaceState> {
    let source = schematic_file.canonicalize().with_context(|| {
        format!(
            "failed to resolve KiCad schematic {}",
            schematic_file.display()
        )
    })?;
    let (scene, schematic_path) = load_scene_from_kicad_schematic_import(&source)?;
    let request = LiveReviewRequest {
        project_root: source
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".")),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    };
    let mut state = ReviewWorkspaceState::new(scene, empty_route_review_payload(&request));
    state.backing = Some(WorkspaceBacking {
        request,
        board_path: schematic_path,
    });
    Ok(state)
}

pub(crate) fn load_scene_from_kicad_schematic_import(
    schematic_file: &Path,
) -> Result<(BoardReviewSceneV1, PathBuf)> {
    let import_started = std::time::Instant::now();
    let (schematic, report) = eda_engine::import::kicad::import_schematic_document(schematic_file)
        .map_err(|e| anyhow::anyhow!("import {}: {e}", schematic_file.display()))?;
    for warning in &report.warnings {
        eprintln!(
            "datum-import warning [{}]: {warning}",
            schematic_file.display()
        );
    }
    trace_protocol_timing(format!(
        "kicad schematic import {}ms warnings={}",
        import_started.elapsed().as_millis(),
        report.warnings.len()
    ));

    let root_sheet = root_sheet(&schematic).context("imported schematic has no root sheet")?;
    let project_name = schematic_file
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("schematic")
        .to_string();
    let project_uuid = schematic.uuid.to_string();
    let sheet_uuid = root_sheet.uuid.to_string();
    let inspect = ProjectInspectPayload {
        project_root: schematic_file
            .parent()
            .unwrap_or(Path::new("."))
            .display()
            .to_string(),
        project_name: project_name.clone(),
        project_uuid: project_uuid.clone(),
        board_uuid: sheet_uuid.clone(),
        board_path: schematic_file.display().to_string(),
    };

    let mut graphics = Vec::new();
    let mut points = Vec::new();
    let mut text = SchematicTextSink::default();
    push_root_sheet_graphics(root_sheet, &schematic, &mut graphics, &mut points, &mut text);
    let (board_texts, board_text_geometries, glyph_mesh_assets) = text.into_parts();
    // P2.2f: the schematic pane has NO sheet border (the prototype draws none), so
    // the projection no longer emits the gold `Edge.Cuts` padded-bounds frame. The
    // padded envelope still drives the fit-to-bounds camera, so we compute it and
    // set it as `scene.bounds` directly, but pass an EMPTY outline to the builder
    // so no frame primitive renders. A proper title-block frame is future work.
    let envelope = schematic_bounds(&points);
    let mut scene = build_board_review_scene(
        &inspect,
        OutlinePayload {
            vertices: Vec::new(),
            closed: false,
        },
        Vec::new(),
        Vec::new(),
        Vec::new(),
        ScenePadExpansionSetup::default(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        graphics,
        board_texts,
        board_text_geometries,
        glyph_mesh_assets,
        Vec::new(),
        Vec::new(),
        String::new(),
    );
    scene.bounds = envelope;
    scene.kind = "schematic_review_scene".to_string();
    scene.scene_id = format!("schematic-review-scene:{sheet_uuid}");
    scene.board_name = root_sheet.name.clone();
    scene.source_revision = format!("schematic:{project_uuid}:sheet:{sheet_uuid}");
    scene.layers = schematic_scene_layers();
    Ok((scene, schematic_file.to_path_buf()))
}

fn root_sheet(schematic: &Schematic) -> Option<&Sheet> {
    schematic
        .sheet_definitions
        .values()
        .min_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)))
        .and_then(|definition| schematic.sheets.get(&definition.root_sheet))
        .or_else(|| {
            schematic
                .sheets
                .values()
                .min_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)))
        })
}

/// Accumulates schematic annotation text (refdes/value, pin names/numbers, net
/// labels, port/sheet names) as real rendered glyph geometry. The projection
/// reuses the board text pipeline (`push_board_text_scene_primitives`) so the
/// world renderer draws the exact same glyph meshes it draws for board silk;
/// this stays projection-only — no renderer changes.
#[derive(Default)]
struct SchematicTextSink {
    texts: Vec<BoardTextPrimitive>,
    geometries: Vec<BoardTextGeometryPrimitive>,
    mesh_assets: BTreeMap<GlyphMeshHandlePrimitive, GlyphMeshAssetPrimitive>,
}

impl SchematicTextSink {
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        points: &mut Vec<PointNm>,
        object_uuid: Uuid,
        object_key: &str,
        content: &str,
        anchor: PointNm,
        height_nm: i64,
        h_align: TextHAlign,
        v_align: TextVAlign,
        layer_int: i32,
    ) {
        let content = content.trim();
        if content.is_empty() {
            return;
        }
        points.push(anchor);
        let board_text = BoardText {
            uuid: text_uuid(object_uuid, object_key),
            text: content.to_string(),
            position: Point {
                x: anchor.x,
                y: anchor.y,
            },
            rotation: 0,
            layer: layer_int,
            render_intent: TextRenderIntent::Annotation,
            family: TextFamilyId::default(),
            family_source: TextFamilySource::default(),
            style: TextStyleId::default(),
            height_nm,
            stroke_width_nm: default_stroke_width_nm(height_nm),
            h_align,
            v_align,
            mirrored: false,
            keep_upright: true,
            line_spacing_ratio_ppm: 1_000_000,
            italic: false,
            bold: false,
            style_class: None,
        };
        push_board_text_scene_primitives(
            &board_text,
            &mut self.texts,
            &mut self.geometries,
            &mut self.mesh_assets,
        );
    }

    fn into_parts(
        self,
    ) -> (
        Vec<BoardTextPrimitive>,
        Vec<BoardTextGeometryPrimitive>,
        Vec<GlyphMeshAssetPrimitive>,
    ) {
        (
            self.texts,
            self.geometries,
            self.mesh_assets.into_values().collect(),
        )
    }
}

/// Deterministic, collision-resistant text UUID derived from the owning object
/// and a per-text role key (e.g. `refdes`, `pin-name:3`).
fn text_uuid(object_uuid: Uuid, key: &str) -> Uuid {
    Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum:schematic-projection-text:{object_uuid}:{key}").as_bytes(),
    )
}

fn push_root_sheet_graphics(
    sheet: &Sheet,
    schematic: &Schematic,
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    text: &mut SchematicTextSink,
) {
    for wire in sorted_values(&sheet.wires) {
        push_line_graphic(
            graphics,
            points,
            format!("schematic-wire:{}", wire.uuid),
            wire.uuid,
            wire.from,
            wire.to,
            SCHEMATIC_STROKE_NM,
            SCHEMATIC_WIRE_LAYER,
        );
    }
    // P2.2e: buses (gold thick polylines) and their diagonal green entry stubs.
    for bus in sorted_values(&sheet.buses) {
        push_bus_graphic(graphics, points, bus);
    }
    for entry in sorted_values(&sheet.bus_entries) {
        push_bus_entry_graphic(graphics, points, entry);
    }
    for drawing in sorted_values(&sheet.drawings) {
        push_drawing_graphic(graphics, points, text, drawing);
    }
    for symbol in sorted_values(&sheet.symbols) {
        // P2.2e: power symbols (KiCad `power:*` lib_ids) project a rail flag or a
        // ground stack instead of a generic IEC box.
        if is_power_symbol(symbol) {
            push_power_symbol_graphics(graphics, points, text, symbol);
        } else {
            push_symbol_graphics(graphics, points, text, symbol);
        }
    }
    for label in sorted_values(&sheet.labels) {
        push_net_label_graphic(graphics, points, text, label);
    }
    for port in sorted_values(&sheet.ports) {
        push_hierarchical_port_graphic(graphics, points, text, port);
    }
    for junction in sorted_values(&sheet.junctions) {
        push_circle_graphic(
            graphics,
            points,
            format!("schematic-junction:{}", junction.uuid),
            junction.uuid,
            junction.position,
            JUNCTION_RADIUS_NM,
            SCHEMATIC_JUNCTION_LAYER,
        );
    }
    for noconnect in sorted_values(&sheet.noconnects) {
        push_cross_graphic(
            graphics,
            points,
            format!("schematic-noconnect:{}", noconnect.uuid),
            noconnect.uuid,
            noconnect.position,
            NOCONNECT_HALF_NM,
            SCHEMATIC_NOCONNECT_LAYER,
        );
    }
    for schematic_text in sorted_values(&sheet.texts) {
        // Free schematic text is annotation, not a boxed object: emit it as real
        // text at its own position rather than a bare labelled rect.
        text.push(
            points,
            schematic_text.uuid,
            "schematic-text",
            &schematic_text.text,
            point_nm(schematic_text.position),
            LABEL_TEXT_HEIGHT_NM,
            TextHAlign::Left,
            TextVAlign::Center,
            SCHEMATIC_ANNOTATION_TEXT_LAYER_INT,
        );
    }
    for instance in sorted_values(&schematic.sheet_instances) {
        if instance.parent_sheet != Some(sheet.uuid) {
            continue;
        }
        push_rect_graphic(
            graphics,
            points,
            text,
            format!("schematic-sheet-instance:{}", instance.uuid),
            instance.uuid,
            instance.position,
            SHEET_INSTANCE_HALF_WIDTH_NM,
            SHEET_INSTANCE_HALF_HEIGHT_NM,
            Some(instance.name.clone()),
            SCHEMATIC_ANNOTATION_LAYER,
            SCHEMATIC_ANNOTATION_TEXT_LAYER_INT,
        );
    }
}

/// Projects one placed symbol at full fidelity: an IEC rectangular body sized
/// from the pin envelope, a pin line + terminal marker per pin, refdes/value
/// text near the body, and pin name/number text. Pins carry absolute positions
/// (rotation baked in at import), so the body is derived from where the pins sit
/// relative to the symbol origin and every pin line meets the terminal a wire
/// already connects to.
fn push_symbol_graphics(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    text: &mut SchematicTextSink,
    symbol: &PlacedSymbol,
) {
    let center = point_nm(symbol.position);
    let (half_w, half_h) = symbol_body_half_extents(center, &symbol.pins);

    // 1. IEC rectangular body (hollow, `--sym` grey stroke).
    push_rect_graphic(
        graphics,
        points,
        text,
        format!("schematic-symbol:{}", symbol.uuid),
        symbol.uuid,
        symbol.position,
        half_w,
        half_h,
        None,
        SCHEMATIC_SYMBOL_LAYER,
        SCHEMATIC_ANNOTATION_TEXT_LAYER_INT,
    );

    // 2. Pin lines + terminal markers, 4. pin name/number text.
    for (index, pin) in symbol.pins.iter().enumerate() {
        push_symbol_pin(graphics, points, text, symbol.uuid, center, half_w, half_h, index, pin);
    }

    // 3. Refdes above the body, value below it.
    let refdes = if symbol.reference.is_empty() {
        symbol.lib_id.clone().unwrap_or_default()
    } else {
        symbol.reference.clone()
    };
    text.push(
        points,
        symbol.uuid,
        "refdes",
        &refdes,
        PointNm {
            x: center.x,
            y: center.y - half_h - SYMBOL_TEXT_GAP_NM,
        },
        REFDES_HEIGHT_NM,
        TextHAlign::Center,
        TextVAlign::Bottom,
        SCHEMATIC_REFDES_TEXT_LAYER_INT,
    );
    text.push(
        points,
        symbol.uuid,
        "value",
        &symbol.value,
        PointNm {
            x: center.x,
            y: center.y + half_h + SYMBOL_TEXT_GAP_NM,
        },
        VALUE_HEIGHT_NM,
        TextHAlign::Center,
        TextVAlign::Top,
        SCHEMATIC_VALUE_TEXT_LAYER_INT,
    );
}

/// Derives the symbol body half-extents from the pin envelope. Pins that stick
/// out horizontally (|dx| >= |dy|) set the width inset by a pin stub; the body
/// must still enclose the perpendicular spread of the other pins so no pin
/// origin falls inside a face it does not belong to.
fn symbol_body_half_extents(center: PointNm, pins: &[SymbolPin]) -> (i64, i64) {
    if pins.is_empty() {
        return (SYMBOL_HALF_WIDTH_NM, SYMBOL_HALF_HEIGHT_NM);
    }
    let mut stick_x = 0_i64; // furthest horizontal pin reach
    let mut stick_y = 0_i64; // furthest vertical pin reach
    let mut enclose_x = 0_i64; // widest x spread among vertical pins
    let mut enclose_y = 0_i64; // tallest y spread among horizontal pins
    for pin in pins {
        let dx = (pin.position.x - center.x).abs();
        let dy = (pin.position.y - center.y).abs();
        if dx >= dy {
            stick_x = stick_x.max(dx);
            enclose_y = enclose_y.max(dy);
        } else {
            stick_y = stick_y.max(dy);
            enclose_x = enclose_x.max(dx);
        }
    }
    let pin_stub = (stick_x.max(stick_y) / 3).clamp(MIN_PIN_STUB_NM, MAX_PIN_STUB_NM);
    let half_w = if stick_x > 0 {
        (stick_x - pin_stub).max(MIN_BODY_HALF_NM).max(enclose_x)
    } else {
        MIN_BODY_HALF_NM.max(enclose_x)
    };
    let half_h = if stick_y > 0 {
        (stick_y - pin_stub).max(MIN_BODY_HALF_NM).max(enclose_y)
    } else {
        MIN_BODY_HALF_NM.max(enclose_y)
    };
    (half_w, half_h)
}

#[allow(clippy::too_many_arguments)]
fn push_symbol_pin(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    text: &mut SchematicTextSink,
    symbol_uuid: Uuid,
    center: PointNm,
    half_w: i64,
    half_h: i64,
    index: usize,
    pin: &SymbolPin,
) {
    let terminal = point_nm(pin.position);
    let dx = terminal.x - center.x;
    let dy = terminal.y - center.y;
    let horizontal = dx.abs() >= dy.abs();

    // Body-edge attach point for this pin, on the face it exits.
    let (edge, name_anchor, name_h, name_v, number_anchor, number_h, number_v) = if horizontal {
        let sign = if dx >= 0 { 1 } else { -1 };
        let edge = PointNm {
            x: center.x + sign * half_w,
            y: terminal.y,
        };
        // Name inside the body next to the edge; number outside on the stub.
        let (name_h, number_h) = if sign >= 0 {
            (TextHAlign::Right, TextHAlign::Left)
        } else {
            (TextHAlign::Left, TextHAlign::Right)
        };
        (
            edge,
            PointNm {
                x: edge.x - sign * PIN_NAME_INSET_NM,
                y: terminal.y,
            },
            name_h,
            TextVAlign::Center,
            PointNm {
                x: edge.x + sign * PIN_NUMBER_OUTSET_NM,
                y: terminal.y,
            },
            number_h,
            TextVAlign::Bottom,
        )
    } else {
        let sign = if dy >= 0 { 1 } else { -1 };
        let edge = PointNm {
            x: terminal.x,
            y: center.y + sign * half_h,
        };
        let (name_v, number_v) = if sign >= 0 {
            (TextVAlign::Top, TextVAlign::Bottom)
        } else {
            (TextVAlign::Bottom, TextVAlign::Top)
        };
        (
            edge,
            PointNm {
                x: terminal.x,
                y: edge.y - sign * PIN_NAME_INSET_NM,
            },
            TextHAlign::Center,
            name_v,
            PointNm {
                x: terminal.x,
                y: edge.y + sign * PIN_NUMBER_OUTSET_NM,
            },
            TextHAlign::Center,
            number_v,
        )
    };

    // Pin line body-edge -> terminal, only when the terminal actually sits
    // outside the body face (degenerate/inner pins get just a marker).
    let outside = if horizontal {
        (terminal.x - edge.x).signum() == dx.signum() && terminal.x != edge.x
    } else {
        (terminal.y - edge.y).signum() == dy.signum() && terminal.y != edge.y
    };
    if outside {
        let path = vec![edge, terminal];
        points.extend(path.iter().copied());
        graphics.push(BoardGraphicPrimitive {
            object_id: format!("schematic-symbol-pin:{}:{index}", symbol_uuid),
            object_kind: "schematic_graphic".to_string(),
            primitive_kind: "line".to_string(),
            source_object_uuid: pin.uuid.to_string(),
            layer_id: SCHEMATIC_SYMBOL_LAYER.to_string(),
            path,
            holes: Vec::new(),
            width_nm: Some(PIN_STROKE_NM),
        });
    }

    // Terminal marker at the wire attach point.
    push_circle_graphic(
        graphics,
        points,
        format!("schematic-symbol-pin-terminal:{}:{index}", symbol_uuid),
        pin.uuid,
        pin.position,
        PIN_TERMINAL_RADIUS_NM,
        SCHEMATIC_SYMBOL_LAYER,
    );

    // Pin name (inside the body) and pin number (outside on the stub).
    text.push(
        points,
        symbol_uuid,
        &format!("pin-name:{index}"),
        &pin.name,
        name_anchor,
        PIN_NAME_HEIGHT_NM,
        name_h,
        name_v,
        SCHEMATIC_PIN_NAME_TEXT_LAYER_INT,
    );
    text.push(
        points,
        symbol_uuid,
        &format!("pin-number:{index}"),
        &pin.number,
        number_anchor,
        PIN_NUMBER_HEIGHT_NM,
        number_h,
        number_v,
        SCHEMATIC_PIN_NUMBER_TEXT_LAYER_INT,
    );
}

/// P2.2e: a bus as a GOLD thick polyline (`Schematic.Bus` -> `--bus`). Geometry
/// comes from the engine `Bus.segments` (KiCad `(bus (pts ...))`); a bus authored
/// through the write path with no segments yet is skipped (nothing to draw).
fn push_bus_graphic(graphics: &mut Vec<BoardGraphicPrimitive>, points: &mut Vec<PointNm>, bus: &Bus) {
    if bus.segments.len() < 2 {
        return;
    }
    let path: Vec<PointNm> = bus.segments.iter().map(|p| point_nm(*p)).collect();
    points.extend(path.iter().copied());
    graphics.push(BoardGraphicPrimitive {
        object_id: format!("schematic-bus:{}", bus.uuid),
        object_kind: "schematic_graphic".to_string(),
        primitive_kind: "polyline".to_string(),
        source_object_uuid: bus.uuid.to_string(),
        layer_id: SCHEMATIC_BUS_LAYER.to_string(),
        path,
        holes: Vec::new(),
        width_nm: Some(BUS_STROKE_NM),
    });
}

/// P2.2e: a bus entry as the diagonal GREEN stub the prototype shows. Runs from
/// `position` to `position + size` (KiCad `(size dx dy)`); entries with no imported
/// size fall back to a 2.54mm 45° stub. Green so it reads as the member wire meeting
/// the bus, so it sits on the shared wire layer.
fn push_bus_entry_graphic(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    entry: &BusEntry,
) {
    let start = point_nm(entry.position);
    let (dx, dy) = if entry.size.x != 0 || entry.size.y != 0 {
        (entry.size.x, entry.size.y)
    } else {
        (BUS_ENTRY_DEFAULT_STUB_NM, -BUS_ENTRY_DEFAULT_STUB_NM)
    };
    let end = PointNm {
        x: start.x + dx,
        y: start.y + dy,
    };
    let path = vec![start, end];
    points.extend(path.iter().copied());
    graphics.push(BoardGraphicPrimitive {
        object_id: format!("schematic-bus-entry:{}", entry.uuid),
        object_kind: "schematic_graphic".to_string(),
        primitive_kind: "line".to_string(),
        source_object_uuid: entry.uuid.to_string(),
        layer_id: SCHEMATIC_WIRE_LAYER.to_string(),
        path,
        holes: Vec::new(),
        width_nm: Some(BUS_ENTRY_STROKE_NM),
    });
}

/// True for KiCad power symbols (`power:+3V3`, `power:GND`, ...), keyed on the
/// `power:` library prefix (case-insensitive).
fn is_power_symbol(symbol: &PlacedSymbol) -> bool {
    symbol
        .lib_id
        .as_deref()
        .map(|lib| lib.to_ascii_lowercase().starts_with("power:"))
        .unwrap_or(false)
}

/// Distinguishes a ground symbol from a positive rail by name: the symbol part of
/// the lib_id (after `:`) containing GND / GROUND / VSS / EARTH is a ground stack;
/// anything else (VCC, +3V3, VEE, PWR_FLAG, ...) is a rail flag.
fn is_ground_power_symbol(lib_id: &str) -> bool {
    let name = lib_id.rsplit(':').next().unwrap_or(lib_id).to_ascii_uppercase();
    ["GND", "GROUND", "VSS", "EARTH"]
        .iter()
        .any(|token| name.contains(token))
}

/// The net name a power symbol labels (its `Value`, e.g. `+3V3`; falling back to
/// the lib_id symbol name).
fn power_net_name(symbol: &PlacedSymbol) -> String {
    if !symbol.value.trim().is_empty() {
        return symbol.value.clone();
    }
    symbol
        .lib_id
        .as_deref()
        .and_then(|lib| lib.rsplit(':').next())
        .unwrap_or("")
        .to_string()
}

/// P2.2e: a power symbol as prototype GEOMETRY on `Schematic.Power` (`--pwr`)
/// instead of a generic IEC box: a rail flag (stem + one bar) for a positive rail,
/// or a ground stack (stem + three shrinking bars) for a ground. Power symbols
/// skeleton-import with no pins, so geometry is anchored at the symbol origin —
/// rail extending up (-y), ground down (+y).
fn push_power_symbol_graphics(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    text: &mut SchematicTextSink,
    symbol: &PlacedSymbol,
) {
    let origin = point_nm(symbol.position);
    let lib = symbol.lib_id.as_deref().unwrap_or("");
    let ground = is_ground_power_symbol(lib);
    // sign: +y (down) for ground, -y (up) for a rail flag.
    let sign = if ground { 1 } else { -1 };

    // Stem from the connection origin to the flag/stack.
    let stem_end = PointNm {
        x: origin.x,
        y: origin.y + sign * POWER_STEM_NM,
    };
    push_power_line(graphics, points, symbol.uuid, "stem", origin, stem_end);

    if ground {
        // Three shrinking horizontal bars beyond the stem.
        for (index, half) in [
            POWER_GND_BAR0_HALF_NM,
            POWER_GND_BAR1_HALF_NM,
            POWER_GND_BAR2_HALF_NM,
        ]
        .into_iter()
        .enumerate()
        {
            let y = stem_end.y + sign * (index as i64) * POWER_GND_BAR_GAP_NM;
            push_power_line(
                graphics,
                points,
                symbol.uuid,
                &format!("gnd-bar:{index}"),
                PointNm {
                    x: origin.x - half,
                    y,
                },
                PointNm {
                    x: origin.x + half,
                    y,
                },
            );
        }
    } else {
        // One horizontal bar at the top of the stem.
        push_power_line(
            graphics,
            points,
            symbol.uuid,
            "rail-bar",
            PointNm {
                x: origin.x - POWER_RAIL_BAR_HALF_NM,
                y: stem_end.y,
            },
            PointNm {
                x: origin.x + POWER_RAIL_BAR_HALF_NM,
                y: stem_end.y,
            },
        );
    }

    // Net name past the flag/stack (above a rail, below a ground stack).
    let far_y = if ground {
        stem_end.y + sign * (2 * POWER_GND_BAR_GAP_NM + POWER_LABEL_GAP_NM)
    } else {
        stem_end.y + sign * POWER_LABEL_GAP_NM
    };
    let (v_align, layer_int) = if ground {
        (TextVAlign::Top, SCHEMATIC_VALUE_TEXT_LAYER_INT)
    } else {
        (TextVAlign::Bottom, SCHEMATIC_VALUE_TEXT_LAYER_INT)
    };
    text.push(
        points,
        symbol.uuid,
        "power-net",
        &power_net_name(symbol),
        PointNm {
            x: origin.x,
            y: far_y,
        },
        POWER_LABEL_HEIGHT_NM,
        TextHAlign::Center,
        v_align,
        layer_int,
    );
}

fn push_power_line(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    symbol_uuid: Uuid,
    key: &str,
    from: PointNm,
    to: PointNm,
) {
    let path = vec![from, to];
    points.extend(path.iter().copied());
    graphics.push(BoardGraphicPrimitive {
        object_id: format!("schematic-power:{symbol_uuid}:{key}"),
        object_kind: "schematic_graphic".to_string(),
        primitive_kind: "line".to_string(),
        source_object_uuid: symbol_uuid.to_string(),
        layer_id: SCHEMATIC_POWER_LAYER.to_string(),
        path,
        holes: Vec::new(),
        width_nm: Some(POWER_STROKE_NM),
    });
}

/// P2.2e: a net label by kind. `Global`/`Hierarchical` become the pointed pentagon
/// tag on `Schematic.GlobalLabel` (`--info` blue); `Local`/`Power` keep the chip.
fn push_net_label_graphic(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    text: &mut SchematicTextSink,
    label: &NetLabel,
) {
    match label.kind {
        LabelKind::Global | LabelKind::Hierarchical => push_pentagon_tag(
            graphics,
            points,
            text,
            format!("schematic-label:{}", label.uuid),
            label.uuid,
            label.position,
            &label.name,
            1,
        ),
        LabelKind::Local | LabelKind::Power => push_rect_graphic(
            graphics,
            points,
            text,
            format!("schematic-label:{}", label.uuid),
            label.uuid,
            label.position,
            LABEL_HALF_WIDTH_NM,
            LABEL_HALF_HEIGHT_NM,
            Some(label.name.clone()),
            SCHEMATIC_ANNOTATION_LAYER,
            SCHEMATIC_ANNOTATION_TEXT_LAYER_INT,
        ),
    }
}

/// P2.2e: a hierarchical sheet port as a DIRECTIONAL pentagon tag on
/// `Schematic.GlobalLabel` (`--info`). The tag points away from the sheet by
/// direction (Output/Bidirectional extend +x; Input/Passive mirror to -x).
fn push_hierarchical_port_graphic(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    text: &mut SchematicTextSink,
    port: &HierarchicalPort,
) {
    let dir_sign = match port.direction {
        PortDirection::Input | PortDirection::Passive => -1,
        PortDirection::Output | PortDirection::Bidirectional => 1,
    };
    push_pentagon_tag(
        graphics,
        points,
        text,
        format!("schematic-port:{}", port.uuid),
        port.uuid,
        port.position,
        &port.name,
        dir_sign,
    );
}

/// A pointed pentagon net-label / port tag: a flat body with one pointed end at the
/// net attach point (`position`). `dir_sign` = +1 extends the body +x (tip on the
/// left), -1 mirrors it. Rendered as a closed polyline on `Schematic.GlobalLabel`
/// with the name centred in the body.
#[allow(clippy::too_many_arguments)]
fn push_pentagon_tag(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    text: &mut SchematicTextSink,
    object_id: String,
    source_uuid: Uuid,
    position: Point,
    name: &str,
    dir_sign: i64,
) {
    let anchor = point_nm(position);
    let half_h = GLOBAL_LABEL_HALF_HEIGHT_NM;
    let char_count = name.trim().chars().count().max(1) as i64;
    let body_w = GLOBAL_LABEL_NOTCH_NM + char_count * GLOBAL_LABEL_CHAR_NM + GLOBAL_LABEL_PAD_NM;
    let notch_x = anchor.x + dir_sign * GLOBAL_LABEL_NOTCH_NM;
    let body_x = anchor.x + dir_sign * body_w;
    let path = vec![
        anchor,
        PointNm {
            x: notch_x,
            y: anchor.y - half_h,
        },
        PointNm {
            x: body_x,
            y: anchor.y - half_h,
        },
        PointNm {
            x: body_x,
            y: anchor.y + half_h,
        },
        PointNm {
            x: notch_x,
            y: anchor.y + half_h,
        },
        anchor,
    ];
    points.extend(path.iter().copied());
    graphics.push(BoardGraphicPrimitive {
        object_id,
        object_kind: "schematic_graphic".to_string(),
        primitive_kind: "polyline".to_string(),
        source_object_uuid: source_uuid.to_string(),
        layer_id: SCHEMATIC_GLOBAL_LABEL_LAYER.to_string(),
        path,
        holes: Vec::new(),
        width_nm: Some(GLOBAL_LABEL_STROKE_NM),
    });
    text.push(
        points,
        source_uuid,
        "label-name",
        name,
        PointNm {
            x: (notch_x + body_x) / 2,
            y: anchor.y,
        },
        LABEL_TEXT_HEIGHT_NM,
        TextHAlign::Center,
        TextVAlign::Center,
        SCHEMATIC_ANNOTATION_TEXT_LAYER_INT,
    );
}

fn sorted_values<T>(map: &std::collections::HashMap<Uuid, T>) -> Vec<&T> {
    let mut entries = map.iter().collect::<Vec<_>>();
    entries.sort_by_key(|(uuid, _)| **uuid);
    entries.into_iter().map(|(_, value)| value).collect()
}

fn push_drawing_graphic(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    text: &mut SchematicTextSink,
    drawing: &SchematicPrimitive,
) {
    // Free graphic drawings are symbol-body geometry: colour them `--sym` grey.
    match drawing {
        SchematicPrimitive::Line { uuid, from, to } => push_line_graphic(
            graphics,
            points,
            format!("schematic-drawing-line:{uuid}"),
            *uuid,
            *from,
            *to,
            SCHEMATIC_STROKE_NM,
            SCHEMATIC_SYMBOL_LAYER,
        ),
        SchematicPrimitive::Rect { uuid, min, max } => {
            let center = Point {
                x: (min.x + max.x) / 2,
                y: (min.y + max.y) / 2,
            };
            push_rect_graphic(
                graphics,
                points,
                text,
                format!("schematic-drawing-rect:{uuid}"),
                *uuid,
                center,
                (max.x - min.x).abs() / 2,
                (max.y - min.y).abs() / 2,
                None,
                SCHEMATIC_SYMBOL_LAYER,
                SCHEMATIC_ANNOTATION_TEXT_LAYER_INT,
            );
        }
        SchematicPrimitive::Circle {
            uuid,
            center,
            radius,
        } => push_circle_graphic(
            graphics,
            points,
            format!("schematic-drawing-circle:{uuid}"),
            *uuid,
            *center,
            (*radius).max(SCHEMATIC_STROKE_NM),
            SCHEMATIC_SYMBOL_LAYER,
        ),
        SchematicPrimitive::Arc { uuid, arc } => {
            let path = arc_path_points(*arc);
            points.extend(path.iter().copied());
            graphics.push(BoardGraphicPrimitive {
                object_id: format!("schematic-drawing-arc:{uuid}"),
                object_kind: "schematic_graphic".to_string(),
                primitive_kind: "polyline".to_string(),
                source_object_uuid: uuid.to_string(),
                layer_id: SCHEMATIC_SYMBOL_LAYER.to_string(),
                path,
                holes: Vec::new(),
                width_nm: Some(SCHEMATIC_STROKE_NM),
            });
        }
    }
}

fn arc_path_points(arc: eda_engine::ir::geometry::Arc) -> Vec<PointNm> {
    let mut sweep = arc.end_angle - arc.start_angle;
    if sweep == 0 {
        sweep = 3600;
    }
    if sweep < 0 {
        sweep += 3600;
    }
    let steps = ((sweep.abs() / 150).max(4) as usize).min(48);
    (0..=steps)
        .map(|step| {
            let angle_tenths = arc.start_angle as f64 + (sweep as f64 * step as f64 / steps as f64);
            let radians = (angle_tenths / 10.0).to_radians();
            PointNm {
                x: arc.center.x + (radians.cos() * arc.radius as f64).round() as i64,
                y: arc.center.y + (radians.sin() * arc.radius as f64).round() as i64,
            }
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn push_line_graphic(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    object_id: String,
    source_uuid: Uuid,
    from: Point,
    to: Point,
    width_nm: i64,
    layer_id: &str,
) {
    let path = vec![point_nm(from), point_nm(to)];
    points.extend(path.iter().copied());
    graphics.push(BoardGraphicPrimitive {
        object_id,
        object_kind: "schematic_graphic".to_string(),
        primitive_kind: "line".to_string(),
        source_object_uuid: source_uuid.to_string(),
        layer_id: layer_id.to_string(),
        path,
        holes: Vec::new(),
        width_nm: Some(width_nm),
    });
}

// Scene import threads many primitive-geometry parameters.
#[allow(clippy::too_many_arguments)]
fn push_rect_graphic(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    text: &mut SchematicTextSink,
    object_id: String,
    source_uuid: Uuid,
    center: Point,
    half_width_nm: i64,
    half_height_nm: i64,
    label: Option<String>,
    layer_id: &str,
    label_text_layer_int: i32,
) {
    let center = point_nm(center);
    // Boxed schematic objects (net labels, hierarchical ports, sheet instances)
    // carry a name; render it centered in the box instead of discarding it.
    if let Some(label) = label {
        text.push(
            points,
            source_uuid,
            "rect-label",
            &label,
            center,
            LABEL_TEXT_HEIGHT_NM,
            TextHAlign::Center,
            TextVAlign::Center,
            label_text_layer_int,
        );
    }
    // IEC bodies (and boxed net-label / port / sheet-instance frames) render as
    // HOLLOW rects: a light stroke outline over the dark canvas, matching the
    // prototype's `#12141a` interior + `--sym` stroke. The world renderer FILLS
    // any `"polygon"` primitive with the layer color, which would read as an
    // opaque light block and bury the pin-name text inside it. Emitting the four
    // edges as a CLOSED `"polyline"` outline (renderer strokes, never fills) lets
    // the dark canvas show through so the interior text stays legible.
    let corners = [
        PointNm {
            x: center.x - half_width_nm,
            y: center.y - half_height_nm,
        },
        PointNm {
            x: center.x + half_width_nm,
            y: center.y - half_height_nm,
        },
        PointNm {
            x: center.x + half_width_nm,
            y: center.y + half_height_nm,
        },
        PointNm {
            x: center.x - half_width_nm,
            y: center.y + half_height_nm,
        },
    ];
    points.extend(corners.iter().copied());
    // Explicitly close the loop: the renderer only auto-closes `"polygon"`
    // primitives, so a polyline outline must repeat its first corner to draw the
    // fourth edge.
    let mut path = corners.to_vec();
    path.push(corners[0]);
    graphics.push(BoardGraphicPrimitive {
        object_id,
        object_kind: "schematic_graphic".to_string(),
        primitive_kind: "polyline".to_string(),
        source_object_uuid: source_uuid.to_string(),
        layer_id: layer_id.to_string(),
        path,
        holes: Vec::new(),
        width_nm: Some(SCHEMATIC_STROKE_NM),
    });
}

fn push_circle_graphic(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    object_id: String,
    source_uuid: Uuid,
    center: Point,
    radius_nm: i64,
    layer_id: &str,
) {
    let center = point_nm(center);
    let mut path = Vec::new();
    for step in 0..24 {
        let theta = (step as f64 / 24.0) * std::f64::consts::TAU;
        path.push(PointNm {
            x: center.x + (theta.cos() * radius_nm as f64).round() as i64,
            y: center.y + (theta.sin() * radius_nm as f64).round() as i64,
        });
    }
    points.extend(path.iter().copied());
    graphics.push(BoardGraphicPrimitive {
        object_id,
        object_kind: "schematic_graphic".to_string(),
        primitive_kind: "polygon".to_string(),
        source_object_uuid: source_uuid.to_string(),
        layer_id: layer_id.to_string(),
        path,
        holes: Vec::new(),
        width_nm: Some(SCHEMATIC_STROKE_NM),
    });
}

fn push_cross_graphic(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    object_id: String,
    source_uuid: Uuid,
    center: Point,
    half_nm: i64,
    layer_id: &str,
) {
    let center = point_nm(center);
    let a = PointNm {
        x: center.x - half_nm,
        y: center.y - half_nm,
    };
    let b = PointNm {
        x: center.x + half_nm,
        y: center.y + half_nm,
    };
    let c = PointNm {
        x: center.x - half_nm,
        y: center.y + half_nm,
    };
    let d = PointNm {
        x: center.x + half_nm,
        y: center.y - half_nm,
    };
    points.extend([a, b, c, d]);
    graphics.push(BoardGraphicPrimitive {
        object_id: format!("{object_id}:a"),
        object_kind: "schematic_graphic".to_string(),
        primitive_kind: "line".to_string(),
        source_object_uuid: source_uuid.to_string(),
        layer_id: layer_id.to_string(),
        path: vec![a, b],
        holes: Vec::new(),
        width_nm: Some(SCHEMATIC_STROKE_NM),
    });
    graphics.push(BoardGraphicPrimitive {
        object_id: format!("{object_id}:b"),
        object_kind: "schematic_graphic".to_string(),
        primitive_kind: "line".to_string(),
        source_object_uuid: source_uuid.to_string(),
        layer_id: layer_id.to_string(),
        path: vec![c, d],
        holes: Vec::new(),
        width_nm: Some(SCHEMATIC_STROKE_NM),
    });
}

/// The schematic scene's fit-to-bounds envelope: the full geometry extent padded
/// on every side. Since P2.2f dropped the rendered `Edge.Cuts` frame (which used
/// to carry this extent via `scene.outline`), this padded envelope is computed
/// directly and assigned to `scene.bounds` so the schematic pane still fits the
/// whole sheet without projecting a visible border.
fn schematic_bounds(points: &[PointNm]) -> SceneBounds {
    let (min_x, min_y, max_x, max_y) = if points.is_empty() {
        (
            -SCHEMATIC_BOUNDS_PAD_NM,
            -SCHEMATIC_BOUNDS_PAD_NM,
            SCHEMATIC_BOUNDS_PAD_NM,
            SCHEMATIC_BOUNDS_PAD_NM,
        )
    } else {
        let min_x = points.iter().map(|point| point.x).min().unwrap() - SCHEMATIC_BOUNDS_PAD_NM;
        let min_y = points.iter().map(|point| point.y).min().unwrap() - SCHEMATIC_BOUNDS_PAD_NM;
        let max_x = points.iter().map(|point| point.x).max().unwrap() + SCHEMATIC_BOUNDS_PAD_NM;
        let max_y = points.iter().map(|point| point.y).max().unwrap() + SCHEMATIC_BOUNDS_PAD_NM;
        (min_x, min_y, max_x, max_y)
    };
    SceneBounds {
        min_x,
        min_y,
        max_x,
        max_y,
    }
}

/// The schematic scene's layer table. Each per-net-role layer carries a
/// `Schematic.*` NAME the renderer's schematic colour path maps to a prototype
/// token (`docs/gui/prototypes/schematic-editor.html`). P2.2f dropped the former
/// `Edge.Cuts` frame layer — the schematic pane has no sheet border. All
/// schematic names resolve to the top-silk render stage (see
/// `render_stage_for_layer`) so they draw in the post-copper pass; within that
/// stage the projection's insertion order (wires -> symbols -> junctions ->
/// annotations) is the draw order.
fn schematic_scene_layers() -> Vec<SceneLayer> {
    let role = |layer_id: &str, name: &str, render_order: u32| SceneLayer {
        layer_id: layer_id.to_string(),
        name: name.to_string(),
        kind: "schematic".to_string(),
        render_order,
        visible_by_default: true,
    };
    vec![
        role(SCHEMATIC_WIRE_LAYER, "Schematic.Wire", 1),
        role(SCHEMATIC_BUS_LAYER, "Schematic.Bus", 2),
        role(SCHEMATIC_POWER_LAYER, "Schematic.Power", 2),
        role(SCHEMATIC_GLOBAL_LABEL_LAYER, "Schematic.GlobalLabel", 2),
        role(SCHEMATIC_SYMBOL_LAYER, "Schematic.Symbol", 2),
        role(SCHEMATIC_JUNCTION_LAYER, "Schematic.Junction", 3),
        role(SCHEMATIC_NOCONNECT_LAYER, "Schematic.NoConnect", 4),
        role(
            &layer_id_string(SCHEMATIC_REFDES_TEXT_LAYER_INT),
            "Schematic.RefDes",
            5,
        ),
        role(
            &layer_id_string(SCHEMATIC_VALUE_TEXT_LAYER_INT),
            "Schematic.Value",
            6,
        ),
        role(
            &layer_id_string(SCHEMATIC_PIN_NAME_TEXT_LAYER_INT),
            "Schematic.PinName",
            7,
        ),
        role(
            &layer_id_string(SCHEMATIC_PIN_NUMBER_TEXT_LAYER_INT),
            "Schematic.PinNumber",
            8,
        ),
        role(SCHEMATIC_ANNOTATION_LAYER, "Schematic.Annotation", 9),
    ]
}

/// The `layer_id` string a `BoardText.layer` integer projects to (mirrors the
/// gui-protocol `L{int}` convention), so a text role's registered `SceneLayer`
/// id matches the geometry the text pipeline emits.
fn layer_id_string(layer_int: i32) -> String {
    format!("L{layer_int}")
}

fn point_nm(point: Point) -> PointNm {
    PointNm {
        x: point.x,
        y: point.y,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_kicad_schematic_projects_to_visible_review_scene() {
        let schematic = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
        let state = load_kicad_schematic_workspace_state(&schematic)
            .expect("simple schematic should load as a review scene");

        assert_eq!(state.scene.kind, "schematic_review_scene");
        assert_eq!(state.scene.board_name, "Sub");
        assert!(
            state
                .scene
                .layers
                .iter()
                .any(|layer| layer.kind == "schematic" && layer.visible_by_default)
        );
        assert!(
            state
                .scene
                .board_graphics
                .iter()
                .any(|graphic| graphic.object_id.starts_with("schematic-wire:")),
            "schematic wires should be visible through review-scene graphics"
        );
        assert!(
            state
                .scene
                .board_graphics
                .iter()
                .any(|graphic| graphic.object_id.starts_with("schematic-symbol:")),
            "schematic symbols should project an IEC rectangular body"
        );
    }

    #[test]
    fn symbols_project_pins_terminals_and_annotation_text() {
        let schematic = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
        let state = load_kicad_schematic_workspace_state(&schematic)
            .expect("simple schematic should load as a review scene");
        let scene = &state.scene;

        // 2. Pin lines + terminal markers now project alongside the body, so the
        //    symbol reads as more than a bare box and wires meet real pins.
        assert!(
            scene
                .board_graphics
                .iter()
                .any(|g| g.object_id.starts_with("schematic-symbol-pin:")),
            "each symbol pin should project a pin line from body edge to terminal"
        );
        assert!(
            scene
                .board_graphics
                .iter()
                .any(|g| g.object_id.starts_with("schematic-symbol-pin-terminal:")),
            "each symbol pin should project a terminal marker at its wire attach point"
        );

        // 3/4. Refdes/value and pin name/number now render as real glyph text
        //      geometry (not discarded labels).
        assert!(
            scene
                .board_texts
                .iter()
                .any(|t| t.object_kind == "board_text"),
            "symbol annotation text (refdes/value/pin names) should project as real text"
        );
        assert!(
            !scene.board_text_geometries.is_empty(),
            "projected schematic text must carry renderable glyph geometry"
        );
        assert!(
            !scene.glyph_mesh_assets.is_empty(),
            "projected schematic text must carry glyph mesh assets for the world renderer"
        );
        // Text geometry must land on the per-role schematic text layers (P2.2c),
        // not the old single silk layer, so the renderer can colour refdes/value/
        // pin-name/pin-number distinctly.
        let text_layers: std::collections::BTreeSet<&str> = scene
            .board_text_geometries
            .iter()
            .map(|g| g.layer_id.as_str())
            .collect();
        assert!(
            !text_layers.contains("L44") && !text_layers.contains("L37"),
            "schematic text must no longer sit on the frame or the old F.SilkS layer"
        );
        assert!(
            text_layers.contains(&layer_id_string(SCHEMATIC_REFDES_TEXT_LAYER_INT).as_str())
                && text_layers
                    .contains(&layer_id_string(SCHEMATIC_VALUE_TEXT_LAYER_INT).as_str()),
            "refdes and value text must carry their own per-role layers, got {text_layers:?}"
        );
    }

    #[test]
    fn schematic_elements_carry_per_net_role_layers() {
        let schematic = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
        let state = load_kicad_schematic_workspace_state(&schematic)
            .expect("simple schematic should load as a review scene");
        let scene = &state.scene;

        // No schematic geometry may land on the old monochrome silk layer any
        // more; each net-role gets its own `Schematic.*` layer so the renderer
        // can resolve prototype token colours (green wires, grey symbols).
        assert!(
            scene
                .board_graphics
                .iter()
                .all(|g| g.layer_id != "L37"),
            "no schematic graphic may remain on the retired F.SilkS layer"
        );

        let layer_of = |prefix: &str| -> &str {
            scene
                .board_graphics
                .iter()
                .find(|g| g.object_id.starts_with(prefix))
                .unwrap_or_else(|| panic!("expected a {prefix} graphic"))
                .layer_id
                .as_str()
        };
        assert_eq!(
            layer_of("schematic-wire:"),
            SCHEMATIC_WIRE_LAYER,
            "wires must sit on the wire (green) layer"
        );
        assert_eq!(
            layer_of("schematic-symbol:"),
            SCHEMATIC_SYMBOL_LAYER,
            "symbol bodies must sit on the symbol (grey) layer"
        );

        // The scene layer table must register each role with its `Schematic.*`
        // name so the renderer's schematic colour path can key off it.
        let names: std::collections::BTreeSet<&str> =
            scene.layers.iter().map(|l| l.name.as_str()).collect();
        for expected in [
            "Schematic.Wire",
            "Schematic.Symbol",
            "Schematic.Junction",
            "Schematic.RefDes",
            "Schematic.Value",
            "Schematic.PinName",
            "Schematic.PinNumber",
        ] {
            assert!(
                names.contains(expected),
                "scene must register schematic role layer {expected}, got {names:?}"
            );
        }
        assert!(
            !names.contains("F.SilkS"),
            "the retired monochrome F.SilkS schematic layer must be gone"
        );
    }

    #[test]
    fn schematic_projection_emits_no_sheet_frame() {
        // P2.2f: the schematic pane has no sheet border. The projection must not
        // emit the former gold `Edge.Cuts` (`L44`) padded-bounds outline, and the
        // scene layer table must no longer register that frame layer — while the
        // fit-to-bounds envelope (scene.bounds) stays a real, non-degenerate rect
        // covering the padded sheet extent.
        let schematic = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
        let state = load_kicad_schematic_workspace_state(&schematic)
            .expect("simple schematic should load as a review scene");
        let scene = &state.scene;

        assert!(
            scene
                .outline
                .iter()
                .all(|outline| outline.path.is_empty()),
            "schematic scene must emit no rendered sheet-frame outline path"
        );
        assert!(
            scene.layers.iter().all(|layer| layer.name != "Edge.Cuts"),
            "schematic scene must no longer register the Edge.Cuts frame layer"
        );
        assert!(
            scene
                .board_graphics
                .iter()
                .all(|g| g.layer_id != "L44"),
            "no schematic graphic may sit on the retired L44 frame layer"
        );
        assert!(
            scene.bounds.max_x > scene.bounds.min_x && scene.bounds.max_y > scene.bounds.min_y,
            "the schematic fit-to-bounds envelope must remain a real rect, got {:?}",
            scene.bounds
        );
    }

    #[test]
    fn buses_project_gold_lines_and_diagonal_entries() {
        // P2.2e: a bus projects as a gold thick polyline on its own Schematic.Bus
        // layer, and its bus entries as diagonal green stubs. The bus-demo fixture
        // carries one bus (DATA) with two entries.
        let schematic = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../engine/testdata/import/kicad/bus-demo.kicad_sch");
        let state = load_kicad_schematic_workspace_state(&schematic)
            .expect("bus-demo schematic should load as a review scene");
        let scene = &state.scene;

        let bus = scene
            .board_graphics
            .iter()
            .find(|g| g.object_id.starts_with("schematic-bus:"))
            .expect("bus should project a gold polyline");
        assert_eq!(
            bus.layer_id, SCHEMATIC_BUS_LAYER,
            "bus must sit on the Schematic.Bus (gold) layer"
        );
        assert!(
            bus.path.len() >= 2 && bus.width_nm == Some(BUS_STROKE_NM),
            "bus must carry its imported geometry at the thick bus stroke, got {bus:?}"
        );

        let entries: Vec<_> = scene
            .board_graphics
            .iter()
            .filter(|g| g.object_id.starts_with("schematic-bus-entry:"))
            .collect();
        assert_eq!(entries.len(), 2, "both bus entries should project");
        for entry in &entries {
            assert_eq!(
                entry.layer_id, SCHEMATIC_WIRE_LAYER,
                "bus entry stubs read as green member wires"
            );
            // The stub is diagonal: start != end in both axes (non-zero imported size).
            assert!(
                entry.path.len() == 2
                    && entry.path[0].x != entry.path[1].x
                    && entry.path[0].y != entry.path[1].y,
                "bus entry must be a diagonal stub, got {entry:?}"
            );
        }

        let names: std::collections::BTreeSet<&str> =
            scene.layers.iter().map(|l| l.name.as_str()).collect();
        assert!(
            names.contains("Schematic.Bus"),
            "scene must register the Schematic.Bus layer, got {names:?}"
        );
    }

    #[test]
    fn global_labels_project_the_pentagon_tag() {
        // P2.2e: a Global net label projects as a pointed pentagon tag on the
        // Schematic.GlobalLabel (blue) layer, not a generic annotation rect. The
        // simple-demo fixture carries a global label (VCC) plus a local label (SCL)
        // that stays a chip.
        let schematic = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
        let state = load_kicad_schematic_workspace_state(&schematic)
            .expect("simple-demo schematic should load as a review scene");
        let scene = &state.scene;

        let pentagon = scene
            .board_graphics
            .iter()
            .find(|g| {
                g.object_id.starts_with("schematic-label:")
                    && g.layer_id == SCHEMATIC_GLOBAL_LABEL_LAYER
            })
            .expect("the global label should project onto the GlobalLabel layer");
        assert_eq!(
            pentagon.primitive_kind, "polyline",
            "the pentagon tag is a stroked (not filled) polyline"
        );
        assert_eq!(
            pentagon.path.len(),
            6,
            "the pentagon tag has five vertices plus the closing point, got {}",
            pentagon.path.len()
        );

        let names: std::collections::BTreeSet<&str> =
            scene.layers.iter().map(|l| l.name.as_str()).collect();
        assert!(
            names.contains("Schematic.GlobalLabel"),
            "scene must register the Schematic.GlobalLabel layer, got {names:?}"
        );
    }

    #[test]
    fn power_symbols_project_rail_flag_and_ground_stack_geometry() {
        // P2.2e: power symbols (`power:*` lib_ids) project a rail flag (stem + one
        // bar) or a ground stack (stem + three shrinking bars) on the Schematic.Power
        // layer, not a generic IEC box. No repo KiCad fixture carries power symbols,
        // so the projection is exercised directly over the engine Sheet model (this
        // constructs engine structs, not fabricated KiCad s-expressions).
        use eda_engine::ir::geometry::Point;
        use eda_engine::schematic::{
            HiddenPowerBehavior, PlacedSymbol, Schematic, Sheet, SymbolDisplayMode,
        };
        use std::collections::HashMap;

        let power_symbol = |uuid: Uuid, lib: &str, value: &str, x: i64| PlacedSymbol {
            uuid,
            part: None,
            entity: None,
            gate: None,
            lib_id: Some(lib.to_string()),
            reference: "#PWR".to_string(),
            value: value.to_string(),
            fields: Vec::new(),
            pins: Vec::new(),
            position: Point::new(x, 0),
            rotation: 0,
            mirrored: false,
            unit_selection: None,
            display_mode: SymbolDisplayMode::LibraryDefault,
            pin_overrides: Vec::new(),
            hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
        };
        let gnd_uuid = Uuid::from_u128(0x9001);
        let rail_uuid = Uuid::from_u128(0x9002);
        let mut symbols = HashMap::new();
        symbols.insert(gnd_uuid, power_symbol(gnd_uuid, "power:GND", "GND", 0));
        symbols.insert(rail_uuid, power_symbol(rail_uuid, "power:+3V3", "+3V3", 5_000_000));

        let sheet = Sheet {
            uuid: Uuid::from_u128(0x1),
            name: "Root".to_string(),
            frame: None,
            symbols,
            wires: HashMap::new(),
            junctions: HashMap::new(),
            labels: HashMap::new(),
            buses: HashMap::new(),
            bus_entries: HashMap::new(),
            ports: HashMap::new(),
            noconnects: HashMap::new(),
            texts: HashMap::new(),
            drawings: HashMap::new(),
        };
        let schematic = Schematic {
            uuid: Uuid::from_u128(0x2),
            sheets: HashMap::new(),
            sheet_definitions: HashMap::new(),
            sheet_instances: HashMap::new(),
            variants: HashMap::new(),
            waivers: Vec::new(),
        };

        let mut graphics = Vec::new();
        let mut points = Vec::new();
        let mut text = SchematicTextSink::default();
        push_root_sheet_graphics(&sheet, &schematic, &mut graphics, &mut points, &mut text);

        let power_lines: Vec<_> = graphics
            .iter()
            .filter(|g| g.object_id.starts_with("schematic-power:"))
            .collect();
        assert!(
            !power_lines.is_empty(),
            "power symbols must project power geometry, not a generic box"
        );
        assert!(
            power_lines
                .iter()
                .all(|g| g.layer_id == SCHEMATIC_POWER_LAYER),
            "all power geometry must sit on the Schematic.Power layer"
        );
        // No power symbol may fall through to the generic IEC symbol body.
        assert!(
            !graphics
                .iter()
                .any(|g| g.object_id.starts_with("schematic-symbol:")),
            "power symbols must not project a generic symbol body"
        );

        // Ground stack = stem + three bars (4 lines); rail flag = stem + one bar (2).
        let gnd_lines = power_lines
            .iter()
            .filter(|g| g.object_id.contains(&gnd_uuid.to_string()))
            .count();
        let rail_lines = power_lines
            .iter()
            .filter(|g| g.object_id.contains(&rail_uuid.to_string()))
            .count();
        assert_eq!(gnd_lines, 4, "ground = stem + three shrinking bars");
        assert_eq!(rail_lines, 2, "positive rail = stem + one bar");
    }
}
