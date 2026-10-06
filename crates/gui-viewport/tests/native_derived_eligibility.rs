//! Pre-acquisition origin gating over the persisted semantic recipe. These are
//! production-adapter/reducer oracles, not native pointer or accessibility proof.
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, DatumSelectionContext,
    LiveReviewRequest, SelectionSubject, SelectionTarget,
    selection_resolution::{NativeSelectionResolution, SelectionAuthority},
    selection_scene::SceneSelectionBindings,
    selection_view::build_selection_view,
};
use datum_gui_viewport::selection_acquisition::{
    AcquisitionEntry as Entry, AcquisitionMode as Mode, acquire_authored,
};
use eda_engine::substrate::{ProjectResolver, transaction_journal_path};
use std::{collections::BTreeSet, path::PathBuf};

fn replay(bus: bool) {
    let fixture = PathBuf::from(
        std::env::var_os("S5A_NATIVE_SEMANTIC_FIXTURE").expect("explicit frozen semantic recipe"),
    );
    assert!(std::env::var_os("EDA_CLI_BIN").is_some(), "pin the CLI");
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(fixture.join("s5a-fixture.json")).unwrap()).unwrap();
    let report = &report["semantic_variant"];
    let root = fixture.join("variants/semantic-scope");
    let model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(model.model_revision.0, report["model_revision"]);
    let mut before: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let journal = transaction_journal_path(&root);
    before.push((journal.clone(), std::fs::read(journal).unwrap()));
    let state = datum_gui_protocol::load_board_editor_workspace_state(&LiveReviewRequest {
        project_root: root.clone(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    })
    .unwrap();
    let scene = state.schematic_scene.as_ref().unwrap();
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let bindings = SceneSelectionBindings::capture(scene, &authority);
    let origin_class = if bus { Class::BusSection } else { Class::Wire };
    let origin = bindings
        .iter()
        .filter(|(_, binding)| binding.acquired.class == origin_class)
        .min_by_key(|(_, binding)| binding.acquired.id)
        .unwrap()
        .1;
    let run = authority
        .acquire_run(origin.acquired.clone(), None)
        .unwrap();
    let SelectionSubject::Run(run_id) = &run else {
        unreachable!()
    };
    let semantic = authority.widen_run(run_id).unwrap();
    assert_eq!(
        semantic,
        if bus {
            SelectionSubject::Bus(report["primary_bus"].as_str().unwrap().parse().unwrap())
        } else {
            SelectionSubject::GlobalNet(report["logical_net"].as_str().unwrap().parse().unwrap())
        }
    );
    for subject in [run, semantic] {
        let complete = authority.members(&subject).unwrap();
        assert!(complete.contains(&origin.acquired));
        let baseline = build_selection_view(
            &model,
            &authority,
            subject.clone(),
            &state.scene,
            Some(scene),
            |_| true,
        )
        .unwrap();
        assert_eq!(baseline.members, complete);
        // Local overlap Select All is an ordinary replacement, not Ctrl+A.
        // Duplicate projections collapse by authoritative acquired identity.
        let mut menu = datum_gui_protocol::selection_menu::SelectionMenu::capture(
            &authority,
            vec![(origin.acquired.clone(), origin.projected.clone()); 2],
            datum_gui_protocol::PointNm { x: 0, y: 0 },
            state.ui.filters.clone(),
        );
        menu.append_inspection(&baseline);
        let local = acquire_authored(
            &subject,
            menu.overlap_candidates().cloned().collect(),
            Mode::Replace,
            Entry::Region,
            &authority,
        )
        .unwrap();
        assert_eq!(local.members, BTreeSet::from([origin.acquired.clone()]));
        assert_eq!(
            local.subject,
            SelectionSubject::Compound(
                datum_gui_protocol::CompoundSelection::new(local.members.clone(), None).unwrap()
            )
        );
        let mut empty = datum_gui_protocol::selection_menu::SelectionMenu::capture(
            &authority,
            vec![],
            datum_gui_protocol::PointNm { x: 0, y: 0 },
            state.ui.filters.clone(),
        );
        empty.append_inspection(&baseline);
        let unchanged = acquire_authored(
            &subject,
            empty.overlap_candidates().cloned().collect(),
            Mode::Replace,
            Entry::Region,
            &authority,
        )
        .unwrap();
        assert_eq!(unchanged.subject, subject);
        assert_eq!(unchanged.members, complete);
        assert!(!unchanged.changed);
        let member = if bus {
            origin
        } else {
            bindings
                .iter()
                .map(|(_, binding)| binding)
                .find(|binding| {
                    binding.acquired.class == Class::Junction
                        && complete.contains(&binding.acquired)
                })
                .expect("actual connected Junction, separate visible layer")
        };
        let mut filters = state.ui.filters.clone();
        // Set eligibility before resolving the derived selection. Hidden-origin
        // Bus is a negative case; no differential Bus layer is fabricated.
        for layer in &member.layers {
            filters.layer_visibility.insert(layer.clone(), false);
        }
        assert!(!member.eligible(&filters));
        assert_eq!(origin.eligible(&filters), !bus);
        if origin.eligible(&filters) {
            let acquired = authority
                .acquire_run(origin.acquired.clone(), None)
                .unwrap();
            let acquired = if matches!(subject, SelectionSubject::GlobalNet(_)) {
                let SelectionSubject::Run(id) = acquired else {
                    unreachable!()
                };
                authority.widen_run(&id).unwrap()
            } else {
                acquired
            };
            assert_eq!(
                acquired, subject,
                "pre-acquisition eligibility does not prune membership"
            );
            assert_eq!(authority.members(&acquired).unwrap(), complete);
        }

        let hidden = build_selection_view(
            &model,
            &authority,
            subject.clone(),
            &state.scene,
            Some(scene),
            |layer| filters.layer_visibility.get(layer).copied().unwrap_or(true),
        )
        .unwrap();
        assert_eq!(hidden.members, complete);
        assert!(hidden.hidden.contains(&member.projected));
        assert_eq!(hidden.unprojected, baseline.unprojected);
        let context =
            DatumSelectionContext::from_selection(&SelectionTarget::Native(Box::new(hidden)));
        let details = context.details.unwrap();
        assert_eq!(details.total, complete.len());
        assert!(details.hidden > 0);

        filters.layer_visibility.clear();
        filters
            .excluded_selection_classes
            .insert(member.acquired.class);
        assert!(!member.eligible(&filters));
        assert_eq!(origin.eligible(&filters), !bus);
        if origin.eligible(&filters) {
            let acquired = authority
                .acquire_run(origin.acquired.clone(), None)
                .unwrap();
            let acquired = if matches!(subject, SelectionSubject::GlobalNet(_)) {
                let SelectionSubject::Run(id) = acquired else {
                    unreachable!()
                };
                authority.widen_run(&id).unwrap()
            } else {
                acquired
            };
            assert_eq!(
                acquired, subject,
                "pre-acquisition eligibility does not prune membership"
            );
            assert_eq!(authority.members(&acquired).unwrap(), complete);
        }

        let filtered = build_selection_view(
            &model,
            &authority,
            subject.clone(),
            &state.scene,
            Some(scene),
            |_| true,
        )
        .unwrap();
        assert_eq!(filtered.members, complete);
        assert_eq!(filtered.schematic_keys, baseline.schematic_keys);
        assert_eq!(filtered.hidden, baseline.hidden);
        // Class-filtered B remains in the selected source-owned projection.
        // This asserts projection incidence, not pixels or native input acceptance.
        let member_keys: BTreeSet<_> = bindings
            .iter()
            .filter(|(_, binding)| binding.projected == member.projected)
            .map(|(key, _)| key.to_owned())
            .collect();
        assert!(!member_keys.is_empty());
        assert!(member_keys.is_subset(&filtered.schematic_keys));
        filters.show_authored = false;
        assert!(!origin.eligible(&filters));

        let foreign: Id = authority
            .authored_identities()
            .iter()
            .find(|id| !matches!(id.class, Class::Pad | Class::Pin) && !complete.contains(*id))
            .expect("fixture has a foreign subject")
            .clone();
        for entry in [Entry::Individual, Entry::Region] {
            for (mode, candidates, refusal) in [
                (Mode::Add, BTreeSet::from([origin.acquired.clone()]), false),
                (Mode::Remove, BTreeSet::from([foreign.clone()]), false),
                (
                    Mode::Remove,
                    BTreeSet::from([member.acquired.clone()]),
                    true,
                ),
                (Mode::Add, BTreeSet::from([foreign.clone()]), true),
            ] {
                let result =
                    acquire_authored(&subject, candidates, mode, entry, &authority).unwrap();
                assert_eq!(result.subject, subject);
                assert_eq!(result.members, complete);
                assert!(!result.changed);
                assert_eq!(result.refusal.is_some(), refusal);
                if let Some(reason) = result.refusal {
                    assert!(
                        reason
                            .explanation()
                            .contains("Membership follows connectivity")
                    );
                }
            }
        }
    }
    for (path, bytes) in before {
        assert_eq!(std::fs::read(path).unwrap(), bytes, "no authored mutation");
    }
    assert_eq!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .model_revision,
        model.model_revision
    );
}

#[test]
#[ignore = "requires frozen semantic-scope recipe and pinned CLI; no absent-fixture pass"]
fn visible_wire_acquires_hidden_and_filtered_complete_run_and_global_net_members() {
    replay(false);
}

#[test]
#[ignore = "requires frozen semantic-scope recipe and pinned CLI; no absent-fixture pass"]
fn declared_bus_origin_gate_preserves_complete_subject_and_refusal_noop_law() {
    replay(true);
}
