//! Disposable painter provenance. A display key/range never establishes an
//! authored identity; consumers must resolve it through the current shared view.
use std::ops::Range;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Role {
    #[default]
    Material,
    Void,
    ZoneBoundary,
    ZoneIsland(usize),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Span {
    pub key: Option<String>,
    pub role: Role,
    pub range: Range<usize>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Presentation {
    pub quads: Vec<Span>,
    pub strokes: Vec<Span>,
}
impl Presentation {
    pub fn bytes(&self) -> usize {
        [&self.quads, &self.strokes]
            .into_iter()
            .fold(0usize, |total, spans| {
                total
                    .saturating_add(crate::cpu_alloc::heap::capacity_bytes::<Span>(
                        spans.capacity(),
                    ))
                    .saturating_add(
                        spans
                            .iter()
                            .map(|span| {
                                crate::cpu_alloc::heap::capacity_bytes::<u8>(
                                    span.key.as_ref().map_or(0, String::capacity),
                                )
                            })
                            .sum::<usize>(),
                    )
            })
    }
    pub fn container_bytes() -> usize {
        if !crate::cpu_alloc::installed() {
            return std::mem::size_of::<Self>();
        }
        static BYTES: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
        *BYTES.get_or_init(|| crate::cpu_alloc::heap::arc_bytes(Self::default()))
    }
}
/// Remap source spans through the exact painter permutation and triangle
/// compaction. The output ranges index final vertices, not authored objects.
pub(crate) fn remap(
    quads: &[crate::Quad],
    commands: &[crate::RetainedDrawCommand],
    spans: &[Span],
    mut admit: impl FnMut(usize) -> anyhow::Result<()>,
) -> anyhow::Result<Vec<Span>> {
    let mut output = Vec::new();
    let mut vertex = 0usize;
    for command in commands {
        let crate::RetainedDrawCommand::Quads { range, .. } = command else {
            continue;
        };
        anyhow::ensure!(
            range.start % 6 == 0 && range.end % 6 == 0,
            "presentation quad alignment"
        );
        let start = range.start as usize / 6;
        let end = range.end as usize / 6;
        anyhow::ensure!(
            start <= end && end <= quads.len(),
            "presentation range outside geometry"
        );
        let mut at = start;
        let mut index = spans.partition_point(|span| span.range.end <= at);
        while at < end {
            let span = spans
                .get(index)
                .ok_or_else(|| anyhow::anyhow!("missing presentation source range"))?;
            anyhow::ensure!(
                span.range.start <= at && span.range.end > at,
                "presentation source gap"
            );
            let stop = end.min(span.range.end);
            let count = quads[at..stop]
                .iter()
                .map(|quad| {
                    if quad.points[2] == quad.points[3] {
                        3usize
                    } else {
                        6
                    }
                })
                .sum::<usize>();
            let next = vertex
                .checked_add(count)
                .ok_or_else(|| anyhow::anyhow!("presentation vertex overflow"))?;
            if output.len() == output.capacity() {
                let capacity = output
                    .len()
                    .checked_mul(2)
                    .ok_or_else(|| anyhow::anyhow!("presentation capacity overflow"))?
                    .max(4);
                admit(crate::cpu_alloc::heap::capacity_bytes::<Span>(capacity))?;
                output.try_reserve_exact(capacity - output.len())?;
            }
            let key = if let Some(key) = &span.key {
                admit(crate::cpu_alloc::heap::capacity_bytes::<u8>(key.len()))?;
                let mut copy = String::new();
                copy.try_reserve_exact(key.len())?;
                copy.push_str(key);
                Some(copy)
            } else {
                None
            };
            output.push(Span {
                key,
                role: span.role,
                range: vertex..next,
            });
            vertex = next;
            at = stop;
            index += 1;
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    #[test]
    fn remap_preserves_sorted_painter_ownership_and_compacted_triangle_ranges() {
        let square = Quad {
            points: [(0., 0.), (1., 0.), (1., 1.), (0., 1.)],
            color: [0.1, 0.2, 0.3],
        };
        let triangle = Quad {
            points: [(0., 0.), (1., 0.), (1., 1.), (1., 1.)],
            color: [0.4, 0.5, 0.6],
        };
        let quads = [square, triangle, square];
        let spans = [
            Span {
                key: Some("body".into()),
                role: Role::Material,
                range: 0..1,
            },
            Span {
                key: Some("body".into()),
                role: Role::Void,
                range: 1..2,
            },
            Span {
                key: None,
                role: Role::Material,
                range: 2..3,
            },
        ];
        let mut commands = [
            RetainedDrawCommand::Quads {
                layer_id: None,
                range: 12..18,
            },
            RetainedDrawCommand::Quads {
                layer_id: None,
                range: 0..12,
            },
        ];
        let result = remap(&quads, &commands, &spans, |_| Ok(())).unwrap();
        let vertices = gpu_data::try_retained_vertices(&quads, &mut commands).unwrap();
        assert_eq!(
            result
                .iter()
                .map(|span| span.range.clone())
                .collect::<Vec<_>>(),
            [0..6, 6..12, 12..15]
        );
        assert_eq!(vertices.len(), 15);
        assert_eq!(result[0].key, None);
        assert_eq!(result[2].role, Role::Void);
        assert_eq!(&vertices[12..15], &gpu_data::quad_vertices(triangle)[..3]);
        assert!(
            remap(&quads, &commands, &spans, |_| Ok(())).is_err(),
            "final compacted commands cannot masquerade as construction ranges"
        );
        assert!(
            remap(
                &quads,
                &[RetainedDrawCommand::Quads {
                    layer_id: None,
                    range: 0..6
                }],
                &[],
                |_| Ok(())
            )
            .is_err()
        );
        assert!(
            remap(
                &quads,
                &[RetainedDrawCommand::Quads {
                    layer_id: None,
                    range: 0..6
                }],
                &spans,
                |_| anyhow::bail!("admission refused")
            )
            .is_err()
        );
    }
    #[test]
    fn emitted_retained_sources_cover_actual_geometry_with_typed_voids() {
        let state = crate::gpu_surface_pass::board_fixture_state();
        let scene = RetainedScene::from_workspace(&state, 960, 720);
        let mut end = 0;
        for span in &scene.presentation.quads {
            assert_eq!(span.range.start, end);
            assert!(span.range.end > span.range.start);
            end = span.range.end;
        }
        assert_eq!(end, scene.world_vertices.len());
        let mut end = 0;
        for span in &scene.presentation.strokes {
            assert_eq!(span.range.start, end);
            end = span.range.end;
        }
        assert_eq!(end, scene.world_strokes.len());
        for track in &state.scene.tracks {
            assert!(
                scene
                    .presentation
                    .strokes
                    .iter()
                    .any(|span| span.key.as_deref() == Some(track.object_id.as_str())
                        && span.role == Role::Material)
            );
        }
        for pad in state
            .scene
            .pads
            .iter()
            .filter(|pad| pad.drill_nm.is_some_and(|drill| drill > 0))
        {
            assert!(
                scene
                    .presentation
                    .quads
                    .iter()
                    .any(|span| span.key.as_deref() == Some(pad.object_id.as_str())
                        && span.role == Role::Void)
            );
        }
        assert!(
            scene
                .presentation
                .quads
                .iter()
                .any(|span| span.role == Role::Void)
        );
    }
}
