//! Actual persisted Projects bind units and lifetime to the shared consumers.
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, CompoundSelection,
    LiveReviewRequest, SelectionSubject,
    selection_resolution::{NativeSelectionResolution, SelectionAuthority},
    selection_view::{InspectionValue, QuantityPresentation, build_selection_view},
};
use eda_engine::substrate::{ProjectResolver, transaction_journal_path};
use serde_json::Value;
use std::path::PathBuf;
#[test]
#[ignore = "requires frozen scope-and-reopen recipe and pinned EDA_CLI_BIN; no absent-fixture pass"]
fn native_project_replacement_clears_selection_and_preserves_exact_quantity_scopes() {
    let root = PathBuf::from(
        std::env::var_os("S5A_NATIVE_SCOPE_FIXTURE").expect("explicit frozen recipe"),
    );
    assert!(
        std::env::var_os("EDA_CLI_BIN").is_some(),
        "pinned CLI; no Cargo fallback"
    );
    let recipe: Value =
        serde_json::from_slice(&std::fs::read(root.join("s5a-fixture.json")).unwrap()).unwrap();
    let mut previous = None;
    for (ordinal, path) in [root.clone(), root.join("variants/second-project")]
        .into_iter()
        .enumerate()
    {
        let model = ProjectResolver::new(&path).resolve().unwrap();
        let mut before: Vec<_> = model
            .source_shards
            .iter()
            .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
            .collect();
        let journal = transaction_journal_path(&path);
        before.push((journal.clone(), std::fs::read(journal).unwrap()));
        let state = datum_gui_protocol::load_board_editor_workspace_state(&LiveReviewRequest {
            project_root: path.clone(),
            board_file: None,
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        })
        .unwrap();
        let authority = NativeSelectionResolution::capture(&model).unwrap();
        let ids: Vec<_> = recipe["scope_variant"]["same_authored_track_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| Id {
                class: Class::Track,
                id: v.as_str().unwrap().parse().unwrap(),
                instance_path: vec![],
            })
            .collect();
        let subject = SelectionSubject::Compound(
            CompoundSelection::new(ids.clone(), Some(ids[1].clone())).unwrap(),
        );
        let mut view = build_selection_view(
            &model,
            &authority,
            subject.clone(),
            &state.scene,
            state.schematic_scene.as_ref(),
            |_| true,
        )
        .unwrap();
        view.scope = Some(Class::Track);
        assert_eq!(view.field("copper_width_nm"), InspectionValue::Mixed);
        assert_eq!(view.field_display_preview("copper_width_nm", 100), "Mixed");
        assert!(
            !view
                .inspection
                .iter()
                .find(|s| s.scope == view.scope)
                .unwrap()
                .quantities
                .contains_key("copper_width_nm"),
            "never format Mixed as rounded Common"
        );
        for (index, id) in ids.iter().enumerate() {
            let mut one = build_selection_view(
                &model,
                &authority,
                SelectionSubject::Object(id.clone()),
                &state.scene,
                None,
                |_| true,
            )
            .unwrap();
            one.scope = Some(Class::Track);
            let display = recipe["scope_variant"]["expected_common_width_displays"][ordinal]
                .as_str()
                .unwrap();
            assert_eq!(
                one.field("copper_width_nm"),
                InspectionValue::Common(serde_json::json!(5_080_000 + index))
            );
            assert_eq!(
                one.field_display_preview("copper_width_nm", 100),
                format!("{display}{}", if index == 1 { " (rounded)" } else { "" })
            );
            assert_eq!(
                one.inspection
                    .iter()
                    .find(|s| s.scope == one.scope)
                    .unwrap()
                    .quantities["copper_width_nm"],
                QuantityPresentation::Formatted {
                    rendered: display.into(),
                    rounded: index == 1
                }
            );
        }
        let members = view.members.clone();
        let focus = view.focus().cloned();
        view.scope = None;
        assert_eq!(view.members, members);
        assert_eq!(view.focus(), focus.as_ref());
        let hidden = build_selection_view(
            &model,
            &authority,
            subject.clone(),
            &state.scene,
            None,
            |_| false,
        )
        .unwrap();
        assert_eq!(hidden.members, members);
        assert_eq!(hidden.hidden.len(), 2);
        if let Some((project, subject)) = previous {
            assert_ne!(project, authority.project());
            assert!(
                ids.iter().all(|id| authority.authored_contains(id)),
                "same object UUIDs in other real Project"
            );
            let replaced = datum_gui_viewport::selection_lifetime::reconcile_selection(
                &subject, project, &authority,
            )
            .unwrap();
            assert_eq!(replaced.subject, SelectionSubject::None);
            assert!(replaced.members.is_empty());
            assert!(replaced.dissolved);
        }
        previous = Some((authority.project(), subject));
        for (path, bytes) in before {
            assert_eq!(
                std::fs::read(path).unwrap(),
                bytes,
                "source/journal unchanged"
            );
        }
        assert_eq!(
            ProjectResolver::new(&path)
                .resolve()
                .unwrap()
                .model_revision,
            model.model_revision
        );
    }
}
