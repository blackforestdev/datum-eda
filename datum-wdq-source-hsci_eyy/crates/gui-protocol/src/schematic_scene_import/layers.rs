use super::common::*;
use crate::*;

/// The schematic scene's layer table. Each per-net-role layer carries a
/// `Schematic.*` NAME the renderer's schematic colour path maps to a prototype
/// token (`docs/gui/prototypes/schematic-editor.html`). P2.2f dropped the former
/// `Edge.Cuts` frame layer — the schematic pane has no sheet border. All
/// schematic names resolve to the top-silk render stage (see
/// `render_stage_for_layer`) so they draw in the post-copper pass; within that
/// stage the projection's insertion order (wires -> symbols -> junctions ->
/// annotations) is the draw order.
pub(super) fn schematic_scene_layers() -> Vec<SceneLayer> {
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
pub(super) fn layer_id_string(layer_int: i32) -> String {
    format!("L{layer_int}")
}
