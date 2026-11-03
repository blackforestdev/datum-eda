use super::*;
use crate::api::native_write::board_components::{
    BoardPackagePlacement, build_place_board_package,
};
use crate::{board::PlacedPackage, ir::geometry::Point};
#[test]
fn native_bound_delete_rejects_physical_multisymbol_and_unresolved_source() {
    for mode in 0..3 {
        let (root, mut model, sheet, symbol, ci, part) = setup("bound-delete-unsupported");
        let write = if mode == 2 {
            let mut payload = raw(&model, symbol)["symbols"][symbol.to_string()].clone();
            payload["extension"] = json!("external source change");
            BatchComposer::compose(&model, provenance())
                .push_op(Operation::SetSchematicSymbol {
                    sheet_id: sheet,
                    symbol_id: symbol,
                    symbol: payload,
                })
                .finish()
                .unwrap()
        } else {
            let other = if mode == 0 {
                let placement = BoardPackagePlacement {
                    package: PlacedPackage {
                        uuid: Uuid::new_v4(),
                        part,
                        package: "50000000-0000-4000-8000-000000000001".parse().unwrap(),
                        reference: "U1".into(),
                        value: "".into(),
                        position: Point::zero(),
                        rotation: 0,
                        layer: 1,
                        locked: false,
                    },
                    materialized: json!({}),
                };
                let write = build_place_board_package(&model, provenance(), &placement).unwrap();
                commit_prepared(&mut model, &root, write).unwrap();
                placement.package.uuid
            } else {
                verb(
                    &mut model,
                    &root,
                    "datum.schematic.place_symbol",
                    json!({"sheet":sheet,"library_symbol":"30000000-0000-4000-8000-000000000001","reference":"U2","value":"OPAMP","x_nm":50_000_000,"y_nm":20_000_000}),
                )
            };
            let previous = raw(&model, ci)["component_instance"].clone();
            let mut next = previous.clone();
            next["object_revision"] = json!(model.objects[&ci].object_revision.0 + 1);
            next[if mode==0 {"placed_package_refs"} else {"placed_symbol_refs"}].as_array_mut().unwrap().push(json!({"object_id":other,"object_revision":model.objects[&other].object_revision.0}));
            BatchComposer::compose(&model, provenance())
                .push_op(Operation::SetComponentInstance {
                    component_instance_id: ci,
                    previous_component_instance: previous,
                    component_instance: next,
                })
                .finish()
                .unwrap()
        };
        commit_prepared(&mut model, &root, write).unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        let before = raw(&model, symbol);
        let len = model.journal.len();
        assert!(bound_symbol_owner(&model, symbol, &[]).is_err());
        let req = request(
            &model,
            &root,
            create(
                Uuid::new_v4(),
                sheet,
                vec![target(DeleteClass::Symbol, symbol)],
            ),
        );
        assert!(execute(&mut model, &req).is_err());
        assert_eq!(model.journal.len(), len);
        assert_eq!(raw(&model, symbol), before);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn native_bound_delete_refuses_surviving_owned_field_relationship_and_variant() {
    for variant in [false, true] {
        let (root, mut model, sheet, symbol, _, _) = setup("bound-delete-field-dependency");
        let mut payload = raw(&model, symbol)["symbols"][symbol.to_string()].clone();
        let field: Uuid = if variant {
            Uuid::new_v4()
        } else {
            serde_json::from_value(payload["fields"][0]["uuid"].clone()).unwrap()
        };
        if variant {
            payload["extension_object"] = json!({"uuid":field,"data":"retain"});
            let original = source(&model).sheets[&sheet].symbols[&symbol].clone();
            let write = BatchComposer::compose(&model, provenance())
                .push_op(Operation::SetSchematicSymbol {
                    sheet_id: sheet,
                    symbol_id: symbol,
                    symbol: payload,
                })
                .push_ops(
                    crate::api::native_write::schematic_assignment::repin_symbols(
                        &model,
                        &[original],
                    )
                    .unwrap(),
                )
                .finish()
                .unwrap();
            commit_prepared(&mut model, &root, write).unwrap();
        }
        let id = Uuid::new_v4();
        let op = if variant {
            Operation::CreateVariantOverlay {
                variant_id: id,
                variant: json!({"id":id,"name":"field variant","base_model_revision":model.model_revision,"variant_revision":0,"fitted":{},"relationship_overrides":{},"property_overrides":{field.to_string():{"value":"retained"}}}),
            }
        } else {
            Operation::CreateRelationship {
                relationship_id: id,
                relationship: serde_json::to_value(Relationship {
                    id,
                    kind: RelationshipKind::SchematicOnly,
                    from: vec![RevisionedRef {
                        object_id: field,
                        object_revision: model.objects[&field].object_revision,
                    }],
                    to: vec![],
                    authored_intent: vec![],
                    object_revision: ObjectRevision(0),
                })
                .unwrap(),
            }
        };
        let write = BatchComposer::compose(&model, provenance())
            .push_op(op)
            .finish()
            .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        let before = raw(&model, symbol);
        let len = model.journal.len();
        assert!(
            bound_symbol_owner(&model, symbol, &[])
                .unwrap_err()
                .to_string()
                .contains("dependencies")
        );
        let req = request(
            &model,
            &root,
            create(
                Uuid::new_v4(),
                sheet,
                vec![target(DeleteClass::Symbol, symbol)],
            ),
        );
        assert!(
            execute(&mut model, &req)
                .unwrap_err()
                .to_string()
                .contains("dependencies")
        );
        assert_eq!(model.journal.len(), len);
        assert_eq!(raw(&model, symbol), before);
        std::fs::remove_dir_all(root).unwrap();
    }
}
