//! Existing layer-type rendering policy, with visible unresolved copper.
//! UnresolvedCopper is a painter bucket, never a front/back source identity.
pub(super) fn scene_layer_name<'a>(
    layer_id: &str,
    scene_layers: &'a [datum_gui_protocol::SceneLayer],
) -> Option<&'a str> {
    scene_layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .map(|layer| layer.name.as_str())
}

pub(super) fn render_stage_for_layer(
    layer_id: &str,
    scene_layers: &[datum_gui_protocol::SceneLayer],
) -> RenderStage {
    match scene_layer_name(layer_id, scene_layers).unwrap_or(layer_id) {
        "B.Cu" => RenderStage::BottomCopper,
        name if name.ends_with(".Cu") && name != "F.Cu" => RenderStage::InnerCopper,
        "F.Cu" => RenderStage::TopCopper,
        "B.Paste" => RenderStage::BottomPaste,
        "F.Paste" => RenderStage::TopPaste,
        "B.Mask" => RenderStage::BottomMask,
        "F.Mask" => RenderStage::TopMask,
        "B.SilkS" => RenderStage::BottomSilk,
        "F.SilkS" => RenderStage::TopSilk,
        // Schematic net-role layers (P2.2c) all draw in the top-silk pass so the
        // schematic pane renders in the post-copper walk; within the stage the
        // projection's insertion order is the draw order. Colour is resolved
        // separately by `schematic_layer_world_color`.
        name if name.starts_with("Schematic.") => RenderStage::TopSilk,
        "Edge.Cuts" => RenderStage::Edge,
        name if name.ends_with(".CrtYd") || name.ends_with(".Fab") => RenderStage::Mechanical,
        _ if scene_layers
            .iter()
            .any(|layer| layer.layer_id == layer_id && layer.kind == "copper") =>
        {
            RenderStage::UnresolvedCopper
        }
        _ => RenderStage::Other,
    }
}

#[path = "render_stage.rs"]
mod render_stage;
pub(super) use render_stage::{POST_COPPER_STAGES, RenderStage, render_stage_priority};

pub(super) fn scene_layer_stack_priority(
    layer_id: &str,
    scene_layers: &[datum_gui_protocol::SceneLayer],
) -> u32 {
    render_stage_priority(render_stage_for_layer(layer_id, scene_layers))
}

pub(super) fn graphic_render_stage(
    layer_id: Option<&str>,
    scene_layers: &[datum_gui_protocol::SceneLayer],
    default_stage: RenderStage,
) -> RenderStage {
    layer_id
        .map(|id| render_stage_for_layer(id, scene_layers))
        .unwrap_or(default_stage)
}

pub(super) fn copper_pass_priority_for_layer(
    layer_id: &str,
    scene_layers: &[datum_gui_protocol::SceneLayer],
) -> Option<u32> {
    match render_stage_for_layer(layer_id, scene_layers) {
        RenderStage::BottomCopper => Some(0),
        RenderStage::InnerCopper | RenderStage::UnresolvedCopper => Some(1),
        RenderStage::TopCopper => Some(2),
        _ => None,
    }
}

#[cfg(test)]
#[path = "layer_stages_tests.rs"]
mod tests;
