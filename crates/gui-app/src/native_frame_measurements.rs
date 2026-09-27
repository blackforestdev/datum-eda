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
        // Only the composed path executes cached bundles. Upload-only, error
        // and legacy fallback attempts must not publish stale cached draws.
        let encoded_world = (frame.submitted_frame == Some(true)
            && !frame.prepared.surface_passes().is_empty()).then(|| {
            frame.renderer.encoded_world_admission().map(|pane| {
                let mut vertices = 0_u64;
                let mut strokes = 0_u64;
                let mut triangles = 0_u64;
                let ranges: Vec<_> = pane.ranges().map(|range| match range {
                    GeometryAdmissionRange::Vertices(r) => {
                        let n = u64::from(r.end - r.start);
                        vertices += n;
                        triangles += n / 3;
                        json!({"kind":"vertices","start":r.start,"end":r.end})
                    }
                    GeometryAdmissionRange::StrokeInstances(r) => {
                        let n = u64::from(r.end - r.start);
                        strokes += n;
                        triangles += n * 2;
                        json!({"kind":"stroke_instances","start":r.start,"end":r.end})
                    }
                }).collect();
                json!({"pane_id":pane.pane_id.0,"surface":format!("{:?}",pane.surface),
                    "submitted_commands":ranges.len(),"submitted_vertices":vertices,
                    "submitted_stroke_instances":strokes,"submitted_triangles":triangles,"ranges":ranges})
            }).collect::<Vec<_>>()
        });
        let screen: Vec<_> = frame.prepared.screen_geometry_admission().into_iter().map(|g| {
            json!({"group":g.group,"prepared_vertices":g.vertices,"payload_bytes":g.payload_bytes})
        }).collect();
        let grids = frame.renderer.grid_geometry_admission().map(|grids| grids.iter().map(|g| {
            json!({"pane_id":g.pane_id.0,"viewport":[g.viewport.x,g.viewport.y,g.viewport.width,g.viewport.height],
                "range":[g.vertices.start,g.vertices.end],"generated_vertices":g.generated_vertices,
                "prepared_vertices":g.vertices.end-g.vertices.start})
        }).collect::<Vec<_>>());
        let terminal = (frame.submitted_frame == Some(true)).then(|| {
            if frame.prepared.prepared_terminal_graphic_count() == 0 { return Vec::new(); }
            frame.renderer.terminal_geometry_admission().map(|g| {
                json!({"graphic_id":g.graphic_id,"foreground":g.foreground,"scissor":g.scissor,
                    "submitted_vertices":g.vertices,"submitted_triangles":g.vertices/3,"payload_bytes":g.payload_bytes})
            }).collect::<Vec<_>>()
        });
        let admission = frame
            .text_observation_attempted
            .then(|| frame.renderer.text_admission_observation())
            .flatten();
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
            "submitted_frame":frame.submitted_frame,"render_error":frame.error.map(|e|format!("{e:#}")),
            "text_observation_attempted":frame.text_observation_attempted,"text_admission":text,
            "text_admission_failed":frame.renderer.text_admission_observation_failed(),
            "prepared_screen_geometry":screen,"prepared_surface_grids":grids,
            "prepared_terminal_graphics":frame.prepared.prepared_terminal_graphic_count(),"submitted_terminal_geometry":terminal,
            "glyph_preparation_workspace_overlay":glyphs,"world_panes":panes,"submitted_world_bundles":encoded_world,
            "scope":"render attempt and prepared retained-world/text; not presentation, raster coverage or complete immediate UI geometry"}))
    }
}
