//! Footprint-owned graphics acquire their whole parent through the production
//! pointer path: retained hit index -> typed binding -> eligibility -> reducer.
use super::*;
use datum_gui_protocol::selection_scene::SceneSelectionBindings;
use datum_gui_viewport::selection_acquisition::{
    AcquisitionEntry, AcquisitionMode, acquire_authored,
};

#[test]
#[ignore = "requires frozen library-parents native recipe (S5A_NATIVE_PARENT_FIXTURE)"]
fn actual_owned_footprint_graphic_hit_acquires_only_its_parent_footprint() {
    let root = std::path::PathBuf::from(
        std::env::var_os("S5A_NATIVE_PARENT_FIXTURE").expect("explicit frozen recipe"),
    );
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let mut before: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let journal = eda_engine::substrate::transaction_journal_path(&root);
    before.push((journal.clone(), std::fs::read(&journal).unwrap()));
    let state = datum_gui_protocol::load_board_editor_workspace_state(
        &datum_gui_protocol::LiveReviewRequest {
            project_root: root.clone(),
            board_file: None,
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        },
    )
    .unwrap();
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let bindings = SceneSelectionBindings::capture(&state.scene, &authority);
    let mut proven = 0;
    for component in &state.scene.components {
        let graphics: Vec<_> = state
            .scene
            .component_graphics
            .iter()
            .filter(|g| g.component_uuid == component.component_uuid)
            .collect();
        assert!(
            !graphics.is_empty(),
            "actual persisted component_mechanical source reaches the scene"
        );
        let parent = Id {
            class: Class::Footprint,
            id: component.source_object_uuid.parse().unwrap(),
            instance_path: vec![],
        };
        for graphic in graphics {
            // An authored vertex outside every pad isolates the owned graphic hit.
            let point = *graphic
                .path
                .iter()
                .find(|p| {
                    !state.scene.pads.iter().any(|pad| {
                        (pad.bounds.min_x..=pad.bounds.max_x).contains(&p.x)
                            && (pad.bounds.min_y..=pad.bounds.max_y).contains(&p.y)
                    })
                })
                .expect("graphic vertex outside pads");
            let hit = retained.world_hit_index.hit_test(point, |_| true);
            let key = hit.target.expect("owned graphic is hittable");
            assert_eq!(key, &component.object_id, "owned graphic hit collapses");
            let binding = bindings.get(key).expect("typed binding");
            assert_eq!(binding.acquired, parent);
            assert!(binding.eligible(&state.ui.filters));
            let acquired = acquire_authored(
                &SelectionSubject::None,
                [binding.acquired.clone()].into_iter().collect(),
                AcquisitionMode::Replace,
                AcquisitionEntry::Individual,
                &authority,
            )
            .unwrap();
            assert_eq!(
                acquired.members,
                std::collections::BTreeSet::from([parent.clone()])
            );
            let view = build_selection_view(
                &model,
                &authority,
                acquired.subject,
                &state.scene,
                state.schematic_scene.as_ref(),
                |_| true,
            )
            .unwrap();
            assert_eq!(view.members, acquired.members, "no child graphic member");
            assert_eq!(
                view.presentation_owner(
                    &state.scene,
                    datum_gui_protocol::PaneContent::Board,
                    &component.object_id
                ),
                Some(&parent),
                "the whole owned presentation projects as its parent"
            );
            proven += 1;
        }
    }
    assert!(proven >= 2, "both placed parents prove an owned graphic");
    for (path, bytes) in before {
        assert_eq!(
            std::fs::read(&path).unwrap(),
            bytes,
            "read-only acquisition"
        );
    }
}
