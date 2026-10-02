//! Display-only flattening retains one authored Track identity. It is never a
//! source of membership, contact, clearance or manufacturing authority.
use super::*;
use eda_engine::{board::nominal_geometry::CertifiedArc, ir::geometry::Point};
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(super) struct BoardTrackPayload {
    pub(super) uuid: String,
    pub(super) net: String,
    pub(super) from: PointNm,
    pub(super) to: PointNm,
    pub(super) width: i64,
    pub(super) layer: i32,
    #[serde(default)]
    pub(super) display_path: Option<Vec<PointNm>>,
}

pub(super) fn extract_tracks(board: &Value) -> Result<Vec<BoardTrackPayload>> {
    let tracks_map = board
        .get("tracks")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let mut tracks = Vec::with_capacity(tracks_map.len());
    for (_key, value) in tracks_map {
        let track: EngineTrackPayload =
            serde_json::from_value(value).context("failed to parse board track")?;
        tracks.push(BoardTrackPayload {
            display_path: project_track(
                track.from.x,
                track.from.y,
                track.to.x,
                track.to.y,
                track.midpoint.map(|p| (p.x, p.y)),
            )
            .with_context(|| format!("Track {} display projection unavailable", track.uuid))?,
            uuid: track.uuid.to_string(),
            net: track.net.to_string(),
            from: PointNm {
                x: track.from.x,
                y: track.from.y,
            },
            to: PointNm {
                x: track.to.x,
                y: track.to.y,
            },
            width: track.width,
            layer: track.layer,
        });
    }
    tracks.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    Ok(tracks)
}

#[derive(Debug, Clone, Deserialize)]
struct EngineTrackPayload {
    uuid: uuid::Uuid,
    net: uuid::Uuid,
    from: EnginePointPayload,
    to: EnginePointPayload,
    width: i64,
    layer: i32,
    #[serde(default)]
    midpoint: Option<EnginePointPayload>,
}

pub(super) fn track_payload(track: &eda_engine::board::Track) -> Result<BoardTrackPayload> {
    Ok(BoardTrackPayload {
        uuid: track.uuid.to_string(),
        net: track.net.to_string(),
        from: PointNm {
            x: track.from.x,
            y: track.from.y,
        },
        to: PointNm {
            x: track.to.x,
            y: track.to.y,
        },
        width: track.width,
        layer: track.layer,
        display_path: project_track(
            track.from.x,
            track.from.y,
            track.to.x,
            track.to.y,
            track.midpoint.map(|p| (p.x, p.y)),
        )
        .with_context(|| format!("Track {} display projection unavailable", track.uuid))?,
    })
}

pub(super) fn track_primitive(track: BoardTrackPayload) -> TrackPrimitive {
    TrackPrimitive {
        object_id: format!("track:{}", track.uuid),
        object_kind: "track".into(),
        source_object_uuid: track.uuid.clone(),
        track_uuid: track.uuid,
        net_uuid: Some(track.net),
        layer_id: layer_id(track.layer),
        width_nm: track.width,
        path: track
            .display_path
            .unwrap_or_else(|| vec![track.from, track.to]),
    }
}
/// Match the existing 15-degree display tessellation, with exact authored
/// anchors retained. Floating point here has no electrical authority.
pub(super) fn project_track(
    fx: i64,
    fy: i64,
    tx: i64,
    ty: i64,
    midpoint: Option<(i64, i64)>,
) -> Result<Option<Vec<PointNm>>> {
    let Some((mx, my)) = midpoint else {
        return Ok(None);
    };
    let arc = CertifiedArc::new(Point::new(fx, fy), Point::new(mx, my), Point::new(tx, ty))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let cx = arc.center[0].numerator as f64 / arc.center[0].denominator as f64;
    let cy = arc.center[1].numerator as f64 / arc.center[1].denominator as f64;
    let radius =
        (arc.radius_squared.numerator as f64 / arc.radius_squared.denominator as f64).sqrt();
    let angle = |x: i64, y: i64| {
        ((i128::from(y) - i128::from(fy)) as f64 - cy)
            .atan2((i128::from(x) - i128::from(fx)) as f64 - cx)
    };
    let direction = if arc.counterclockwise { 1.0 } else { -1.0 };
    let mut path = vec![PointNm { x: fx, y: fy }];
    for ((sx, sy), (ex, ey)) in [((fx, fy), (mx, my)), ((mx, my), (tx, ty))] {
        let start = angle(sx, sy);
        let sweep = ((angle(ex, ey) - start) * direction).rem_euclid(std::f64::consts::TAU);
        let count = (sweep / (std::f64::consts::PI / 12.0)).ceil().max(1.0) as usize;
        for i in 1..count {
            let theta = start + direction * sweep * i as f64 / count as f64;
            let x = fx as f64 + cx + radius * theta.cos();
            let y = fy as f64 + cy + radius * theta.sin();
            if !x.is_finite()
                || !y.is_finite()
                || x < i64::MIN as f64
                || x >= i64::MAX as f64
                || y < i64::MIN as f64
                || y >= i64::MAX as f64
            {
                bail!("arc display coordinate unavailable");
            }
            path.push(PointNm {
                x: x.round() as i64,
                y: y.round() as i64,
            });
        }
        path.push(PointNm { x: ex, y: ey });
    }
    Ok(Some(path))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arc_display_retains_exact_anchors_and_single_authored_source() {
        let path = project_track(0, 0, 3, 0, Some((1, 3))).unwrap().unwrap();
        assert_eq!(path.first(), Some(&PointNm { x: 0, y: 0 }));
        assert_eq!(path.last(), Some(&PointNm { x: 3, y: 0 }));
        assert!(path.contains(&PointNm { x: 1, y: 3 }));
        assert!(path.len() > 3);
        let primitive = track_primitive(BoardTrackPayload {
            uuid: "authored".into(),
            net: "net".into(),
            from: PointNm { x: 0, y: 0 },
            to: PointNm { x: 3, y: 0 },
            width: 1,
            layer: 1,
            display_path: Some(path.clone()),
        });
        assert_eq!(primitive.source_object_uuid, "authored");
        assert_eq!(primitive.track_uuid, "authored");
        assert_eq!(primitive.path, path);
        assert_eq!(project_track(0, 0, 10, 0, None).unwrap(), None);
        assert!(project_track(0, 0, 10, 0, Some((5, 0))).is_err());
    }
}
