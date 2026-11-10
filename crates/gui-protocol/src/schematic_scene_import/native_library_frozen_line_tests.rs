//! Real old placements retain graphic projection; a later placement consumes new Lines.
use super::*;
#[test]
fn line_authoring_preserves_old_scene_through_history_cold_and_new_placement() {
    let (root, mut model, _, _, symbol) = fixture();
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
        json!({"sheet":sheet,"library_symbol":symbol,"reference":"U1","value":"Old","x_nm":10000000,"y_nm":20000000}),
    );
    let old_id = p.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let original = raw(&model, "symbols", symbol);
    let old_scene = placed_snapshot(&model, &root);
    let mut changed = original["drawings"][1].clone();
    changed["Line"]["to"]["x"] = json!(6000000);
    let added = json!({"Line":{"from":{"x":-500000,"y":-500000},"to":{"x":500000,"y":500000},"width":200000}});
    edit(
        &mut model,
        &root,
        "symbols",
        symbol,
        json!({"line_edits":[{"index":1,"previous":original["drawings"][1],"line":changed}],"line_additions":[added]}),
    );
    assert_eq!(placed_snapshot(&model, &root), old_scene);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(placed_snapshot(&model, &root), old_scene);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(placed_snapshot(&model, &root), old_scene);
    let current = raw(&model, "symbols", symbol);
    edit(
        &mut model,
        &root,
        "symbols",
        symbol,
        json!({"line_removals":[{"index":1,"previous":current["drawings"][1]}]}),
    );
    assert_eq!(placed_snapshot(&model, &root), old_scene);
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(placed_snapshot(&model, &root), old_scene);
    let current = raw(&model, "symbols", symbol);
    assert_eq!(current["drawings"].as_array().unwrap().len(), 2);
    assert_eq!(current["drawings"][1], added);
    let p = verb(
        &model,
        &root,
        "datum.schematic.place_symbol",
        json!({"sheet":sheet,"library_symbol":symbol,"reference":"U2","value":"New","x_nm":30000000,"y_nm":20000000}),
    );
    let new_id = p.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    for model in [&model, &ProjectResolver::new(&root).resolve().unwrap()] {
        assert_eq!(
            model
                .materialized_placed_symbol_definition(old_id, symbol)
                .unwrap(),
            Some(original.clone())
        );
        assert_eq!(
            model
                .materialized_placed_symbol_definition(new_id, symbol)
                .unwrap(),
            Some(current.clone())
        );
        let scene = crate::native_schematic_scene::project_native_schematic_scene(model, &root)
            .unwrap()
            .unwrap();
        let graphics: Vec<_> = scene
            .board_graphics
            .iter()
            .filter(|g| g.source_object_uuid == new_id.to_string())
            .collect();
        assert!(graphics.iter().any(|g| g.primitive_kind == "line"
            && g.width_nm == Some(200000)
            && g.path
                == vec![
                    crate::PointNm {
                        x: 29500000,
                        y: 19500000
                    },
                    crate::PointNm {
                        x: 30500000,
                        y: 20500000
                    }
                ]));
        let old_graphics: Vec<_> = scene
            .board_graphics
            .iter()
            .filter(|g| g.source_object_uuid == old_id.to_string())
            .collect();
        assert!(old_graphics.iter().any(|g| g.primitive_kind == "line"
            && g.width_nm == Some(1)
            && g.path
                == vec![
                    crate::PointNm {
                        x: 10000001,
                        y: 20000002
                    },
                    crate::PointNm {
                        x: 10000003,
                        y: 20000004
                    }
                ]));
    }
}

fn placed_ink(
    scene: &crate::BoardReviewSceneV1,
    placed: &eda_engine::schematic::PlacedSymbol,
) -> Value {
    let ids: std::collections::BTreeSet<_> = std::iter::once(placed.uuid.to_string())
        .chain(placed.pins.iter().map(|pin| pin.uuid.to_string()))
        .collect();
    let value = json!({"graphics":scene.board_graphics,"texts":scene.board_texts,"geometry":scene.board_text_geometries});
    let matches = |row: &Value| {
        row["source_object_uuid"]
            .as_str()
            .is_some_and(|id| ids.contains(id))
            || row["object_id"]
                .as_str()
                .is_some_and(|id| id.contains(&placed.uuid.to_string()))
    };
    let texts: Vec<_> = value["texts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| matches(row))
        .cloned()
        .collect();
    let text_ids: std::collections::BTreeSet<_> = texts
        .iter()
        .filter_map(|row| row["text_uuid"].as_str())
        .collect();
    let geometry: Vec<_> = value["geometry"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["text_uuid"]
                .as_str()
                .is_some_and(|id| text_ids.contains(id))
        })
        .cloned()
        .collect();
    let graphics: Vec<_> = value["graphics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| matches(row))
        .cloned()
        .collect();
    let mut result = json!({"graphics":graphics,"texts":texts,"geometry":geometry});
    for values in result.as_object_mut().unwrap().values_mut() {
        values
            .as_array_mut()
            .unwrap()
            .sort_by_cached_key(Value::to_string);
    }
    result
}
#[test]
fn warm_preview_preserves_each_old_definition_for_placement_wire_marker_and_moved_pose() {
    let (root, mut model, _, _, symbol) = fixture();
    let mut original = raw(&model, "symbols", symbol);
    original["drawings"] = json!([{"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}}]);
    let prepared = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new("pool", "symbols", symbol),
            object: original,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let sheet = Uuid::new_v4();
    let p = verb(
        &model,
        &root,
        "datum.schematic.create_sheet",
        json!({"sheet":sheet,"name":"Root"}),
    );
    commit_prepared(&mut model, &root, p).unwrap();
    let mut old_ids = Vec::new();
    for (reference, x) in [("OLD1", 0), ("OLD2", 4000000)] {
        let p = verb(
            &model,
            &root,
            "datum.schematic.place_symbol",
            json!({"sheet":sheet,"library_symbol":symbol,"reference":reference,"value":"Old","x_nm":x,"y_nm":0}),
        );
        old_ids.push(p.primary_object_id.unwrap());
        commit_prepared(&mut model, &root, p).unwrap();
    }
    let added = json!({"Line":{"from":{"x":-500000,"y":-500000},"to":{"x":500000,"y":750000},"width":150000}});
    edit(
        &mut model,
        &root,
        "symbols",
        symbol,
        json!({"line_additions":[added]}),
    );
    let base = crate::native_schematic_scene::project_native_schematic_scene(&model, &root)
        .unwrap()
        .unwrap();
    let mut authoring = crate::schematic_authoring::SchematicAuthoring::from_model(&model).unwrap();
    authoring.sheet = Some(sheet);
    let captured = authoring.source.as_ref().unwrap().sheets[&sheet]
        .symbols
        .clone();
    let assert_old = |scene: &crate::BoardReviewSceneV1| {
        for id in &old_ids {
            assert_eq!(
                placed_ink(scene, &captured[id]),
                placed_ink(&base, &captured[id]),
                "old graphics/pins/texts at {id}"
            );
        }
    };
    assert_old(
        &authoring
            .project_source(authoring.source.as_ref().unwrap(), sheet, &base, &root)
            .unwrap(),
    );
    let mut wire = authoring.clone();
    wire.wire = true;
    wire.position = Some(crate::PointNm {
        x: 8000000,
        y: 6000000,
    });
    wire.vertices = vec![crate::PointNm {
        x: 6000000,
        y: 6000000,
    }];
    let preview = wire.preview_scene(&base, &root).unwrap();
    assert_old(&preview);
    assert!(
        preview
            .board_graphics
            .iter()
            .any(|g| g.object_id.starts_with("schematic-wire:")
                && g.path
                    == vec![
                        crate::PointNm {
                            x: 6000000,
                            y: 6000000
                        },
                        crate::PointNm {
                            x: 8000000,
                            y: 6000000
                        }
                    ])
    );
    let mut marker = authoring.clone();
    marker.marker = Some(crate::schematic_authoring::MarkerKind::Junction);
    marker.position = Some(crate::PointNm {
        x: 8000000,
        y: 8000000,
    });
    assert_old(&marker.preview_scene(&base, &root).unwrap());
    let choice = authoring
        .choices
        .iter()
        .position(|c| c.id == symbol && c.explicit_part.is_none())
        .unwrap();
    let mut placement = authoring.clone();
    assert!(placement.select(choice));
    placement.picker = false;
    placement.position = Some(crate::PointNm { x: 10000000, y: 0 });
    let ghost = placement.preview_scene(&base, &root).unwrap();
    assert_old(&ghost);
    assert!(
        ghost
            .board_graphics
            .iter()
            .any(|g| g.source_object_uuid == Uuid::nil().to_string()
                && g.width_nm == Some(150000)
                && g.path
                    == vec![
                        crate::PointNm {
                            x: 9500000,
                            y: -500000
                        },
                        crate::PointNm {
                            x: 10500000,
                            y: 750000
                        }
                    ])
    );
    let moved = authoring.move_bases[&old_ids[0]]
        .as_ref()
        .unwrap()
        .destination_with_pose(
            eda_engine::ir::geometry::Point::new(20000000, 20000000),
            Some(90),
            Some(false),
        )
        .unwrap()
        .symbol;
    let mut source = authoring.source.clone().unwrap();
    source
        .sheets
        .get_mut(&sheet)
        .unwrap()
        .symbols
        .insert(moved.uuid, moved);
    let moved_scene = authoring
        .project_source(&source, sheet, &base, &root)
        .unwrap();
    assert_eq!(
        placed_ink(&moved_scene, &captured[&old_ids[1]]),
        placed_ink(&base, &captured[&old_ids[1]])
    );
    let rectangle = moved_scene
        .board_graphics
        .iter()
        .find(|g| g.object_id == format!("schematic-symbol:{}:library:0", old_ids[0]))
        .unwrap();
    assert_eq!(
        rectangle.path,
        vec![
            crate::PointNm {
                x: 21000000,
                y: 19000000
            },
            crate::PointNm {
                x: 21000000,
                y: 21000000
            },
            crate::PointNm {
                x: 19000000,
                y: 21000000
            },
            crate::PointNm {
                x: 19000000,
                y: 19000000
            },
            crate::PointNm {
                x: 21000000,
                y: 19000000
            }
        ]
    );
    for missing in [false, true] {
        let mut invalid = authoring.clone();
        if missing {
            invalid.placed_symbol_definitions.remove(&old_ids[0]);
        } else {
            invalid
                .placed_symbol_definitions
                .insert(old_ids[0], Err("Captured source unavailable".into()));
        }
        let scene = invalid
            .project_source(invalid.source.as_ref().unwrap(), sheet, &base, &root)
            .unwrap();
        assert!(scene.board_texts.iter().any(|text| text.source_object_uuid
            == Some(old_ids[0].to_string())
            && text.text == "Library source unavailable"));
        assert!(
            !scene
                .board_graphics
                .iter()
                .any(|g| g.source_object_uuid == old_ids[0].to_string()
                    && g.width_nm == Some(150000))
        );
        assert_eq!(
            placed_ink(&scene, &captured[&old_ids[1]]),
            placed_ink(&base, &captured[&old_ids[1]])
        );
    }
}
