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

#[test]
#[ignore = "read-only diagnosis using S5A_DENSE_NATIVE_ROOT preserved canonical fixture"]
fn existing_native_dense_fixture_reports_retained_layout_without_authored_writes() {
    use datum_gui_protocol::selection_resolution::SelectionAuthority;
    let root = std::path::PathBuf::from(
        std::env::var("S5A_DENSE_NATIVE_ROOT").expect("explicit preserved fixture path"),
    );
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let mut files: Vec<_> = model.source_shards.iter().map(|s| s.path.clone()).collect();
    files.push(eda_engine::substrate::transaction_journal_path(&root));
    let before: Vec<_> = files.iter().map(|p| std::fs::read(p).unwrap()).collect();
    let native = NativeSelectionResolution::capture(&model).unwrap();
    let members: std::collections::BTreeSet<_> = native
        .authored_identities()
        .iter()
        .filter(|id| id.class == Class::SchematicText)
        .cloned()
        .collect();
    assert_eq!(members.len(), 100_000);
    let subject = SelectionSubject::Compound(
        datum_gui_protocol::CompoundSelection::new(members.clone(), None).unwrap(),
    );
    assert_eq!(native.members(&subject).unwrap(), members);
    let source = native.snapshot().schematic_source().unwrap();
    assert_eq!(
        source
            .sheets
            .values()
            .flat_map(|s| s.texts.values())
            .filter(|t| t.text == "I")
            .count(),
        100_000
    );
    let scene = datum_gui_protocol::project_native_schematic_scene(&model, &root)
        .unwrap()
        .unwrap();
    assert_eq!(scene.board_text_geometries.len(), 100_000);
    let assets: std::collections::BTreeMap<_, _> = scene
        .glyph_mesh_assets
        .iter()
        .map(|a| (a.handle, a))
        .collect();
    let mut triangles = 0_usize;
    for text in &scene.board_text_geometries {
        assert!(text.world_transform_nm.is_some());
        assert_eq!(text.glyphs.len(), 1);
        for glyph in &text.glyphs {
            let asset = assets[&glyph.glyph_handle];
            assert_eq!(asset.indices.len() % 3, 0);
            assert!(
                asset
                    .indices
                    .iter()
                    .all(|i| (*i as usize) < asset.vertices.len())
            );
            triangles = triangles.checked_add(asset.indices.len() / 3).unwrap();
        }
    }
    // Quad construction stores [a,b,c,c]. Retained expansion keeps only
    // three vertices; its admission currently reserves six. This diagnostic
    // distinguishes actual payload from that upper bound and never credits
    // construction refusal as positive dense acceptance.
    let quad_bytes = triangles.checked_mul(std::mem::size_of::<Quad>()).unwrap();
    let vertex_bytes = triangles
        .checked_mul(3 * std::mem::size_of::<Vertex>())
        .unwrap();
    let source_span_bytes = scene
        .board_text_geometries
        .len()
        .checked_mul(std::mem::size_of::<crate::presentation_ranges::Span>())
        .unwrap();
    let source_key_bytes: usize = scene
        .board_text_geometries
        .iter()
        .map(|t| t.object_id.len())
        .sum();
    let final_bare_lower_bound = vertex_bytes + source_span_bytes + source_key_bytes;
    let unique_vertices: usize = scene
        .board_text_geometries
        .iter()
        .flat_map(|t| &t.glyphs)
        .map(|g| assets[&g.glyph_handle].vertices.len())
        .sum();
    eprintln!(
        "authored_texts={}; glyph_assets={}; actual_glyph_triangles={triangles}; quad_payload_bytes={quad_bytes}; actual_compacted_vertex_payload_bytes={vertex_bytes}; admission_vertex_upper_bound_bytes={}; required_source_span_bytes={source_span_bytes}; required_source_key_bytes={source_key_bytes}; final_bare_lower_bound_bytes={final_bare_lower_bound}; exact_unique_mesh_vertex_bytes={}; exact_mesh_index_bytes={}; document_limit_bytes={}; allocation_headers_hit_index_and_other_geometry_additional=true",
        members.len(),
        assets.len(),
        vertex_bytes * 2,
        unique_vertices * std::mem::size_of::<Vertex>(),
        triangles * 3 * std::mem::size_of::<u32>(),
        crate::retained_scene_owner::document_cpu::DOCUMENT_LIMIT
    );
    let scope = crate::cpu_alloc::Scope::new("native-dense-hit-diagnostic");
    let budget = crate::retained_scene_owner::document_cpu::for_scene(&scene.scene_id);
    let hit_result = crate::retained_scene_owner::document_cpu::with_constructor(&scope, || {
        let regions = crate::coordinate_hit::schematic_hit_regions(&scene, |bytes| {
            crate::retained_scene_owner::document_cpu::admit_constructor_allocation(
                &budget,
                &scope,
                bytes,
                crate::retained_scene_owner::document_cpu::DOCUMENT_LIMIT,
                "diagnostic hit regions",
            )
        })?;
        RetainedScene::admitted_hit_index(
            regions,
            &budget,
            &scope,
            crate::retained_scene_owner::document_cpu::DOCUMENT_LIMIT,
        )
    });
    match hit_result {
        Ok(index) => eprintln!(
            "standalone_hit_index_payload_and_tracking={:?}; standalone_hit_index_heap={:?}; not_full_scene_admission=true",
            scope.usage(),
            index.heap_bytes_with(
                |target| Some(crate::cpu_alloc::heap::capacity_bytes::<u8>(
                    target.capacity()
                )),
                |layout| Some(crate::cpu_alloc::heap::allocation_bytes(layout))
            )
        ),
        Err(error) => {
            eprintln!("standalone_hit_index_refusal={error}; not_full_scene_admission=true")
        }
    }
    for (path, bytes) in files.iter().zip(before) {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}
