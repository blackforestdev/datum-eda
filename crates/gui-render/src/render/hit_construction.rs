//! Shared preflight and fallible ownership for borrowed retained hit geometry.
use super::*;
use crate::cpu_alloc::heap::allocation_bytes;
use std::alloc::Layout;

pub(super) enum Shape<'a> {
    Rect(datum_gui_protocol::RectNm),
    Polyline {
        path: &'a [PointNm],
        half_width_nm: f32,
    },
    Polygon(&'a [PointNm]),
    Circle {
        center: PointNm,
        radius_nm: f32,
    },
}

pub(super) struct Region<'a> {
    pub target: &'a str,
    pub layer_id: Option<&'a str>,
    pub shape: Shape<'a>,
}

fn bytes<T>(len: usize) -> anyhow::Result<usize> {
    Ok(allocation_bytes(Layout::array::<T>(len)?))
}

impl Region<'_> {
    fn payload_bytes(&self) -> anyhow::Result<usize> {
        let path = match self.shape {
            Shape::Polyline { path, .. } | Shape::Polygon(path) => bytes::<PointNm>(path.len())?,
            _ => 0,
        };
        bytes::<u8>(self.target.len())?
            .checked_add(bytes::<u8>(self.layer_id.map_or(0, str::len))?)
            .and_then(|value| value.checked_add(path))
            .ok_or_else(|| anyhow::anyhow!("retained hit payload overflow"))
    }

    fn into_owned(self) -> anyhow::Result<WorldHitRegion> {
        Ok(WorldHitRegion {
            target: HitTarget::AuthoredObject(copy_string(self.target)?),
            layer_id: self.layer_id.map(copy_string).transpose()?,
            shape: match self.shape {
                Shape::Rect(rect) => WorldHitShape::Rect(rect),
                Shape::Polyline {
                    path,
                    half_width_nm,
                } => WorldHitShape::Polyline {
                    path: copy_slice(path)?,
                    half_width_nm,
                },
                Shape::Polygon(path) => WorldHitShape::Polygon(copy_slice(path)?),
                Shape::Circle { center, radius_nm } => WorldHitShape::Circle { center, radius_nm },
            },
        })
    }
}

/// The same borrowed visitor determines exact cost and then constructs output.
/// It must emit deterministically without allocating source-dependent scratch.
pub(super) fn build<'a>(
    mut visit: impl FnMut(&mut dyn FnMut(Region<'a>) -> anyhow::Result<()>) -> anyhow::Result<()>,
    admit: impl FnOnce(usize) -> anyhow::Result<()>,
) -> anyhow::Result<Vec<WorldHitRegion>> {
    let mut count = 0usize;
    let mut payload = 0usize;
    visit(&mut |region| {
        count = count
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("hit count overflow"))?;
        payload = payload
            .checked_add(region.payload_bytes()?)
            .ok_or_else(|| anyhow::anyhow!("hit storage overflow"))?;
        Ok(())
    })?;
    let required = payload
        .checked_add(bytes::<WorldHitRegion>(count)?)
        .ok_or_else(|| anyhow::anyhow!("hit storage overflow"))?;
    admit(required)?;
    let mut out = Vec::new();
    out.try_reserve_exact(count)?;
    visit(&mut |region| {
        anyhow::ensure!(out.len() < count, "hit visitor grew after admission");
        out.push(region.into_owned()?);
        Ok(())
    })?;
    anyhow::ensure!(out.len() == count, "hit visitor changed after admission");
    Ok(out)
}

fn copy_slice<T: Copy>(source: &[T]) -> anyhow::Result<Vec<T>> {
    let mut copy = Vec::new();
    copy.try_reserve_exact(source.len())?;
    copy.extend_from_slice(source);
    Ok(copy)
}

fn copy_string(source: &str) -> anyhow::Result<String> {
    let mut copy = String::new();
    copy.try_reserve_exact(source.len())?;
    copy.push_str(source);
    Ok(copy)
}
