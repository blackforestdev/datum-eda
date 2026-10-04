//! Authored lock presentation is independent of selection, focus and visibility.
//! Only existing Package ownership qualifies a source; drill voids stay intact.
use super::*;
use crate::presentation_ranges::{Role, Span};
fn grey(color: [f32; 3]) -> [f32; 3] {
    let neutral = color[0] * 0.2126 + color[1] * 0.7152 + color[2] * 0.0722;
    mix_color(color, [neutral; 3], 0.25)
}
pub(crate) fn apply(
    scene: &BoardReviewSceneV1,
    quads: &mut [Quad],
    quad_sources: &[Span],
    strokes: &mut [WorldStrokeInstance],
    stroke_sources: &[Span],
    mut admit: impl FnMut(usize) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let count = scene.components.iter().filter(|c| c.locked).count();
    if count == 0 {
        return Ok(());
    }
    admit(crate::cpu_alloc::heap::capacity_bytes::<(&str, &str)>(
        count,
    ))?;
    let mut parents = Vec::new();
    parents.try_reserve_exact(count)?;
    parents.extend(
        scene
            .components
            .iter()
            .filter(|c| c.locked)
            .map(|c| (c.component_uuid.as_str(), c.object_id.as_str())),
    );
    parents.sort_unstable_by_key(|(uuid, _)| *uuid);
    let owned = |uuid: &str| {
        parents
            .binary_search_by_key(&uuid, |(uuid, _)| *uuid)
            .is_ok()
    };
    let pads = scene
        .pads
        .iter()
        .filter(|p| owned(&p.component_uuid))
        .count();
    let capacity = count
        .checked_add(pads)
        .ok_or_else(|| anyhow::anyhow!("locked presentation index overflow"))?;
    admit(crate::cpu_alloc::heap::capacity_bytes::<&str>(capacity))?;
    let mut keys = Vec::new();
    keys.try_reserve_exact(capacity)?;
    keys.extend(parents.iter().map(|(_, key)| *key));
    keys.extend(
        scene
            .pads
            .iter()
            .filter(|p| owned(&p.component_uuid))
            .map(|p| p.object_id.as_str()),
    );
    keys.sort_unstable();
    keys.dedup();
    let locked = |s: &Span| {
        s.role != Role::Void
            && s.key
                .as_deref()
                .is_some_and(|key| keys.binary_search(&key).is_ok())
    };
    for span in quad_sources.iter().filter(|s| locked(s)) {
        let range = quads
            .get_mut(span.range.clone())
            .ok_or_else(|| anyhow::anyhow!("invalid locked quad provenance"))?;
        for q in range {
            q.color = grey(q.color);
        }
    }
    for span in stroke_sources.iter().filter(|s| locked(s)) {
        let range = strokes
            .get_mut(span.range.clone())
            .ok_or_else(|| anyhow::anyhow!("invalid locked stroke provenance"))?;
        for s in range {
            let color = grey([s.color_min_px[0], s.color_min_px[1], s.color_min_px[2]]);
            s.color_min_px[..3].copy_from_slice(&color);
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn locked_owned_material_changes_chroma_only_and_preserves_void_and_unrelated() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let component = state.scene.components.first_mut().unwrap();
        component.locked = true;
        let parent = component.object_id.clone();
        let uuid = component.component_uuid.clone();
        let pad = state
            .scene
            .pads
            .iter()
            .find(|p| p.component_uuid == uuid)
            .unwrap()
            .object_id
            .clone();
        let color = [0.8, 0.4, 0.1];
        let original = Quad::from_rect(
            RectPx {
                x: 1.0,
                y: 2.0,
                width: 3.0,
                height: 4.0,
            },
            color,
        );
        let mut quads = vec![original; 4];
        let spans = vec![
            Span {
                key: Some(parent.clone()),
                role: Role::Material,
                range: 0..1,
            },
            Span {
                key: Some(pad.clone()),
                role: Role::Material,
                range: 1..2,
            },
            Span {
                key: Some(pad),
                role: Role::Void,
                range: 2..3,
            },
            Span {
                key: None,
                role: Role::Material,
                range: 3..4,
            },
        ];
        let original_stroke = WorldStrokeInstance::segment(
            PointNm { x: 0, y: 0 },
            PointNm { x: 10, y: 20 },
            color,
            100,
            1.0,
        );
        let mut strokes = vec![original_stroke];
        apply(
            &state.scene,
            &mut quads,
            &spans,
            &mut strokes,
            &[Span {
                key: Some(parent),
                role: Role::Material,
                range: 0..1,
            }],
            |_| Ok(()),
        )
        .unwrap();
        for q in &quads[..2] {
            assert_eq!(q.color, grey(color));
            assert_eq!(q.points, original.points);
        }
        assert_eq!(&quads[2..], &[original, original]);
        assert_eq!(strokes[0].from, original_stroke.from);
        assert_eq!(strokes[0].to, original_stroke.to);
        assert_eq!(strokes[0].nominal_nm, original_stroke.nominal_nm);
        assert_eq!(strokes[0].color_min_px[3], original_stroke.color_min_px[3]);
        assert_eq!(&strokes[0].color_min_px[..3], &grey(color));
        assert!(grey(color)[0] - grey(color)[2] < color[0] - color[2]);
    }
    #[test]
    fn unlocked_scene_is_exact_and_index_admission_refuses_before_material_changes() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        for c in &mut state.scene.components {
            c.locked = false;
        }
        let original = Quad::from_rect(
            RectPx {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            [0.8, 0.2, 0.1],
        );
        let mut quads = vec![original];
        apply(&state.scene, &mut quads, &[], &mut [], &[], |_| {
            panic!("no unlocked allocation")
        })
        .unwrap();
        state.scene.components[0].locked = true;
        assert!(
            apply(&state.scene, &mut quads, &[], &mut [], &[], |_| Err(
                anyhow::anyhow!("admission")
            ))
            .is_err()
        );
        assert_eq!(quads, vec![original]);
    }
}
