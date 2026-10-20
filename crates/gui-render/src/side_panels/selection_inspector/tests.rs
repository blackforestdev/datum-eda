use super::*;
use crate::native_selection_overlay::tests::selected_state;
#[path = "tests/notice.rs"]
mod notice;
#[path = "tests/singleton_inspector.rs"]
mod singleton_inspector;
#[test]
fn inventory_identity_and_human_reason_survive_duplicate_long_names() {
    let (fixture, state) = selected_state();
    let source = fixture.source_bytes();
    let SelectionTarget::Native(view) = &state.selection else {
        panic!("native fixture")
    };
    let mut preview = view.as_ref().clone();
    let mut first = preview.members.first().unwrap().clone();
    first.class = Class::SchematicText;
    first.id = "50000000-0000-4000-8000-0000000010ff".parse().unwrap();
    let mut last = first.clone();
    last.id = "50000000-0000-4000-8000-000000001100".parse().unwrap();
    for id in [&first, &last] {
        preview.values.insert(
            id.clone(),
            std::collections::BTreeMap::from([
                (
                    "text".into(),
                    datum_gui_protocol::LibraryJsonValue::from(
                        "Ω repeated very long identical member name",
                    ),
                ),
                (
                    "position_nm".into(),
                    datum_gui_protocol::LibraryJsonValue::Object(
                        [("x".into(), 0.into()), ("y".into(), 0.into())]
                            .into_iter()
                            .collect(),
                    ),
                ),
                ("text_rotation".into(), 0.into()),
            ]),
        );
    }
    let before = preview.clone();
    let a = inventory_label::rows(&preview, &first, 256, 43).join(" | ");
    let b = inventory_label::rows(&preview, &last, 257, 43).join(" | ");
    assert_ne!(a, b);
    assert!(a.contains("10ff") && b.contains("1100"));
    assert!(a.contains("Unavailable") && a.contains("height"), "{a}");
    assert!(!a.contains("_nm") && !a.contains("_ppm"));
    for width in [0, 1, 12, 43, 80] {
        assert!(
            inventory_label::rows(&preview, &last, 257, width)
                .iter()
                .all(|row| row.chars().count() <= width)
        );
    }
    let mut occurrence = first.clone();
    occurrence
        .instance_path
        .push("90000000-0000-4000-8000-000000000002".parse().unwrap());
    let mut qualified = preview.clone();
    qualified
        .values
        .insert(occurrence.clone(), preview.values[&first].clone());
    let ordinary_rows = inventory_label::rows(&preview, &first, 1, 43);
    let qualified_rows = inventory_label::rows(&qualified, &occurrence, 1, 43);
    assert_eq!(
        ordinary_rows[0], qualified_rows[0],
        "occurrence must not displace original UUID clue"
    );
    assert!(
        qualified_rows
            .last()
            .unwrap()
            .starts_with("Sheet path … / ")
    );
    assert!(qualified_rows.last().unwrap().ends_with("000000000002"));
    assert_eq!(qualified_rows.len(), ordinary_rows.len() + 1);
    let mut other_occurrence = occurrence.clone();
    occurrence
        .instance_path
        .insert(0, "90000000-0000-4000-8000-000000000003".parse().unwrap());
    other_occurrence
        .instance_path
        .insert(0, "90000000-0000-4000-8000-000000000004".parse().unwrap());
    qualified
        .values
        .insert(occurrence.clone(), preview.values[&first].clone());
    qualified
        .values
        .insert(other_occurrence.clone(), preview.values[&first].clone());
    let ancestor_rows = inventory_label::rows(&qualified, &occurrence, 1, 160);
    let other_rows = inventory_label::rows(&qualified, &other_occurrence, 1, 160);
    assert_eq!(ancestor_rows[0], other_rows[0]);
    assert_ne!(
        ancestor_rows.last(),
        other_rows.last(),
        "same-tail occurrences retain ancestor evidence at ample width"
    );
    assert!(ancestor_rows.last().unwrap().contains("000000000003"));
    assert!(other_rows.last().unwrap().contains("000000000004"));
    assert_eq!(preview, before);
    preview.hidden.insert(last.clone());
    preview.locked.insert(last.clone());
    preview.unprojected.insert(last.clone());
    let combined = preview.clone();
    for width in [43, 80] {
        let rows = inventory_label::rows(&preview, &last, 257, width);
        assert_eq!(rows.len(), inventory_label::row_count(&preview, &last));
        let text = rows.join(" | ");
        for component in ["hidden", "locked", "unprojected", "Unavailable", "height"] {
            assert!(text.contains(component), "{text}");
        }
        assert!(rows.iter().all(|row| row.chars().count() <= width));
    }
    assert_eq!(preview, combined);
    for (path, bytes) in source {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}
fn frame(state: &ReviewWorkspaceState, rect: RectPx) -> (Vec<Quad>, Vec<TextRun>, Vec<HitRegion>) {
    let mut quads = Vec::new();
    let mut text = Vec::new();
    let mut hits = Vec::new();
    super::super::inspector_dispatch::render_active_inspector(
        state, rect, &mut quads, &mut text, &mut hits,
    )
    .unwrap();
    (quads, text, hits)
}
#[test]
fn inventory_status_subrow_publishes_member_and_invalidates_old_capacity() {
    use datum_gui_protocol::selection_view::inspector_state::InspectorPresentationBasis;
    let (fixture, mut state) = selected_state();
    let source = fixture.source_bytes();
    state.ui.selection_inspector.inventory_expanded = true;
    let before = match &state.selection {
        SelectionTarget::Native(view) => {
            InspectorPresentationBasis::capture(view, &state.ui.selection_inspector)
        }
        _ => panic!("native fixture"),
    };
    let SelectionTarget::Native(view) = &mut state.selection else {
        unreachable!()
    };
    let id = view.scoped_members().next().unwrap().clone();
    view.hidden.insert(id.clone());
    let after = InspectorPresentationBasis::capture(view, &state.ui.selection_inspector);
    assert_ne!(
        before, after,
        "same-revision flag subrow changes presentation basis"
    );
    let mut complete = view.as_ref().clone();
    let fields = complete.unavailable_fields(&id);
    assert!(
        !fields.is_empty(),
        "fixture has unavailable canonical properties"
    );
    for field in &fields {
        complete
            .values
            .entry(id.clone())
            .or_default()
            .insert((*field).into(), 0.into());
    }
    let complete_basis =
        InspectorPresentationBasis::capture(&complete, &state.ui.selection_inspector);
    complete.values.get_mut(&id).unwrap().remove(fields[0]);
    assert_ne!(
        complete_basis,
        InspectorPresentationBasis::capture(&complete, &state.ui.selection_inspector),
        "same-revision unavailable subrow changes presentation basis"
    );
    let mut qualified = complete.clone();
    let mut occurrence = id.clone();
    occurrence
        .instance_path
        .push("90000000-0000-4000-8000-000000000002".parse().unwrap());
    assert!(qualified.members.remove(&id));
    qualified.members.insert(occurrence);
    assert_ne!(
        InspectorPresentationBasis::capture(&complete, &state.ui.selection_inspector)
            .qualified_members,
        InspectorPresentationBasis::capture(&qualified, &state.ui.selection_inspector)
            .qualified_members
    );
    let selection = state.selection.clone();
    let rect = RectPx {
        x: 400.,
        y: 80.,
        width: 300.,
        height: 98.,
    };
    let (_, _, hits) = frame(&state, rect);
    let total = hits
        .iter()
        .find_map(|h| match &h.target {
            HitTarget::SelectionInspectorScroll { total, visible, .. } => {
                assert_eq!(*visible, 1);
                Some(*total)
            }
            _ => None,
        })
        .unwrap();
    let mut status_reached = false;
    for offset in 0..total {
        state.ui.selection_inspector.scroll_offset = offset;
        let (_, text, hits) = frame(&state, rect);
        if text.iter().any(|run| run.text == "hidden") {
            let members = hits
                .iter()
                .find_map(|h| match &h.target {
                    HitTarget::SelectionInspectorScroll {
                        visible_members, ..
                    } => Some(visible_members),
                    _ => None,
                })
                .unwrap();
            assert_eq!(
                members,
                &vec![id.clone()],
                "status-only page must publish its exact member once"
            );
            status_reached = true;
        }
        assert_eq!(state.selection, selection);
    }
    assert!(status_reached);
    for (path, bytes) in source {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}
#[test]
fn inventory_and_filters_are_scrollable_with_complete_identity_and_no_reselection() {
    let (fixture, mut state) = selected_state();
    let bytes = fixture.source_bytes();
    let selection = state.selection.clone();
    let rect = RectPx {
        x: 400.0,
        y: 80.0,
        width: 300.0,
        height: 164.0,
    };
    let (_, _, hits) = frame(&state, rect);
    let (total, visible) = hits
        .iter()
        .find_map(|r| match r.target {
            HitTarget::SelectionInspectorScroll { total, visible, .. } => Some((total, visible)),
            _ => None,
        })
        .unwrap();
    assert_eq!(visible, 4);
    assert!(total > visible);
    state.ui.selection_inspector.inventory_expanded = true;
    state.ui.selection_inspector.filters_expanded = true;
    let (_, _, hits) = frame(&state, rect);
    let total = hits
        .iter()
        .find_map(|r| match r.target {
            HitTarget::SelectionInspectorScroll { total, .. } => Some(total),
            _ => None,
        })
        .unwrap();
    let mut found_members = false;
    let mut classes = BTreeSet::new();
    let SelectionTarget::Native(view) = &state.selection else {
        panic!("native fixture");
    };
    let member = view.scoped_members().next().unwrap().clone();
    for offset in 0..total {
        state.ui.selection_inspector.scroll_offset = offset;
        let (quads, text, hits) = frame(&state, rect);
        found_members |= hits.iter().any(|hit| match &hit.target {
            HitTarget::SelectionInspectorScroll {
                visible_members, ..
            } => visible_members.contains(&member),
            _ => false,
        }) && text
            .iter()
            .any(|run| run.text.contains(&member.id.to_string()[28..]));
        for hit in hits {
            assert_eq!(hit.rect.intersect(rect), Some(hit.rect));
            if let HitTarget::SelectionClassToggle(class) = hit.target {
                classes.insert(class);
            }
        }
        assert!(
            quads
                .iter()
                .all(|q| q.points.iter().all(|&(x, y)| rect.contains(x, y)))
        );
        assert!(text.iter().all(|run| {
            run.clip_bounds
                .is_some_and(|clip| clip.intersect(rect) == Some(clip))
        }));
        assert_eq!(state.selection, selection);
    }
    assert!(
        found_members,
        "offscreen inventory can be reached by scrolling"
    );
    assert_eq!(classes, CLASSES.into_iter().collect());
    for (path, bytes) in bytes {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}

#[test]
fn long_values_and_reasons_are_explicit_previews_clipped_to_one_scroll_row() {
    let (fixture, mut state) = selected_state();
    let source = fixture.source_bytes();
    if let SelectionTarget::Native(view) = &mut state.selection {
        view.inspection
            .iter_mut()
            .find(|s| s.scope == view.scope)
            .unwrap()
            .fields
            .insert(
                "zz_large_value".into(),
                datum_gui_protocol::selection_view::InspectionValue::Common(
                    "long value ".repeat(100_000).into(),
                ),
            );
    }
    let selection = state.selection.clone();
    let rect = RectPx {
        x: 400.0,
        y: 80.0,
        width: 300.0,
        height: 200.0,
    };
    let (_, _, hits) = frame(&state, rect);
    let total = hits
        .iter()
        .find_map(|h| match h.target {
            HitTarget::SelectionInspectorScroll { total, .. } => Some(total),
            _ => None,
        })
        .unwrap();
    let mut saw_preview = false;
    for offset in 0..total {
        state.ui.selection_inspector.scroll_offset = offset;
        let (_, text, _) = frame(&state, rect);
        for run in text
            .iter()
            .filter(|r| r.layout_size.is_some_and(|(_, h)| h == 18.0))
        {
            let clip = run.clip_bounds.expect("row-owned clip");
            assert_eq!(clip.height, 22.0);
            assert!(run.y >= clip.y && run.y < clip.y + clip.height);
            assert!(run.text.chars().count() <= 43);
            assert_eq!(clip.intersect(rect), Some(clip));
            if run.text.starts_with("zz large value:") {
                assert!(run.text.ends_with('…'));
                saw_preview = true;
            }
        }
    }
    assert!(saw_preview);
    assert_eq!(state.selection, selection);
    assert_eq!(fixture.source_bytes(), source);
}

#[test]
fn inspector_quantity_rows_use_project_units_without_changing_exact_values() {
    let (f, mut state) = selected_state();
    use eda_engine::{
        api::native_write,
        ir::units::{PreFeatureProjectUnitsMigration, migrate_pre_feature_project_units},
    };
    let PreFeatureProjectUnitsMigration::Create { profile, receipt } =
        migrate_pre_feature_project_units(None).unwrap()
    else {
        panic!("fresh receipt")
    };
    let mut model = eda_engine::substrate::ProjectResolver::new(f.root())
        .resolve()
        .unwrap();
    let write = native_write::project::build_initialize_project_display_units(
        &model,
        native_write::WriteProvenance::new(
            "selection-render-test",
            eda_engine::substrate::CommitSource::Test,
            "actual Project units",
        ),
        profile,
        &receipt,
    )
    .unwrap();
    native_write::commit_prepared(&mut model, f.root(), write).unwrap();
    let bytes = f.source_bytes();
    let SelectionTarget::Native(previous) = &state.selection else {
        panic!("native selection")
    };
    let subject = previous.subject.clone();
    let authority =
        datum_gui_protocol::selection_resolution::NativeSelectionResolution::capture(&model)
            .unwrap();
    state.scene.source_revision = model.model_revision.0.clone();
    let mut view = datum_gui_protocol::selection_view::build_selection_view(
        &model,
        &authority,
        subject,
        &state.scene,
        None,
        |_| true,
    )
    .unwrap();
    view.scope = Some(datum_gui_protocol::AuthoredSelectionClass::Track);
    let datum_gui_protocol::selection_view::InspectionValue::Common(width) =
        view.field("copper_width_nm")
    else {
        panic!("canonical width")
    };
    assert_eq!(width.as_i64(), Some(200000));
    state.selection = SelectionTarget::Native(Box::new(view));
    let selection = state.selection.clone();
    let (_, text, _) = frame(
        &state,
        RectPx {
            x: 400.0,
            y: 80.0,
            width: 600.0,
            height: 1400.0,
        },
    );
    assert!(
        text.iter().any(|t| t.text.contains("Width: 0.200mm")),
        "native Inspector consumes shared formatted quantity"
    );
    assert_eq!(state.selection, selection);
    assert_eq!(f.source_bytes(), bytes);
}

#[test]
fn presented_inspector_basis_refuses_unpresented_expansion_scope_and_notice() {
    use datum_gui_protocol::selection_view::inspector_state::InspectorPresentationBasis;
    let (_fixture, mut state) = selected_state();
    let rect = RectPx {
        x: 400.,
        y: 80.,
        width: 300.,
        height: 300.,
    };
    let (_, _, hits) = frame(&state, rect);
    let before = hits
        .iter()
        .find_map(|hit| match &hit.target {
            HitTarget::SelectionInspectorScroll { basis, .. } => Some(basis.clone()),
            _ => None,
        })
        .unwrap();
    let SelectionTarget::Native(view) = &state.selection else {
        panic!("native selection")
    };
    assert_eq!(
        before,
        InspectorPresentationBasis::capture(view, &state.ui.selection_inspector)
    );
    // Menu-only composition is deliberately absent from this Inspector basis.
    state.ui.active_menu = Some("View".into());
    assert_eq!(
        before,
        InspectorPresentationBasis::capture(view, &state.ui.selection_inspector)
    );
    state.ui.selection_inspector.toggle_inventory();
    assert_ne!(
        before,
        InspectorPresentationBasis::capture(view, &state.ui.selection_inspector)
    );
    let (_, _, hits) = frame(&state, rect);
    let expanded = hits
        .iter()
        .find_map(|hit| match &hit.target {
            HitTarget::SelectionInspectorScroll { basis, .. } => Some(basis.clone()),
            _ => None,
        })
        .unwrap();
    let SelectionTarget::Native(view) = &mut state.selection else {
        unreachable!()
    };
    view.notice = Some("basis changed".into());
    assert_ne!(
        expanded,
        InspectorPresentationBasis::capture(view, &state.ui.selection_inspector)
    );
    view.notice = None;
    view.scope = None;
    assert_ne!(
        expanded,
        InspectorPresentationBasis::capture(view, &state.ui.selection_inspector)
    );
}

#[test]
fn focused_duplicate_names_keep_authored_identity_and_qualified_path() {
    use datum_gui_protocol::{AuthoredSelectionIdentity, CompoundSelection, SelectionSubject};
    let (fixture, mut state) = selected_state();
    let source = fixture.source_bytes();
    let SelectionTarget::Native(original) = &state.selection else {
        panic!("native fixture")
    };
    let mut view = original.as_ref().clone();
    let a = AuthoredSelectionIdentity {
        class: Class::SchematicText,
        id: "50000000-0000-4000-8000-0000000010fe".parse().unwrap(),
        instance_path: vec![],
    };
    let mut b = a.clone();
    b.id = "50000000-0000-4000-8000-0000000010ff".parse().unwrap();
    view.members = [a.clone(), b.clone()].into_iter().collect();
    for id in [&a, &b] {
        view.values.insert(
            id.clone(),
            [(
                "text".into(),
                "Repeated identical very long designer annotation".into(),
            )]
            .into_iter()
            .collect(),
        );
    }
    for width in [220., 300.] {
        let mut labels = Vec::new();
        for id in [&a, &b] {
            view.subject = SelectionSubject::Compound(
                CompoundSelection::new(view.members.clone(), Some(id.clone())).unwrap(),
            );
            state.selection = SelectionTarget::Native(Box::new(view.clone()));
            let before = state.selection.clone();
            let (_, text, _) = frame(
                &state,
                RectPx {
                    x: 400.,
                    y: 80.,
                    width,
                    height: 1200.,
                },
            );
            let label = text
                .iter()
                .find(|run| run.text.starts_with("Focus: "))
                .unwrap();
            assert!(
                label.text.contains(if id == &a { "10fe" } else { "10ff" }),
                "{}",
                label.text
            );
            labels.push(label.text.clone());
            assert_eq!(state.selection, before);
        }
        assert_ne!(labels[0], labels[1]);
    }
    let tail = "90000000-0000-4000-8000-000000000002".parse().unwrap();
    let mut path_labels = Vec::new();
    for ancestor in [
        "90000000-0000-4000-8000-000000000003",
        "90000000-0000-4000-8000-000000000004",
    ] {
        let mut id = a.clone();
        id.instance_path = vec![ancestor.parse().unwrap(), tail];
        view.members = [id.clone(), b.clone()].into_iter().collect();
        view.values.insert(id.clone(), view.values[&a].clone());
        view.subject = SelectionSubject::Compound(
            CompoundSelection::new(view.members.clone(), Some(id)).unwrap(),
        );
        state.selection = SelectionTarget::Native(Box::new(view.clone()));
        let before = state.selection.clone();
        let (_, text, _) = frame(
            &state,
            RectPx {
                x: 400.,
                y: 80.,
                width: 600.,
                height: 1200.,
            },
        );
        assert!(
            text.iter()
                .find(|run| run.text.starts_with("Focus: "))
                .unwrap()
                .text
                .contains("10fe")
        );
        path_labels.push(
            text.iter()
                .find(|run| run.text.starts_with("Sheet path … / "))
                .unwrap()
                .text
                .clone(),
        );
        assert_eq!(state.selection, before);
    }
    assert_ne!(path_labels[0], path_labels[1]);
    assert_eq!(fixture.source_bytes(), source);
}

#[test]
fn empty_lifetime_report_shows_reason_without_selected_badge_or_focus() {
    let (fixture, mut state) = selected_state();
    let source = fixture.source_bytes();
    let model = eda_engine::substrate::ProjectResolver::new(fixture.root())
        .resolve()
        .unwrap();
    let authority =
        datum_gui_protocol::selection_resolution::NativeSelectionResolution::capture(&model)
            .unwrap();
    let SelectionTarget::Native(previous) = &state.selection else {
        panic!("native fixture")
    };
    let mut report = datum_gui_protocol::selection_view::build_selection_view(
        &model,
        &authority,
        datum_gui_protocol::SelectionSubject::None,
        &state.scene,
        None,
        |_| true,
    )
    .unwrap();
    report.dropped = previous.members.clone();
    report.notice = Some("1 deleted or stale identities dropped; focus was not promoted".into());
    assert!(
        report.members.is_empty()
            && report.board_keys.is_empty()
            && report.schematic_keys.is_empty()
            && report.focus().is_none()
    );
    state.selection = SelectionTarget::Native(Box::new(report));
    let selection = state.selection.clone();
    let (_, text, _) = frame(
        &state,
        RectPx {
            x: 400.,
            y: 80.,
            width: 600.,
            height: 1000.,
        },
    );
    assert!(text.iter().any(|run| run.text == "No selection"));
    assert!(
        text.iter()
            .any(|run| run.text.contains("1 deleted or stale identities dropped"))
    );
    assert!(
        !text
            .iter()
            .any(|run| run.text == "SELECTED" || run.text.starts_with("Focus: "))
    );
    assert_eq!(state.selection, selection);
    assert_eq!(fixture.source_bytes(), source);
}
