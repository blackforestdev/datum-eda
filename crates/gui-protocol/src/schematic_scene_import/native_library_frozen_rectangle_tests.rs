//! Numeric body source corrections preserve actual old rendering and later explicit use.
use super::*;
use eda_engine::substrate::Operation;
fn store(model: &mut DesignModel, root: &std::path::Path, symbol: Uuid, object: Value) {
    let p = library::build_pool_library_write(
        model,
        provenance(),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new("pool", "symbols", symbol),
            object,
        }],
    )
    .unwrap();
    commit_prepared(model, root, p).unwrap();
}
fn component(
    model: &mut DesignModel,
    root: &std::path::Path,
    unit: Uuid,
    symbol: Uuid,
) -> (Uuid, Uuid) {
    let (entity, gate, package, part) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let p = library::build_pool_library_write(
        model,
        provenance(),
        None,
        vec![
            PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "entities", entity),
                object: library::pool_entity_payload(
                    entity,
                    gate,
                    unit,
                    symbol,
                    "Component",
                    "U",
                    "Maker",
                    "A",
                ),
            },
            PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "packages", package),
                object: library::pool_package_payload(package, "Body", json!({})),
            },
            PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "parts", part),
                object: library::pool_part_payload(
                    part, entity, package, "MPN", "Maker", "Value", "", "", "Active",
                ),
            },
        ],
    )
    .unwrap();
    commit_prepared(model, root, p).unwrap();
    (part, gate)
}
#[test]
fn saved_symbol_rectangle_mixed_body_keeps_bound_and_birth_scenes_and_new_explicit_body() {
    for bound in [false, true] {
        let (root, mut model, unit, _, symbol) = fixture();
        let mut original = raw(&model, "symbols", symbol);
        original["drawings"].as_array_mut().unwrap().insert(1,json!({"Rect":{"min":{"x":-2000000,"y":-1000000,"vendor":[3,1,3]},"max":{"x":2000000,"y":1000000,"optional":null},"width":100000,"unknown":{"keep":true}}}));
        store(&mut model, &root, symbol, original.clone());
        let chosen_component = bound.then(|| component(&mut model, &root, unit, symbol));
        let sheet = Uuid::new_v4();
        let p = verb(
            &model,
            &root,
            "datum.schematic.create_sheet",
            json!({"sheet":sheet,"name":"Root"}),
        );
        commit_prepared(&mut model, &root, p).unwrap();
        let mut old = vec![];
        for (reference, x, rotation, mirrored) in
            [("U1", 10000000, 0, false), ("U2", 20000000, 90, true)]
        {
            let mut p = verb(
                &model,
                &root,
                "datum.schematic.place_symbol",
                json!({"sheet":sheet,"library_symbol":symbol,"reference":reference,"value":"original","x_nm":x,"y_nm":20000000,"rotation_deg":rotation,"mirrored":mirrored}),
            );
            old.push(p.primary_object_id.unwrap());
            if bound {
                add_symbol_binding(&mut p, symbol, model.objects[&symbol].object_revision.0);
            }
            commit_prepared(&mut model, &root, p).unwrap();
        }
        assert_eq!(!model.component_instances.is_empty(), bound);
        let before = placed_snapshot(&model, &root);
        let mut corrected = original.clone();
        let rect = &mut corrected["drawings"][1]["Rect"];
        rect["min"]["x"] = json!(-3000000);
        rect["min"]["y"] = json!(-2000000);
        rect["max"]["x"] = json!(4000000);
        rect["max"]["y"] = json!(3000000);
        rect["width"] = json!(200000);
        edit(
            &mut model,
            &root,
            "symbols",
            symbol,
            json!({"drawings":corrected["drawings"]}),
        );
        let check = |model: &DesignModel| {
            assert_eq!(
                placed_snapshot(model, &root),
                before,
                "old bodies/ordered graphics/pins/annotations/CI remain exact"
            );
            for id in &old {
                assert_eq!(
                    model
                        .materialized_placed_symbol_definition(*id, symbol)
                        .unwrap(),
                    Some(original.clone())
                );
            }
        };
        check(&model);
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        check(&model);
        assert_eq!(raw(&model, "symbols", symbol), original);
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        check(&model);
        model = ProjectResolver::new(&root).resolve().unwrap();
        check(&model);
        assert_eq!(raw(&model, "symbols", symbol), corrected);
        let Some((part, gate)) = chosen_component else {
            // Birth-only users have completed their old-render preservation case.
            // The later exact-source-guard consumer is the existing explicit Part/gate path.
            std::fs::remove_dir_all(root).unwrap();
            continue;
        };
        let selected = native_write::schematic_library::part_placement::PartGateSelection::capture(
            &model, part, gate, symbol,
        )
        .unwrap();
        let revision = model.objects[&symbol].object_revision;
        let p = verb(
            &model,
            &root,
            "datum.schematic.place_symbol",
            json!({"sheet":sheet,"library_symbol":symbol,"explicit_part":selected,"reference":"U_NEW","value":"new corrected body","x_nm":40000000,"y_nm":20000000}),
        );
        assert!(p.batch.operations.iter().any(|op| matches!(op,Operation::GuardObjectRevision {object_id,expected_object_revision} if *object_id==symbol && *expected_object_revision==revision)));
        let new = p.primary_object_id.unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        for _ in 0..2 {
            let cold = ProjectResolver::new(&root).resolve().unwrap();
            for id in &old {
                assert_eq!(
                    cold.materialized_placed_symbol_definition(*id, symbol)
                        .unwrap(),
                    Some(original.clone())
                );
            }
            assert_eq!(
                cold.materialized_placed_symbol_definition(new, symbol)
                    .unwrap(),
                Some(corrected.clone())
            );
            let after = placed_snapshot(&cold, &root);
            for key in ["graphics", "texts", "text_geometry"] {
                let mut actual = after[key].as_array().unwrap().clone();
                for expected in before[key].as_array().unwrap() {
                    let i = actual
                        .iter()
                        .position(|v| v == expected)
                        .expect("all old rendered primitives remain exact, including duplicates");
                    actual.remove(i);
                }
            }
            let scene = crate::native_schematic_scene::project_native_schematic_scene(&cold, &root)
                .unwrap()
                .unwrap();
            let body = scene
                .board_graphics
                .iter()
                .find(|g| g.source_object_uuid == new.to_string() && g.width_nm == Some(200000))
                .unwrap();
            assert_eq!(body.path.iter().map(|p| p.x).min(), Some(37000000));
            assert_eq!(body.path.iter().map(|p| p.x).max(), Some(44000000));
            assert_eq!(body.path.iter().map(|p| p.y).min(), Some(18000000));
            assert_eq!(body.path.iter().map(|p| p.y).max(), Some(23000000));
            model
                .commit_journal_undo(&root, provenance().into())
                .unwrap();
            check(&model);
            model
                .commit_journal_redo(&root, provenance().into())
                .unwrap();
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
