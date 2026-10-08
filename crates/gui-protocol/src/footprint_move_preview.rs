//! Complete source-qualified footprint translation preview. This is consumer
//! scene state only; the native Project bytes remain unchanged until commit.
use crate::*;
use eda_engine::board::nominal_geometry::Rational;

pub struct FootprintMovePreview {
    pub base: BoardReviewSceneV1,
    pub component: ComponentBounds,
    pub cursor_anchor: PointNm,
}
impl FootprintMovePreview {
    pub fn begin(scene: &BoardReviewSceneV1, target: &str, cursor: PointNm) -> Result<Self> {
        let component = scene
            .components
            .iter()
            .find(|c| c.object_id == target || c.component_uuid == target)
            .or_else(|| {
                scene
                    .pads
                    .iter()
                    .find(|p| p.object_id == target)
                    .and_then(|p| {
                        scene
                            .components
                            .iter()
                            .find(|c| c.component_uuid == p.component_uuid)
                    })
            })
            .context("Move requires an existing Footprint")?
            .clone();
        if component.locked {
            bail!("Cannot move a locked Footprint");
        }
        Ok(Self {
            base: scene.clone(),
            component,
            cursor_anchor: cursor,
        })
    }
    pub fn destination(&self, cursor: PointNm, snap: AuthoringSnapState) -> Result<PointNm> {
        let x = self
            .component
            .position
            .x
            .checked_add(
                cursor
                    .x
                    .checked_sub(self.cursor_anchor.x)
                    .context("Move exceeds integer coordinates")?,
            )
            .context("Move exceeds integer coordinates")?;
        let y = self
            .component
            .position
            .y
            .checked_add(
                cursor
                    .y
                    .checked_sub(self.cursor_anchor.y)
                    .context("Move exceeds integer coordinates")?,
            )
            .context("Move exceeds integer coordinates")?;
        fn snap_one(v: i64, s: AuthoringSnapState) -> Result<i64> {
            if !s.enabled || s.grid_nm <= 0 {
                return Ok(v);
            }
            let adjusted = if v >= 0 {
                v.checked_add(s.grid_nm / 2)
            } else {
                v.checked_sub(s.grid_nm / 2)
            }
            .context("Move snap exceeds integer coordinates")?;
            (adjusted / s.grid_nm)
                .checked_mul(s.grid_nm)
                .context("Move snap exceeds integer coordinates")
        }
        Ok(PointNm {
            x: snap_one(x, snap)?,
            y: snap_one(y, snap)?,
        })
    }
    pub fn scene_at(&self, destination: PointNm) -> Result<BoardReviewSceneV1> {
        let dx = destination
            .x
            .checked_sub(self.component.position.x)
            .context("Move exceeds integer coordinates")?;
        let dy = destination
            .y
            .checked_sub(self.component.position.y)
            .context("Move exceeds integer coordinates")?;
        let shift = |p: PointNm| -> Result<PointNm> {
            Ok(PointNm {
                x: p.x
                    .checked_add(dx)
                    .context("Move geometry exceeds integer coordinates")?,
                y: p.y
                    .checked_add(dy)
                    .context("Move geometry exceeds integer coordinates")?,
            })
        };
        let rect = |r: RectNm| -> Result<RectNm> {
            let a = shift(PointNm {
                x: r.min_x,
                y: r.min_y,
            })?;
            let b = shift(PointNm {
                x: r.max_x,
                y: r.max_y,
            })?;
            Ok(RectNm {
                min_x: a.x,
                min_y: a.y,
                max_x: b.x,
                max_y: b.y,
            })
        };
        let path = |p: &mut Vec<PointNm>| -> Result<()> {
            for q in p {
                *q = shift(*q)?;
            }
            Ok(())
        };
        let mut scene = self.base.clone();
        for component in scene
            .components
            .iter_mut()
            .filter(|c| c.component_uuid == self.component.component_uuid)
        {
            component.position = shift(component.position)?;
            component.bounds = rect(component.bounds)?;
        }
        for pad in scene
            .pads
            .iter_mut()
            .filter(|p| p.component_uuid == self.component.component_uuid)
        {
            pad.center = match pad.center {
                PadCenter::Integer(point) => PadCenter::Integer(shift(point)?),
                PadCenter::Exact {
                    mut exact,
                    local_width_nm,
                    local_height_nm,
                    back,
                } => {
                    exact[0] = exact[0]
                        .add(Quadratic3::rational(Rational::integer(dx.into())))
                        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
                    exact[1] = exact[1]
                        .add(Quadratic3::rational(Rational::integer(dy.into())))
                        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
                    PadCenter::Exact {
                        exact,
                        local_width_nm,
                        local_height_nm,
                        back,
                    }
                }
            };
            pad.bounds = rect(pad.bounds)?;
        }
        for graphic in scene
            .component_graphics
            .iter_mut()
            .filter(|g| g.component_uuid == self.component.component_uuid)
        {
            path(&mut graphic.path)?;
            for hole in &mut graphic.holes {
                path(hole)?;
            }
        }
        for text in scene
            .component_texts
            .iter_mut()
            .filter(|t| t.component_uuid == self.component.component_uuid)
        {
            text.position = shift(text.position)?;
            for polygon in &mut text.cached_polygons {
                path(polygon)?;
            }
        }
        // Existing copper is the unchanged source preimage during preview.
        // Tracks and vias remain fixed; commit refresh recomputes derived state.
        Ok(scene)
    }
}
