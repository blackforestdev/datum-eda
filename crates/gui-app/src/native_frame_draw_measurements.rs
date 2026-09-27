//! Decode actual per-region draw receipts; prepared ranges alone are not work.
use super::*;

fn rect(rect: datum_gui_render::RectPx) -> [u32; 4] {
    [
        rect.x.max(0.0).floor() as u32,
        rect.y.max(0.0).floor() as u32,
        rect.width.max(1.0).ceil() as u32,
        rect.height.max(1.0).ceil() as u32,
    ]
}
fn scissors(
    frame: &FrameObservation<'_>,
    mask: Option<u32>,
    clip: [u32; 4],
) -> Result<Vec<[u32; 4]>> {
    frame
        .renderer
        .encoded_region_scissors(mask.context("invalid or duplicate draw receipt")?, clip)
        .map(|scissor| scissor.context("draw receipt does not match current frame clipping"))
        .collect()
}

pub(super) fn world(frame: &FrameObservation<'_>) -> Result<Option<Vec<Value>>> {
    if frame.submitted_frame != Some(true) || frame.prepared.surface_passes().is_empty() {
        return Ok(None);
    }
    frame.renderer.encoded_world_admission().map(|pane| {
        let surface = frame.prepared.surface_passes().iter().find(|p| p.pane_id == pane.pane_id)
            .context("submitted world pane missing from prepared frame")?;
        let scissors = scissors(frame, pane.encoded_regions(), rect(surface.scene_viewport))?;
        let copies = scissors.len() as u64;
        let (mut vertices, mut strokes, mut triangles) = (0_u64, 0_u64, 0_u64);
        let ranges: Vec<_> = pane.ranges().map(|range| match range {
            GeometryAdmissionRange::Vertices(r) => {
                let n = u64::from(r.end-r.start); vertices += n; triangles += n/3;
                json!({"kind":"vertices","start":r.start,"end":r.end})
            }
            GeometryAdmissionRange::StrokeInstances(r) => {
                let n = u64::from(r.end-r.start); strokes += n; triangles += n*2;
                json!({"kind":"stroke_instances","start":r.start,"end":r.end})
            }
        }).collect();
        Ok(json!({"pane_id":pane.pane_id.0,"surface":format!("{:?}",pane.surface),
            "bundle_executions":copies,"scissors":scissors,
            "submitted_commands":ranges.len() as u64 * copies,"submitted_vertices":vertices*copies,
            "submitted_stroke_instances":strokes*copies,"submitted_triangles":triangles*copies,"ranges":ranges}))
    }).collect::<Result<Vec<_>>>().map(Some)
}

pub(super) fn grids(frame: &FrameObservation<'_>) -> Result<Option<Vec<Value>>> {
    let Some(grids) = frame.renderer.grid_geometry_admission() else {
        return Ok(None);
    };
    grids.iter().map(|g| {
        let scissors = scissors(frame, g.encoded_regions(), rect(g.viewport))?;
        let copies = scissors.len() as u64;
        let n = u64::from(g.vertices.end-g.vertices.start);
        Ok(json!({"pane_id":g.pane_id.0,"viewport":[g.viewport.x,g.viewport.y,g.viewport.width,g.viewport.height],
            "range":[g.vertices.start,g.vertices.end],"generated_vertices":g.generated_vertices,
            "prepared_vertices":n,"encoded":g.encoded(),"scissors":scissors,
            "submitted_commands":(frame.submitted_frame==Some(true)).then_some(copies),
            "submitted_vertices":(frame.submitted_frame==Some(true)).then_some(n*copies),
            "submitted_triangles":(frame.submitted_frame==Some(true)).then_some((n/3)*copies)}))
    }).collect::<Result<Vec<_>>>().map(Some)
}

pub(super) fn terminal(frame: &FrameObservation<'_>) -> Result<Option<Vec<Value>>> {
    if frame.submitted_frame != Some(true) {
        return Ok(None);
    }
    if frame.prepared.prepared_terminal_graphic_count() == 0 {
        return Ok(Some(Vec::new()));
    }
    frame.renderer.terminal_geometry_admission().map(|g| {
        let scissors = scissors(frame, g.encoded_regions, g.scissor)?;
        let copies = scissors.len() as u64;
        Ok(json!({"graphic_id":g.graphic_id,"foreground":g.foreground,"scissor":g.scissor,
            "scissors":scissors,"submitted_commands":copies,
            "submitted_vertices":u64::from(g.vertices)*copies,"submitted_triangles":u64::from(g.vertices/3)*copies,
            "payload_bytes":g.payload_bytes}))
    }).collect::<Result<Vec<_>>>().map(Some)
}
