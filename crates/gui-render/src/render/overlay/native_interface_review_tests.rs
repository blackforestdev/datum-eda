//! Complete interface identity review and constrained viewport refusal.
use super::tests::review_fixture;
use super::*;

#[test]
fn interface_review_keeps_identities_and_refusal_visible_and_refuses_clipped_layout() {
    use eda_engine::api::native_write::schematic_interface_binding::InterfaceBindingSummary;
    let (mut state, scene, _, _) = review_fixture();
    let id = uuid::Uuid::new_v4();
    let binding = InterfaceBindingSummary {
        instance: id,
        definition: uuid::Uuid::new_v4(),
        parent: uuid::Uuid::new_v4(),
        parent_path: vec![],
        child: uuid::Uuid::new_v4(),
        instance_path: vec![id],
        parent_port: uuid::Uuid::new_v4(),
        child_label: uuid::Uuid::new_v4(),
        name: "SIGNAL".into(),
        bind: true,
    };
    let review = state.schematic_authoring.delete_review.as_mut().unwrap();
    review.interface_binding = Some(binding.clone());
    review.targets.clear();
    review.response.as_mut().unwrap().deleted.clear();
    review.refusal = Some("Child endpoint changed; select again".into());
    for (width, height, fits) in [
        (800., 600., true),
        (200., 200., false),
        (200., 2000., false),
    ] {
        let viewport = RectPx {
            x: 0.,
            y: 0.,
            width,
            height,
        };
        let measured = datum_gui_protocol::schematic_authoring::interface_binding::review_layout(
            &binding,
            Some("Child endpoint changed; select again"),
            width,
            height,
        );
        assert_eq!(measured.fits, fits);
        let camera = CameraState::fit_to_bounds(&scene.bounds);
        let projection = Projection::new(viewport, &scene.bounds, camera);
        let mut plan =
            native_selection_overlay::cue_plan::PanePlan::new(&state, viewport, camera).unwrap();
        let (mut quads, mut text) = (vec![], vec![]);
        render(
            &mut quads,
            &mut text,
            &scene,
            viewport,
            &projection,
            &state,
            &mut plan,
        );
        let visible = text
            .iter()
            .map(|run| run.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        if fits {
            for uuid in [
                binding.parent,
                binding.parent_port,
                binding.child,
                binding.child_label,
                binding.instance,
            ] {
                assert!(
                    visible.contains(&uuid.to_string()),
                    "missing identity {uuid}"
                );
            }
            assert!(visible.contains("Child endpoint changed"));
        } else {
            assert!(visible.contains("Enlarge"));
            assert!(!visible.contains("Tab accept"));
        }
    }
}
