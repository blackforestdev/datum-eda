//! Read-only integration over a canonically authored repeated-instance project.
use super::*;
use crate::selection_resolution::{NativeSelectionResolution, SelectionAuthority};
use crate::{AuthoredSelectionClass as Class, SelectionSubject};

#[test]
#[ignore = "requires DATUM_HIERARCHY_TEST_PROJECT canonical repeated-instance project"]
fn hierarchy_membership_selection_inspection_and_stale_projection() {
    let root = std::path::PathBuf::from(
        std::env::var_os("DATUM_HIERARCHY_TEST_PROJECT")
            .expect("supply canonically authored hierarchy project"),
    );
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let interfaces: Vec<_> = authority
        .hierarchy_targets()
        .values()
        .filter(|target| target.identity.class == Class::SheetInterface)
        .collect();
    let (a, b) = interfaces
        .iter()
        .enumerate()
        .find_map(|(i, a)| {
            interfaces[i + 1..]
                .iter()
                .find(|b| a.port == b.port && a.parent == b.parent && a.child != b.child)
                .map(|b| (*a, *b))
        })
        .expect("fixture must bind one actual port to repeated child instances");
    assert_eq!(a.definition, b.definition);
    assert_eq!(a.child.source_sheet, b.child.source_sheet);
    assert_ne!(a.identity, b.identity);
    assert_eq!(a.position, b.position);
    assert!(a.child_label.is_some() && a.child_label == b.child_label);
    assert!(a.unavailable.is_none() && b.unavailable.is_none());
    assert!(authority.semantic_subject_for_origin(&a.identity).is_err());
    let mut documents = SchematicDocuments::from_source(
        model.project.project_id,
        model.model_revision.clone(),
        "Hierarchy".into(),
        authority.snapshot().schematic_source().unwrap().clone(),
    )
    .unwrap();
    documents.prepare_scene(&a.parent, &root, &model).unwrap();
    documents.prepare_scene(&a.child, &root, &model).unwrap();
    documents.prepare_scene(&b.child, &root, &model).unwrap();
    let scene = documents.scenes[&a.parent].as_ref();
    let bindings = crate::selection_scene::SceneSelectionBindings::capture(scene, &authority);
    assert_eq!(bindings.get(&a.graphic_key()).unwrap().acquired, a.identity);
    assert_eq!(bindings.get(&b.graphic_key()).unwrap().acquired, b.identity);
    let mut menu = crate::selection_menu::SelectionMenu::capture(
        &authority,
        vec![
            (a.identity.clone(), a.identity.clone()),
            (b.identity.clone(), b.identity.clone()),
        ],
        a.position,
        crate::load_fixture_workspace_state().ui.filters,
    );
    menu.source_scene = scene.scene_id.clone();
    menu.schematic = true;
    menu.bind_presentation(scene, &bindings, &authority);
    for target in [a, b] {
        let item = menu
            .items
            .iter()
            .find(|item| {
                item.command == crate::selection_menu::Command::Object(target.identity.clone())
            })
            .unwrap();
        assert!(
            item.label
                .starts_with(&format!("Select {} ·", target.instance_name))
        );
        assert!(item.label.contains(target.port_name.as_deref().unwrap()));
        assert!(item.label.ends_with(&target.instance.to_string()[28..]));
    }
    let source_port = crate::AuthoredSelectionIdentity {
        class: Class::Port,
        id: a.port.unwrap(),
        instance_path: a.parent.instance_path.clone(),
    };
    assert!(
        bindings
            .iter()
            .any(|(_, binding)| binding.acquired == source_port)
    );
    let mut board = crate::load_fixture_workspace_state().scene;
    board.project_uuid = model.project.project_id.to_string();
    board.source_revision = model.model_revision.0.clone();
    let view = crate::selection_view::build_selection_view_for_documents(
        &model,
        &authority,
        SelectionSubject::Object(a.identity.clone()),
        &board,
        &[
            scene,
            &documents.scenes[&a.child],
            &documents.scenes[&b.child],
        ],
        |_| true,
    )
    .unwrap();
    assert_eq!(view.members, BTreeSet::from([a.identity.clone()]));
    assert!(view.inspector_title().contains(&a.instance_name));
    assert_eq!(
        view.values[&a.identity]["parent_port_uuid"],
        serde_json::json!(a.port.unwrap())
    );
    assert_eq!(
        view.values[&a.identity]["sheet_occurrence_path"],
        serde_json::json!(a.child.instance_path)
    );
    let keys = view
        .projection_keys(scene, crate::PaneContent::Schematic)
        .unwrap();
    assert!(keys.contains(&a.graphic_key()));
    assert!(keys.contains(&a.text_key()));
    assert_eq!(bindings.get(&a.text_key()).unwrap().acquired, a.identity);
    assert_eq!(bindings.get(&b.text_key()).unwrap().acquired, b.identity);
    assert!(!keys.contains(&b.text_key()));
    assert!(!keys.contains(&b.graphic_key()));
    let geometry =
        crate::selection_geometry_catalog::qualification_catalog(&model, &authority, scene);
    assert!(geometry[&a.identity].is_ok());
    for body in authority
        .hierarchy_targets()
        .values()
        .filter(|target| target.port.is_none())
    {
        assert!(authority.authored_contains(&body.identity));
        if body.parent == a.parent {
            assert!(matches!(
                geometry[&body.identity],
                Ok(crate::selection_geometry_catalog::QualificationGeometry::Filled(_))
            ));
        }
    }
    let mut stale = scene.clone();
    stale.source_revision.push_str("-stale");
    assert!(
        crate::selection_scene::SceneSelectionBindings::capture(&stale, &authority)
            .iter()
            .next()
            .is_none()
    );
    let mut foreign = scene.clone();
    foreign.schematic_document.as_mut().unwrap().instance_path = a.child.instance_path.clone();
    assert!(
        crate::selection_scene::SceneSelectionBindings::capture(&foreign, &authority)
            .get(&a.graphic_key())
            .is_none()
    );
    let mut layout = crate::WorkspaceLayout::board_schematic();
    let pane_a = documents.open_beside(&mut layout, &a.child).unwrap();
    let pane_b = documents.open_beside(&mut layout, &b.child).unwrap();
    assert_ne!(pane_a, pane_b);
    assert_eq!(documents.open_beside(&mut layout, &a.child), Some(pane_a));

    // Negative controls alter only an in-memory copy, never the source project.
    let mut missing = authority.snapshot().schematic_source().unwrap().clone();
    missing
        .sheets
        .get_mut(&a.parent.source_sheet)
        .unwrap()
        .ports
        .remove(&a.port.unwrap());
    let captured = hierarchy::capture(
        model.project.project_id,
        model.model_revision.clone(),
        &missing,
    )
    .unwrap();
    assert!(!captured.contains_key(&a.identity));
    assert!(
        captured
            .values()
            .filter(|target| target.instance == a.instance)
            .all(|target| target.unavailable.is_some())
    );
    let mut unbound = authority.snapshot().schematic_source().unwrap().clone();
    unbound
        .sheet_instances
        .get_mut(&a.instance)
        .unwrap()
        .ports
        .clear();
    let captured = hierarchy::capture(
        model.project.project_id,
        model.model_revision.clone(),
        &unbound,
    )
    .unwrap();
    assert!(
        captured
            .values()
            .find(|target| target.instance == a.instance && target.port.is_none())
            .unwrap()
            .unavailable
            .is_some()
    );
    assert!(captured[&b.identity].unavailable.is_none());
    let mut ambiguous = authority.snapshot().schematic_source().unwrap().clone();
    // The same source port repeated is an invalid mapping even while B remains valid.
    ambiguous
        .sheet_instances
        .get_mut(&a.instance)
        .unwrap()
        .ports
        .push(a.port.unwrap());
    let captured = hierarchy::capture(
        model.project.project_id,
        model.model_revision.clone(),
        &ambiguous,
    )
    .unwrap();
    assert!(captured[&a.identity].unavailable.is_some());
    assert!(captured[&b.identity].unavailable.is_none());
    missing.sheet_definitions.remove(&a.definition);
    assert!(
        hierarchy::capture(
            model.project.project_id,
            model.model_revision.clone(),
            &missing
        )
        .is_err()
    );
}
