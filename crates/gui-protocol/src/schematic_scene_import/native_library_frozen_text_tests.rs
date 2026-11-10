//! Old literal text and glyphs remain authentic; only explicit new use sees revision.
use super::*;
fn literal(text: &str, x: i64, y: i64, rotation: i32) -> Value {
    json!({"Text":{"text":text,"position":{"x":x,"y":y},"rotation":rotation}})
}
fn ink(scene: &crate::BoardReviewSceneV1, ids: &std::collections::BTreeSet<String>) -> Value {
    let graphics: Vec<_> = scene
        .board_graphics
        .iter()
        .filter(|g| ids.contains(&g.source_object_uuid))
        .collect();
    let texts: Vec<_> = scene
        .board_texts
        .iter()
        .filter(|t| {
            t.source_object_uuid
                .as_ref()
                .is_some_and(|id| ids.contains(id))
        })
        .collect();
    let text_ids: std::collections::BTreeSet<_> =
        texts.iter().map(|t| t.text_uuid.clone()).collect();
    let geometry: Vec<_> = scene
        .board_text_geometries
        .iter()
        .filter(|g| text_ids.contains(&g.text_uuid))
        .collect();
    let mut value = json!({"graphics":graphics,"texts":texts,"geometry":geometry});
    for rows in value.as_object_mut().unwrap().values_mut() {
        rows.as_array_mut()
            .unwrap()
            .sort_by_cached_key(Value::to_string);
    }
    value
}
fn revised_text(scene: &crate::BoardReviewSceneV1, id: Uuid) {
    let text = scene
        .board_texts
        .iter()
        .find(|t| {
            t.source_object_uuid.as_deref() == Some(id.to_string().as_str()) && t.text == "REVISED"
        })
        .expect("actual native literal Text primitive");
    assert_eq!(
        text.position,
        crate::PointNm {
            x: 9000000,
            y: 2000000
        }
    );
    assert_eq!(
        text.rotation_degrees, 180,
        "library90 plus explicit Symbol90"
    );
    assert!(
        scene
            .board_text_geometries
            .iter()
            .any(|g| g.text_uuid == text.text_uuid
                && (!g.glyphs.is_empty() || !g.strokes.is_empty() || !g.fills.is_empty())),
        "actual rendered glyph geometry, not just a label record"
    );
}
#[test]
fn symbol_text_lifecycle_preserves_old_body_pin_text_glyphs_and_projects_new_rotated_literal() {
    let (root, mut model, _, _, symbol) = fixture();
    edit(
        &mut model,
        &root,
        "symbols",
        symbol,
        json!({"graphic_additions":[literal("ORIGINAL",0,1000000,0)]}),
    );
    let original = raw(&model, "symbols", symbol);
    let sheet = Uuid::new_v4();
    let p = verb(
        &model,
        &root,
        "datum.schematic.create_sheet",
        json!({"sheet":sheet,"name":"Root"}),
    );
    commit_prepared(&mut model, &root, p).unwrap();
    let p = verb(
        &model,
        &root,
        "datum.schematic.place_symbol",
        json!({"sheet":sheet,"library_symbol":symbol,"reference":"OLD","value":"Old","x_nm":0,"y_nm":0}),
    );
    let old = p.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let before = placed_snapshot(&model, &root);
    edit(
        &mut model,
        &root,
        "symbols",
        symbol,
        json!({"graphic_additions":[literal("FIRST",0,0,0),literal("SECOND",0,0,0)]}),
    );
    assert_eq!(placed_snapshot(&model, &root), before);
    let multiple = raw(&model, "symbols", symbol);
    edit(
        &mut model,
        &root,
        "symbols",
        symbol,
        json!({"text_edits":[{"index":3,"previous":multiple["drawings"][3],"text":literal("REVISED",2000000,1000000,90)}],"text_removals":[{"index":2,"previous":multiple["drawings"][2]},{"index":4,"previous":multiple["drawings"][4]}]}),
    );
    let current = raw(&model, "symbols", symbol);
    assert_eq!(
        current["drawings"],
        json!([
            original["drawings"][0],
            original["drawings"][1],
            literal("REVISED", 2000000, 1000000, 90)
        ])
    );
    assert_eq!(current["pin_anchors"], original["pin_anchors"]);
    assert_eq!(current["fields"], original["fields"]);
    assert_eq!(placed_snapshot(&model, &root), before);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(placed_snapshot(&model, &root), before);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(placed_snapshot(&model, &root), before);
    let base = crate::native_schematic_scene::project_native_schematic_scene(&model, &root)
        .unwrap()
        .unwrap();
    assert!(
        base.board_texts
            .iter()
            .any(
                |t| t.source_object_uuid.as_deref() == Some(old.to_string().as_str())
                    && t.text == "ORIGINAL"
            )
    );
    let mut authoring = crate::schematic_authoring::SchematicAuthoring::from_model(&model).unwrap();
    authoring.sheet = Some(sheet);
    let old_ids: std::collections::BTreeSet<_> = std::iter::once(old.to_string())
        .chain(
            authoring.source.as_ref().unwrap().sheets[&sheet].symbols[&old]
                .pins
                .iter()
                .map(|p| p.uuid.to_string()),
        )
        .collect();
    let choice = authoring
        .choices
        .iter()
        .position(|c| c.id == symbol && c.explicit_part.is_none())
        .unwrap();
    assert!(authoring.select(choice));
    authoring.picker = false;
    authoring.position = Some(crate::PointNm { x: 10000000, y: 0 });
    authoring.rotation_deg = 90;
    let ghost = authoring.preview_scene(&base, &root).unwrap();
    assert_eq!(ink(&ghost, &old_ids), ink(&base, &old_ids));
    revised_text(&ghost, Uuid::nil());
    let p = verb(
        &model,
        &root,
        "datum.schematic.place_symbol",
        json!({"sheet":sheet,"library_symbol":symbol,"reference":"NEW","value":"New","x_nm":10000000,"y_nm":0,"rotation_deg":90}),
    );
    let new = p.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    for m in [&model, &ProjectResolver::new(&root).resolve().unwrap()] {
        assert_eq!(
            m.materialized_placed_symbol_definition(old, symbol)
                .unwrap(),
            Some(original.clone())
        );
        assert_eq!(
            m.materialized_placed_symbol_definition(new, symbol)
                .unwrap(),
            Some(current.clone())
        );
        let scene = crate::native_schematic_scene::project_native_schematic_scene(m, &root)
            .unwrap()
            .unwrap();
        assert_eq!(ink(&scene, &old_ids), ink(&base, &old_ids));
        revised_text(&scene, new);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn symbol_text_library_preview_contains_actual_rotated_layout_and_body_extents() {
    use crate::selection_geometry::Rational as R;
    use std::cmp::Ordering;
    let (root, mut model, unit, _, symbol) = fixture();
    edit(
        &mut model,
        &root,
        "symbols",
        symbol,
        json!({"graphic_additions":[literal("DRAFT",0,-1000000,90),literal("REMOVE",500000,2000000,0),literal("TILTED",-1000000,-1000000,37)]}),
    );
    let original = raw(&model, "symbols", symbol);
    let preview = crate::library_browser::symbol_geometry::preview::project(
        &original,
        Some(&raw(&model, "units", unit)),
    )
    .unwrap();
    for content in ["DRAFT", "REMOVE", "TILTED"] {
        assert!(preview.board_texts.iter().any(|t| t.text == content));
    }
    for geometry in &preview.board_text_geometries {
        let corners = crate::selection_geometry_catalog::layout_polygon(geometry)
            .expect("native ASCII text has captured layout geometry");
        for [x, y] in corners {
            assert_ne!(
                x.compare(R::integer(preview.bounds.min_x.into())).unwrap(),
                Ordering::Less,
                "left bearing is visible"
            );
            assert_ne!(
                x.compare(R::integer(preview.bounds.max_x.into())).unwrap(),
                Ordering::Greater,
                "full text advance is visible"
            );
            assert_ne!(
                y.compare(R::integer(preview.bounds.min_y.into())).unwrap(),
                Ordering::Less,
                "rotated top extent is visible"
            );
            assert_ne!(
                y.compare(R::integer(preview.bounds.max_y.into())).unwrap(),
                Ordering::Greater,
                "rotated bottom extent is visible"
            );
        }
    }
    for point in preview.board_graphics.iter().flat_map(|g| &g.path) {
        assert!((preview.bounds.min_x..=preview.bounds.max_x).contains(&point.x));
        assert!((preview.bounds.min_y..=preview.bounds.max_y).contains(&point.y));
    }
    assert_eq!(
        raw(&model, "symbols", symbol),
        original,
        "preview never changes source text or pose"
    );
    std::fs::remove_dir_all(root).unwrap();
}
