use eda_engine::board::BoardText;
use eda_engine::ir::geometry::Point;
use eda_engine::text::{
    TextFamilyId, TextFamilySource, TextHAlign, TextRenderIntent, TextStyleId, TextVAlign,
    default_stroke_width_nm,
};
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::*;

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
pub(super) const SCHEMATIC_WIRE_LAYER: &str = "L200";
pub(super) const SCHEMATIC_SYMBOL_LAYER: &str = "L201";
pub(super) const SCHEMATIC_JUNCTION_LAYER: &str = "L202";
pub(super) const SCHEMATIC_NOCONNECT_LAYER: &str = "L203";
// P2.2e typed-object layers. Buses render gold (`--bus`), power flags/stacks grey
// (`--pwr`), global/hierarchical net-label tags blue (`--info`). Same private high
// band (`L20x`) as the P2.2c net-role layers, mapped to prototype tokens by the
// renderer's `schematic_layer_world_color`.
pub(super) const SCHEMATIC_BUS_LAYER: &str = "L204";
pub(super) const SCHEMATIC_POWER_LAYER: &str = "L205";
pub(super) const SCHEMATIC_GLOBAL_LABEL_LAYER: &str = "L206";
pub(super) const SCHEMATIC_ANNOTATION_LAYER: &str = "L214";
pub(super) const SCHEMATIC_REFDES_TEXT_LAYER_INT: i32 = 210;
pub(super) const SCHEMATIC_VALUE_TEXT_LAYER_INT: i32 = 211;
pub(super) const SCHEMATIC_PIN_NAME_TEXT_LAYER_INT: i32 = 212;
pub(super) const SCHEMATIC_PIN_NUMBER_TEXT_LAYER_INT: i32 = 213;
pub(super) const SCHEMATIC_ANNOTATION_TEXT_LAYER_INT: i32 = 214;
// P2.2f removed the schematic sheet frame: there is no longer an `Edge.Cuts`
// (`L44`) padded-bounds outline primitive in the schematic projection. A proper
// title-block frame is future work.
pub(super) const SCHEMATIC_STROKE_NM: i64 = 120_000;
pub(super) const SYMBOL_HALF_WIDTH_NM: i64 = 1_600_000;
pub(super) const SYMBOL_HALF_HEIGHT_NM: i64 = 900_000;
pub(super) const JUNCTION_RADIUS_NM: i64 = 180_000;
// Symbol projection fidelity (P2.2b). All nm in the schematic's own coordinate
// space; pins carry absolute positions (rotation baked in at import), so the
// body is derived from the pin envelope and pin lines connect body edge ->
// terminal (where wires already meet).
pub(super) const PIN_STROKE_NM: i64 = 100_000;
pub(super) const PIN_TERMINAL_RADIUS_NM: i64 = 90_000;
pub(super) const MIN_BODY_HALF_NM: i64 = 500_000;
pub(super) const MIN_PIN_STUB_NM: i64 = 300_000;
pub(super) const MAX_PIN_STUB_NM: i64 = 2_000_000;
pub(super) const REFDES_HEIGHT_NM: i64 = 900_000;
pub(super) const VALUE_HEIGHT_NM: i64 = 720_000;
pub(super) const PIN_NAME_HEIGHT_NM: i64 = 520_000;
pub(super) const PIN_NUMBER_HEIGHT_NM: i64 = 420_000;
pub(super) const LABEL_TEXT_HEIGHT_NM: i64 = 640_000;
pub(super) const SYMBOL_TEXT_GAP_NM: i64 = 320_000;
pub(super) const PIN_NAME_INSET_NM: i64 = 260_000;
pub(super) const PIN_NUMBER_OUTSET_NM: i64 = 180_000;
pub(super) const LABEL_HALF_WIDTH_NM: i64 = 1_100_000;
pub(super) const LABEL_HALF_HEIGHT_NM: i64 = 300_000;
pub(super) const NOCONNECT_HALF_NM: i64 = 260_000;
pub(super) const SHEET_INSTANCE_HALF_WIDTH_NM: i64 = 2_200_000;
pub(super) const SHEET_INSTANCE_HALF_HEIGHT_NM: i64 = 1_400_000;
pub(super) const SCHEMATIC_BOUNDS_PAD_NM: i64 = 5_000_000;
// P2.2e geometry (schematic nm). Buses are thicker than wires (prototype 3.2 vs
// 1.5 stroke). Bus entries with no imported size fall back to a 2.54mm 45° stub.
pub(super) const BUS_STROKE_NM: i64 = 320_000;
pub(super) const BUS_ENTRY_STROKE_NM: i64 = 150_000;
pub(super) const BUS_ENTRY_DEFAULT_STUB_NM: i64 = 2_540_000;
// Power flag/stack geometry. Rail = stem + one bar; ground = stem + three
// shrinking bars. All derived from the symbol origin (power symbols skeleton-
// import with no pins), rail extending "up" (-y) and ground "down" (+y).
pub(super) const POWER_STROKE_NM: i64 = 140_000;
pub(super) const POWER_STEM_NM: i64 = 1_400_000;
pub(super) const POWER_RAIL_BAR_HALF_NM: i64 = 900_000;
pub(super) const POWER_GND_BAR0_HALF_NM: i64 = 900_000;
pub(super) const POWER_GND_BAR1_HALF_NM: i64 = 560_000;
pub(super) const POWER_GND_BAR2_HALF_NM: i64 = 240_000;
pub(super) const POWER_GND_BAR_GAP_NM: i64 = 360_000;
pub(super) const POWER_LABEL_HEIGHT_NM: i64 = 560_000;
pub(super) const POWER_LABEL_GAP_NM: i64 = 300_000;
// Global/hierarchical label pentagon tag: a flat body with one pointed end at the
// net attach point (KiCad global-label shape). Body extends +x from the anchor.
pub(super) const GLOBAL_LABEL_STROKE_NM: i64 = 130_000;
pub(super) const GLOBAL_LABEL_HALF_HEIGHT_NM: i64 = 420_000;
pub(super) const GLOBAL_LABEL_NOTCH_NM: i64 = 420_000;
pub(super) const GLOBAL_LABEL_CHAR_NM: i64 = 560_000;
pub(super) const GLOBAL_LABEL_PAD_NM: i64 = 500_000;

/// Accumulates schematic annotation text (refdes/value, pin names/numbers, net
/// labels, port/sheet names) as real rendered glyph geometry. The projection
/// reuses the board text pipeline (`push_board_text_scene_primitives`) so the
/// world renderer draws the exact same glyph meshes it draws for board silk;
/// this stays projection-only — no renderer changes.
#[derive(Default)]
pub(super) struct SchematicTextSink {
    texts: Vec<BoardTextPrimitive>,
    geometries: Vec<BoardTextGeometryPrimitive>,
    mesh_assets: BTreeMap<GlyphMeshHandlePrimitive, GlyphMeshAssetPrimitive>,
}

impl SchematicTextSink {
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn push(
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

    pub(super) fn into_parts(
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

pub(super) fn sorted_values<T>(map: &std::collections::HashMap<Uuid, T>) -> Vec<&T> {
    let mut entries = map.iter().collect::<Vec<_>>();
    entries.sort_by_key(|(uuid, _)| **uuid);
    entries.into_iter().map(|(_, value)| value).collect()
}

pub(super) fn arc_path_points(arc: eda_engine::ir::geometry::Arc) -> Vec<PointNm> {
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
pub(super) fn push_line_graphic(
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
pub(super) fn push_rect_graphic(
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

pub(super) fn push_circle_graphic(
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

pub(super) fn push_cross_graphic(
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
pub(super) fn schematic_bounds(points: &[PointNm]) -> SceneBounds {
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

pub(super) fn point_nm(point: Point) -> PointNm {
    PointNm {
        x: point.x,
        y: point.y,
    }
}
