//! Canonically authored population: counts are objects, never DTO clones.
use super::*;
use eda_engine::{
    api::native_write::{self, BatchComposer, WriteProvenance},
    schematic::SchematicText,
    substrate::{CommitSource, Operation, ProjectResolver},
};

#[test]
#[ignore = "focused 100k canonical authoring/projection capacity proof"]
fn complete_native_schematic_population_survives_reopen_and_inspection_without_writes() {
    const COUNT: usize = 100_000;
    let (mut fixture, mut state) = selected_points_state();
    let authority = NativeSelectionResolution::capture(&fixture.model).unwrap();
    let sheet = authority
        .snapshot()
        .schematic_source()
        .unwrap()
        .sheets
        .values()
        .next()
        .unwrap()
        .uuid;
    let provenance = || {
        WriteProvenance::new(
            "selection-dense-fixture",
            CommitSource::Test,
            "100k authored text population",
        )
    };
    let mut operations = Vec::with_capacity(COUNT);
    let mut members = std::collections::BTreeSet::new();
    for ordinal in 0..COUNT {
        let text = SchematicText {
            uuid: uuid::Uuid::from_u128(0x10000000000040008000000000000000 + ordinal as u128),
            text: "I".into(),
            position: Point::new(
                (ordinal % 400) as i64 * 2_000_000,
                (ordinal / 400) as i64 * 2_000_000,
            ),
            rotation: 0,
        };
        let write = native_write::schematic_sheets::build_create_schematic_text(
            &fixture.model,
            provenance(),
            sheet,
            &text,
        )
        .unwrap();
        operations.extend(
            write
                .batch
                .operations
                .into_iter()
                .filter(|operation| matches!(operation, Operation::CreateSchematicText { .. })),
        );
        members.insert(Id {
            class: Class::SchematicText,
            id: text.uuid,
            instance_path: vec![],
        });
    }
    let write = BatchComposer::compose(&fixture.model, provenance())
        .push_ops(operations)
        .finish()
        .unwrap();
    let root = fixture.root().to_path_buf();
    native_write::commit_prepared(&mut fixture.model, &root, write).unwrap();
    fixture.model = ProjectResolver::new(fixture.root()).resolve().unwrap();
    let source = fixture.source_bytes();
    let authority = NativeSelectionResolution::capture(&fixture.model).unwrap();
    assert_eq!(
        authority.snapshot().schematic_source().unwrap().sheets[&sheet]
            .texts
            .len(),
        COUNT
    );
    let schematic =
        datum_gui_protocol::project_native_schematic_scene(&fixture.model, fixture.root())
            .unwrap()
            .unwrap();
    state.scene.source_revision = fixture.model.model_revision.0.clone();
    let subject = SelectionSubject::Compound(
        datum_gui_protocol::CompoundSelection::new(members.clone(), None).unwrap(),
    );
    let view = build_selection_view(
        &fixture.model,
        &authority,
        subject.clone(),
        &state.scene,
        Some(&schematic),
        |_| true,
    )
    .unwrap();
    assert_eq!(view.members, members);
    assert_eq!(view.subject, subject);
    assert!(view.hidden.is_empty());
    assert!(view.unprojected.is_empty());
    assert_eq!(view.schematic_keys.len(), COUNT);
    assert_eq!(schematic.board_texts.len(), COUNT);
    assert_eq!(schematic.board_text_geometries.len(), COUNT);
    let datum_gui_protocol::selection_view::InspectionValue::Common(count) =
        view.field("member_count")
    else {
        panic!("complete aggregate count");
    };
    assert_eq!(count.as_u64(), Some(COUNT as u64));
    state.selection = SelectionTarget::Native(Box::new(view));
    state.schematic_scene = Some(schematic);
    state.ui.layout.open_beside_root(
        datum_gui_protocol::PaneContent::Schematic,
        datum_gui_protocol::SplitOrientation::Vertical,
        0.5,
        true,
    );
    let retained =
        RetainedScene::from_workspace_schematic_for_surface(&state, 1280, 800, 1.0).unwrap();
    let SelectionTarget::Native(view) = &state.selection else {
        unreachable!()
    };
    let owned = retained
        .presentation
        .quads
        .iter()
        .filter_map(|span| span.key.as_ref())
        .filter(|key| view.schematic_keys.contains(*key))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        owned.len(),
        COUNT,
        "every authored glyph has retained source incidence"
    );
    eprintln!(
        "authored_texts={COUNT}; retained_vertices={}; retained_strokes={}; distinct_owned_keys={}",
        retained.world_vertices().len(),
        retained.world_strokes().len(),
        owned.len()
    );
    assert_eq!(fixture.source_bytes(), source);
}
