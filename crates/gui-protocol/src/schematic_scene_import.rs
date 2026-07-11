use anyhow::{Context, Result};
use eda_engine::board::BoardText;
use eda_engine::ir::geometry::Point;
use eda_engine::schematic::{PlacedSymbol, Schematic, SchematicPrimitive, Sheet, SymbolPin};
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
const SCHEMATIC_ANNOTATION_LAYER: &str = "L214";
const SCHEMATIC_REFDES_TEXT_LAYER_INT: i32 = 210;
const SCHEMATIC_VALUE_TEXT_LAYER_INT: i32 = 211;
const SCHEMATIC_PIN_NAME_TEXT_LAYER_INT: i32 = 212;
const SCHEMATIC_PIN_NUMBER_TEXT_LAYER_INT: i32 = 213;
const SCHEMATIC_ANNOTATION_TEXT_LAYER_INT: i32 = 214;
const SCHEMATIC_FRAME_LAYER: &str = "L44";
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
const PORT_HALF_WIDTH_NM: i64 = 1_300_000;
const PORT_HALF_HEIGHT_NM: i64 = 450_000;
const NOCONNECT_HALF_NM: i64 = 260_000;
const SHEET_INSTANCE_HALF_WIDTH_NM: i64 = 2_200_000;
const SHEET_INSTANCE_HALF_HEIGHT_NM: i64 = 1_400_000;
const SCHEMATIC_BOUNDS_PAD_NM: i64 = 5_000_000;

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
    let outline = schematic_outline(&points);
    let mut scene = build_board_review_scene(
        &inspect,
        outline,
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
        SCHEMATIC_FRAME_LAYER.to_string(),
    );
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
    for drawing in sorted_values(&sheet.drawings) {
        push_drawing_graphic(graphics, points, text, drawing);
    }
    for symbol in sorted_values(&sheet.symbols) {
        push_symbol_graphics(graphics, points, text, symbol);
    }
    for label in sorted_values(&sheet.labels) {
        push_rect_graphic(
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
        );
    }
    for port in sorted_values(&sheet.ports) {
        push_rect_graphic(
            graphics,
            points,
            text,
            format!("schematic-port:{}", port.uuid),
            port.uuid,
            port.position,
            PORT_HALF_WIDTH_NM,
            PORT_HALF_HEIGHT_NM,
            Some(port.name.clone()),
            SCHEMATIC_ANNOTATION_LAYER,
            SCHEMATIC_ANNOTATION_TEXT_LAYER_INT,
        );
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

fn schematic_outline(points: &[PointNm]) -> OutlinePayload {
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
    OutlinePayload {
        vertices: vec![
            PointNm { x: min_x, y: min_y },
            PointNm { x: max_x, y: min_y },
            PointNm { x: max_x, y: max_y },
            PointNm { x: min_x, y: max_y },
        ],
        closed: true,
    }
}

/// The schematic scene's layer table. Each per-net-role layer carries a
/// `Schematic.*` NAME the renderer's schematic colour path maps to a prototype
/// token (`docs/gui/prototypes/schematic-editor.html`); the frame keeps the
/// board `Edge.Cuts` identity (gold) until P2.2f removes it. All schematic
/// names resolve to the top-silk render stage (see `render_stage_for_layer`) so
/// they draw in the post-copper pass; within that stage the projection's
/// insertion order (wires -> symbols -> junctions -> annotations) is the draw
/// order.
fn schematic_scene_layers() -> Vec<SceneLayer> {
    let role = |layer_id: &str, name: &str, render_order: u32| SceneLayer {
        layer_id: layer_id.to_string(),
        name: name.to_string(),
        kind: "schematic".to_string(),
        render_order,
        visible_by_default: true,
    };
    vec![
        SceneLayer {
            layer_id: SCHEMATIC_FRAME_LAYER.to_string(),
            name: "Edge.Cuts".to_string(),
            kind: "mechanical".to_string(),
            render_order: 0,
            visible_by_default: true,
        },
        role(SCHEMATIC_WIRE_LAYER, "Schematic.Wire", 1),
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
            !text_layers.contains(SCHEMATIC_FRAME_LAYER)
                && !text_layers.contains("L37"),
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
}
