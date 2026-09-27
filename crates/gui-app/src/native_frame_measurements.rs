//! Per-attempt delivery in the existing resource trace; no polling or timer.
use super::*;
use resource_observation::{FrameObservation, GeometryAdmissionRange, TextAdmissionObservation};

pub(super) fn text_value(a: TextAdmissionObservation) -> Value {
    let group = |g: resource_observation::TextAdmissionGroup| {
        json!({
            "runs":g.runs,"layout_rows":g.layout_rows,"shaped_instances":g.shaped_instances,
            "unique_raster_keys":g.unique_raster_keys
        })
    };
    json!({"preparation_serial":a.preparation_serial,"font_owner_id":a.font_owner_id,
        "cache_revision":a.cache_revision,"workspace":group(a.workspace),"overlay":group(a.overlay),
        "union_unique_raster_keys":a.union_unique_raster_keys,"observer_scratch_bytes":a.scratch_bytes})
}
pub(super) fn record(frame: FrameObservation<'_>) -> Result<()> {
    if !ENABLED.load(Ordering::Acquire) {
        return Ok(());
    }
    let mut lock = WRITER.lock().unwrap_or_else(|e| e.into_inner());
    let writer = lock.as_mut().context("resource frame writer missing")?;
    let scope = writer.frame_scope.clone();
    let result = scope.with(|| writer.frame(frame));
    writer.frame_failed |= result.is_err();
    result
}
impl Writer {
    fn frame(&mut self, frame: FrameObservation<'_>) -> Result<()> {
        if self.frames >= self.frame_limit {
            self.line(json!({"phase":"invalid","reason":"frame capacity exhausted"}))?;
            anyhow::bail!("resource frame capacity exhausted");
        }
        self.frames += 1;
        let panes: Vec<_> = frame.prepared.geometry_admission(frame.board, frame.schematic).map(|pane| {
            let ranges: Vec<_> = pane.ranges().map(|r| match r {
                GeometryAdmissionRange::Vertices(r) => json!({"kind":"vertices","start":r.start,"end":r.end}),
                GeometryAdmissionRange::StrokeInstances(r) => json!({"kind":"stroke_instances","start":r.start,"end":r.end}),
            }).collect();
            let (counts, error) = match pane.counts() {
                Ok(c) => (json!({"retained_vertices":c.retained_vertices,"retained_stroke_instances":c.retained_stroke_instances,
                    "prepared_commands":c.prepared_commands,"prepared_vertices":c.prepared_vertices,
                    "prepared_stroke_instances":c.prepared_stroke_instances,"prepared_triangles":c.prepared_triangles}), None),
                Err(e) => (Value::Null, Some(e.to_string())),
            };
            json!({"pane_id":pane.pane_id.0,"surface":format!("{:?}",pane.surface),
                "viewport":[pane.viewport.x,pane.viewport.y,pane.viewport.width,pane.viewport.height],
                "source":pane.source.map(|s|json!({"scene_id":s.scene_id,"source_revision":s.source_revision})),
                "counts":counts,"error":error,"ranges":ranges})
        }).collect();
        let encoded_world = draw_measurements::world(&frame)?;
        let screen: Vec<_> = frame.prepared.screen_geometry_admission().into_iter().map(|g| {
            json!({"group":g.group,"prepared_vertices":g.vertices,"payload_bytes":g.payload_bytes})
        }).collect();
        let submitted_screen = if frame.submitted_frame == Some(true) {
            Some(frame.renderer.encoded_screen_geometry().context("screen draw observation unavailable")?.into_iter().flatten().map(|d| {
                json!({"group":d.group,"range":[0,d.vertices],"submitted_commands":d.draws,
                    "submitted_vertices":u64::from(d.vertices)*u64::from(d.draws),"submitted_triangles":u64::from(d.vertices/3)*u64::from(d.draws),"scissor":d.scissor,"scissors":d.scissors()})
            }).collect::<Vec<_>>())
        } else {
            None
        };
        let grids = draw_measurements::grids(&frame)?;
        let terminal = draw_measurements::terminal(&frame)?;
        let admission = frame
            .text_observation_attempted
            .then(|| frame.renderer.text_admission_observation())
            .flatten();
        let text_origins = admission.and_then(|_| frame.renderer.text_origin_admission()).map(|origins| {
            origins.iter().map(|o| {
                let origin = match o.origin {
                    resource_observation::TextOrigin::Host => json!({"kind":"host"}),
                    resource_observation::TextOrigin::Viewport(pane) => json!({"kind":"viewport","pane_id":pane.0}),
                    resource_observation::TextOrigin::TerminalLeaf(index) => json!({"kind":"terminal","leaf_index":index,
                        "session_id":frame.prepared.admission_sources().and_then(|s|s.terminal_sessions.get(index))}),
                };
                json!({"origin":origin,"overlay":o.overlay,"runs":o.counts.runs,"layout_rows":o.counts.layout_rows,
                    "shaped_instances":o.counts.shaped_instances,"unique_raster_keys":o.counts.unique_raster_keys})
            }).collect::<Vec<_>>()
        });
        let text = admission.map(text_value);
        let glyphs = admission.map(|a| {
            let counts = frame.renderer.glyph_preparation_counts();
            [a.workspace.runs, a.overlay.runs].into_iter().zip(counts).map(|(runs, c)| {
                if runs == 0 { return None; }
                c.map(|c| json!({"areas":c.areas,"layout_rows":c.layout_rows,"shaped_instances":c.shaped_instances,
                    "row_culled_instances":c.row_culled_instances,"clipped_instances":c.clipped_instances,
                    "without_raster_instances":c.without_raster_instances,"prepared_instances":c.prepared_instances,
                    "draw_batches":c.draw_batches}))
            }).collect::<Vec<_>>()
        });
        self.line(json!({"phase":"frame","sequence":self.frames,"monotonic_ns":self.started.elapsed().as_nanos(),
            "renderer_id":frame.renderer.resource_owner_id(),"extent":frame.extent,
            "submitted_frame":frame.submitted_frame,
            "redraw_scissors":frame.renderer.frame_redraw_scissors().collect::<Vec<_>>(),"render_error":frame.error.map(|e|format!("{e:#}")),
            "text_observation_attempted":frame.text_observation_attempted,"text_admission":text,"text_origins":text_origins,
            "text_admission_failed":frame.renderer.text_admission_observation_failed(),
            "prepared_screen_geometry":screen,"submitted_screen_geometry":submitted_screen,"prepared_surface_grids":grids,
            "prepared_terminal_graphics":frame.prepared.prepared_terminal_graphic_count(),"submitted_terminal_geometry":terminal,
            "glyph_preparation_workspace_overlay":glyphs,"world_panes":panes,"submitted_world_bundles":encoded_world,
            "scope":"render attempt and prepared retained-world/text; not presentation, raster coverage or complete immediate UI geometry"}))
    }
}

#[path = "native_frame_draw_measurements.rs"]
mod draw_measurements;
