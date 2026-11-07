//! Transient Wire/Bus vertices share the engine's checked path geometry.
use super::*;
use eda_engine::{api::native_write::schematic_wire_edit::routed_wire_points, ir::geometry::Point};

impl SchematicAuthoring {
    pub fn preview_wire_points(&self) -> Result<Vec<PointNm>, String> {
        if !self.wire {
            return Err("No Wire or Bus draft is active".into());
        }
        let mut points = self.vertices.clone();
        if let Some(at) = self.position {
            if let Some(&from) = points.last() {
                points.extend(self.routed_draft_span(from, at)?.into_iter().skip(1));
            } else {
                points.push(at);
            }
        }
        Ok(points)
    }

    /// Anchor exactly the same path shown by preview. Electrical snapping is
    /// owned by the caller, so the final contact is never quantized here.
    pub fn append_wire_vertex(&mut self, at: PointNm) -> Result<(), String> {
        if !self.wire {
            return Err("No Wire or Bus draft is active".into());
        }
        if self.pending_request.is_some() {
            return Err("Wait for the pending Wire or Bus edit".into());
        }
        if self.vertices.last() == Some(&at) {
            return Ok(());
        }
        let points = match self.vertices.last().copied() {
            Some(from) => self
                .routed_draft_span(from, at)?
                .into_iter()
                .skip(1)
                .collect(),
            None => vec![at],
        };
        self.vertices.extend(points);
        self.position = Some(at);
        self.preview_dirty = true;
        self.refusal = None;
        Ok(())
    }

    pub fn remove_last_wire_vertex(&mut self) -> bool {
        if !self.wire || self.pending_request.is_some() || self.vertices.pop().is_none() {
            return false;
        }
        self.position = self.vertices.last().copied();
        self.last_wire_click = None;
        self.preview_dirty = true;
        self.refusal = None;
        true
    }

    fn routed_draft_span(&self, from: PointNm, to: PointNm) -> Result<Vec<PointNm>, String> {
        routed_wire_points(
            Point::new(from.x, from.y),
            Point::new(to.x, to.y),
            self.wire_route,
        )
        .map(|points| {
            points
                .into_iter()
                .map(|p| PointNm { x: p.x, y: p.y })
                .collect()
        })
        .map_err(|error| error.to_string())
    }
}

#[cfg(test)]
#[path = "schematic_wire_draft_tests.rs"]
mod tests;
