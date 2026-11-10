//! Actual old placements survive Rect append/correction/removal; new use is explicit.
use super::*;
fn rect(max: i64, width: i64) -> Value {
    json!({"Rect":{"min":{"x":-500000,"y":-500000},"max":{"x":500000,"y":max},"width":width}})
}
#[test]
fn rect_lifecycle_keeps_old_ink_during_ghost_history_cold_and_later_explicit_placement() {
    let (root, mut model, _, _, symbol) = fixture();
    edit(
        &mut model,
        &root,
        "symbols",
        symbol,
        json!({"graphic_additions":[rect(500000,100000)]}),
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
        json!({"graphic_additions":[rect(500000,200000),rect(1000000,300000)]}),
    );
    let multiple = raw(&model, "symbols", symbol);
    assert_eq!(placed_snapshot(&model, &root), before);
    edit(
        &mut model,
        &root,
        "symbols",
        symbol,
        json!({"rect_edits":[{"index":3,"previous":multiple["drawings"][3],"rect":rect(750000,150000)}],"rect_removals":[{"index":2,"previous":multiple["drawings"][2]},{"index":4,"previous":multiple["drawings"][4]}]}),
    );
    let current = raw(&model, "symbols", symbol);
    assert_eq!(
        current["drawings"],
        json!([
            original["drawings"][0],
            original["drawings"][1],
            rect(750000, 150000)
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
    let mut authoring = crate::schematic_authoring::SchematicAuthoring::from_model(&model).unwrap();
    authoring.sheet = Some(sheet);
    let choice = authoring
        .choices
        .iter()
        .position(|c| c.id == symbol && c.explicit_part.is_none())
        .unwrap();
    assert!(authoring.select(choice));
    authoring.picker = false;
    authoring.position = Some(crate::PointNm { x: 10000000, y: 0 });
    let ghost = authoring.preview_scene(&base, &root).unwrap();
    let expected_path = vec![
        crate::PointNm {
            x: 9500000,
            y: -500000,
        },
        crate::PointNm {
            x: 10500000,
            y: -500000,
        },
        crate::PointNm {
            x: 10500000,
            y: 750000,
        },
        crate::PointNm {
            x: 9500000,
            y: 750000,
        },
        crate::PointNm {
            x: 9500000,
            y: -500000,
        },
    ];
    assert!(
        ghost
            .board_graphics
            .iter()
            .any(|g| g.source_object_uuid == Uuid::nil().to_string()
                && g.primitive_kind == "polyline"
                && g.width_nm == Some(150000)
                && g.path == expected_path)
    );
    let old_ids: std::collections::BTreeSet<_> = std::iter::once(old.to_string())
        .chain(
            authoring.source.as_ref().unwrap().sheets[&sheet].symbols[&old]
                .pins
                .iter()
                .map(|p| p.uuid.to_string()),
        )
        .collect();
    let ink = |scene: &crate::BoardReviewSceneV1| {
        let graphics: Vec<_> = scene
            .board_graphics
            .iter()
            .filter(|g| old_ids.contains(&g.source_object_uuid))
            .collect();
        let texts: Vec<_> = scene
            .board_texts
            .iter()
            .filter(|t| {
                t.source_object_uuid
                    .as_ref()
                    .is_some_and(|id| old_ids.contains(id))
            })
            .collect();
        let text_ids: std::collections::BTreeSet<_> =
            texts.iter().map(|t| t.text_uuid.clone()).collect();
        let glyphs: Vec<_> = scene
            .board_text_geometries
            .iter()
            .filter(|g| text_ids.contains(&g.text_uuid))
            .collect();
        let mut value = json!({"graphics":graphics,"texts":texts,"glyphs":glyphs});
        for rows in value.as_object_mut().unwrap().values_mut() {
            rows.as_array_mut()
                .unwrap()
                .sort_by_cached_key(Value::to_string);
        }
        value
    };
    assert_eq!(ink(&ghost), ink(&base));
    let p = verb(
        &model,
        &root,
        "datum.schematic.place_symbol",
        json!({"sheet":sheet,"library_symbol":symbol,"reference":"NEW","value":"New","x_nm":10000000,"y_nm":0}),
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
        assert_eq!(ink(&scene), ink(&base));
        assert!(
            scene
                .board_graphics
                .iter()
                .any(|g| g.source_object_uuid == new.to_string()
                    && g.width_nm == Some(150000)
                    && g.path == expected_path)
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}
