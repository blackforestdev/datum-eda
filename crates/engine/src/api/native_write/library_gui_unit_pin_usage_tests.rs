//! Actual qualified current, legacy, frozen and declared reference protection.
use super::*;

fn map(f: &mut Fixture, pin: Option<Uuid>) -> Uuid {
    let id = Uuid::new_v4();
    let rows = pin
        .map(|pin| json!({f.pad.to_string():{"gate":f.gate,"pin":pin}}))
        .unwrap_or_else(|| json!({}));
    let object = json!({"schema_version":1,"uuid":id,"part":f.part,"mappings":rows,"tags":[]});
    canonical(&mut f.model, &f.root, "pin_pad_maps", id, object, true);
    id
}
fn refusal_unchanged(f: &Fixture, pin: Uuid, contains: &str) {
    let before = raw(&f.model, f.unit);
    let bytes = std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap();
    let error = edit(
        &f.model,
        &f.root,
        "units",
        f.unit,
        json!({"pin_removals":[pin]}),
    )
    .unwrap_err();
    assert!(error.to_string().contains(contains), "{error}");
    assert_eq!(raw(&f.model, f.unit), before);
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap(),
        bytes
    );
}
fn binding(id: Uuid, revision: u64, role: &str) -> Value {
    json!({"target_object_id":id,"pinned_object_revision":revision,"binding_role":role,"local_override_refs":[]})
}

#[test]
fn mapping_foreign_unit_with_duplicate_pin_child_does_not_prove_original_pin_unused() {
    let mut f = fixture("unit-pin-foreign-duplicate-child");
    let pin = f.pins[2];
    let foreign: Uuid = "ffffffff-ffff-4fff-8fff-fffffffffff0".parse().unwrap();
    assert!(f.unit.to_string() < foreign.to_string());
    let mut source = raw(&f.model, f.unit);
    source["uuid"] = json!(foreign);
    source["name"] = json!("Foreign duplicate child");
    source["pins"]
        .as_object_mut()
        .unwrap()
        .retain(|key, _| key.parse::<Uuid>().ok() == Some(pin));
    let path = f.root.join(format!("pool/units/{foreign}.json"));
    assert!(!path.exists());
    // Initial ambiguous child ownership input; no canonical Create claim.
    std::fs::write(&path, serde_json::to_vec_pretty(&source).unwrap()).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    for id in [f.unit, foreign] {
        assert!(f.model.objects.contains_key(&id));
    }
    assert_eq!(
        f.model.objects[&pin].source_shard_id,
        f.model.objects[&f.unit].source_shard_id
    );
    assert_ne!(
        f.model.objects[&pin].source_shard_id,
        f.model.objects[&foreign].source_shard_id
    );
    let entity: Uuid = serde_json::from_value(raw(&f.model, f.part)["entity"].clone()).unwrap();
    let mut source = raw(&f.model, entity);
    source["gates"][f.gate.to_string()]["unit"] = json!(foreign);
    canonical(&mut f.model, &f.root, "entities", entity, source, false);
    map(&mut f, Some(pin));
    assert!(
        raw(&f.model, f.symbol)["pin_anchors"]
            .as_array()
            .unwrap()
            .iter()
            .all(|a| a["pin"] != json!(pin))
    );
    refusal_unchanged(
        &f,
        pin,
        "Pin mapping logical pin membership is unavailable or ambiguous",
    );
}

#[test]
fn duplicate_roots_cannot_hide_exact_pin_references_and_unrelated_duplicates_do_not_block() {
    for kind in ["symbols", "parts", "pin_pad_maps"] {
        for relevant in [true, false] {
            let mut f = fixture(&format!("unit-pin-duplicate-{kind}-{relevant}"));
            let pin = f.pins[2];
            let id = if relevant {
                match kind {
                    "symbols" => f.symbol,
                    "parts" => f.part,
                    _ => map(&mut f, None),
                }
            } else {
                let id = Uuid::new_v4();
                let mut value = match kind {
                    "symbols" => {
                        let mut v = raw(&f.model, f.symbol);
                        v["unit"] = raw(&f.model, f.other_entity)["gates"]
                            .as_object()
                            .unwrap()
                            .values()
                            .next()
                            .unwrap()["unit"]
                            .clone();
                        v["pin_anchors"] = json!([]);
                        v
                    }
                    "parts" => {
                        let mut v = raw(&f.model, f.part);
                        v["entity"] = json!(f.other_entity);
                        v
                    }
                    _ => {
                        json!({"schema_version":1,"uuid":id,"part":f.part,"mappings":{},"tags":[]})
                    }
                };
                value["uuid"] = json!(id);
                canonical(&mut f.model, &f.root, kind, id, value, true);
                id
            };
            let mut duplicate = raw(&f.model, id);
            if relevant {
                match kind {
                    "symbols" => duplicate["pin_anchors"][0]["pin"] = json!(pin),
                    "parts" => {
                        duplicate["pad_map"] = json!({f.pad.to_string():{"gate":f.gate,"pin":pin}})
                    }
                    _ => {
                        duplicate["mappings"] = json!({f.pad.to_string():{"gate":f.gate,"pin":pin}})
                    }
                }
            }
            // New initial duplicate input before the negative baseline; never
            // rewrite the existing canonical source or claim canonical admission.
            let duplicate_path = format!("pool/{kind}/z-duplicate-{id}.json");
            let path = f.root.join(&duplicate_path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            assert!(!path.exists());
            std::fs::write(&path, serde_json::to_vec_pretty(&duplicate).unwrap()).unwrap();
            f.model = ProjectResolver::new(&f.root).resolve().unwrap();
            assert!(
                f.model
                    .source_shards
                    .iter()
                    .any(|s| s.relative_path == duplicate_path)
            );
            assert_eq!(
                crate::api::native_write::library_gui::part::graph(&f.model)
                    .unwrap()
                    .1[&id],
                2
            );
            let owner = f
                .model
                .source_shards
                .iter()
                .find(|s| s.shard_id == f.model.objects[&id].source_shard_id)
                .unwrap();
            assert_eq!(owner.relative_path, format!("pool/{kind}/{id}.json"));
            if relevant {
                refusal_unchanged(&f, pin, "ambiguous duplicate");
            } else {
                let write = edit(
                    &f.model,
                    &f.root,
                    "units",
                    f.unit,
                    json!({"pin_removals":[pin]}),
                )
                .unwrap();
                commit_prepared(&mut f.model, &f.root, write).unwrap();
                assert!(!f.model.objects.contains_key(&pin));
            }
        }
    }
}

#[test]
fn exact_symbol_anchor_still_protects_pin_after_unit_relink() {
    let mut f = fixture("unit-pin-symbol-relinked");
    let other = raw(&f.model, f.other_entity)["gates"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap()["unit"]
        .clone();
    let mut source = raw(&f.model, f.symbol);
    source["unit"] = other;
    canonical(&mut f.model, &f.root, "symbols", f.symbol, source, false);
    refusal_unchanged(&f, f.pins[1], "Symbol anchor");
}

#[test]
fn ambiguous_gate_alias_and_unavailable_entity_do_not_prove_a_mapped_pin_unused() {
    for aliases in [true, false] {
        let mut f = fixture(&format!("unit-pin-qualify-{aliases}"));
        let pin = f.pins[2];
        map(&mut f, Some(pin));
        let entity: Uuid = serde_json::from_value(raw(&f.model, f.part)["entity"].clone()).unwrap();
        if aliases {
            let mut payload = raw(&f.model, entity);
            let gate = payload["gates"][f.gate.to_string()].clone();
            let other_unit = raw(&f.model, f.other_entity)["gates"]
                .as_object()
                .unwrap()
                .values()
                .next()
                .unwrap()["unit"]
                .clone();
            payload["gates"][f.gate.to_string()]["unit"] = other_unit;
            payload["gates"][f.gate.simple().to_string()] = gate;
            canonical(&mut f.model, &f.root, "entities", entity, payload, false);
            refusal_unchanged(&f, pin, "ambiguous");
        } else {
            // Missing registered Entity authority is not a proven different Unit.
            f.model.objects.remove(&entity);
            refusal_unchanged(&f, pin, "mapping Entity qualification");
        }
    }
}

#[test]
fn frozen_actual_placed_pin_remains_used_after_current_anchor_is_canonically_removed() {
    let mut f = fixture("unit-pin-old-placed");
    let pin = f.pins[1];
    let snapshot = crate::substrate::ElectricalSelectionSnapshot::capture(&f.model).unwrap();
    let sheet = *snapshot
        .schematic_source()
        .unwrap()
        .sheets
        .keys()
        .next()
        .unwrap();
    let intent =
        crate::api::native_write::schematic_library::part_placement::PartGateSelection::capture(
            &f.model, f.part, f.gate, f.symbol,
        )
        .unwrap();
    let prepared = (find_native_write_verb("datum.schematic.place_symbol")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &f.model,
            project_root: &f.root,
        },
        provenance(),
        json!({"sheet":sheet,"library_symbol":f.symbol,"explicit_part":intent,
            "reference":"U_UNUSED_TEST","value":"Source","x_nm":40000000,"y_nm":10000000}),
    )
    .unwrap();
    let placed = prepared.primary_object_id.unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let prepared = edit(
        &f.model,
        &f.root,
        "symbols",
        f.symbol,
        json!({"anchor_removals":[pin]}),
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    assert!(
        raw(&f.model, f.symbol)["pin_anchors"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        raw(&f.model, f.part)["pad_map"]
            .as_object()
            .unwrap()
            .is_empty()
    );
    let frozen = f
        .model
        .materialized_placed_symbol_definition(placed, f.symbol)
        .unwrap()
        .unwrap();
    assert_eq!(frozen["pin_anchors"][0]["pin"], json!(pin));
    refusal_unchanged(&f, pin, "actual placed pin");
}

#[test]
fn each_retained_part_revision_keeps_its_own_legacy_mapping_context() {
    let mut f = fixture("unit-pin-old-part-context");
    let pin = f.pins[2];
    let mut part = raw(&f.model, f.part);
    part["pad_map"] = json!({f.pad.to_string():{"gate":f.gate,"pin":pin}});
    canonical(&mut f.model, &f.root, "parts", f.part, part, false);
    let old = f.model.objects[&f.part].object_revision.0;
    let ci = Uuid::new_v4();
    let role = Uuid::new_v4();
    let payload = json!({"uuid":ci,"object_revision":0,"part_ref":{"object_id":f.part,"object_revision":old},
        "library_bindings":{role.to_string():binding(f.part,old,"part")},
        "placed_symbol_refs":[{"object_id":f.original_placed,"object_revision":f.model.objects[&f.original_placed].object_revision.0}],"placed_package_refs":[]});
    let p = crate::api::native_write::BatchComposer::compose(&f.model, provenance())
        .push_op(crate::substrate::Operation::CreateComponentInstance {
            component_instance_id: ci,
            component_instance: payload,
        })
        .finish()
        .unwrap();
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    let mut next = raw(&f.model, f.part);
    next["pad_map"] = json!({});
    next["entity"] = json!(f.other_entity);
    canonical(&mut f.model, &f.root, "parts", f.part, next, false);
    let previous = raw(&f.model, ci)["component_instance"].clone();
    let mut next = previous.clone();
    next["object_revision"] = json!(f.model.objects[&ci].object_revision.0 + 1);
    next["library_bindings"][role.to_string()]["local_override_refs"] = json!([
        {"object_id":f.part,"object_revision":f.model.objects[&f.part].object_revision.0}]);
    let p = crate::api::native_write::BatchComposer::compose(&f.model, provenance())
        .push_op(crate::substrate::Operation::SetComponentInstance {
            component_instance_id: ci,
            previous_component_instance: previous,
            component_instance: next,
        })
        .finish()
        .unwrap();
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    assert!(
        raw(&f.model, f.part)["pad_map"]
            .as_object()
            .unwrap()
            .is_empty()
    );
    refusal_unchanged(&f, pin, "mapping");
}

#[test]
fn unresolved_registered_ci_still_protects_its_retained_map_and_legacy_part_scope() {
    for missing_map in [false, true] {
        let mut f = fixture(&format!("unit-pin-declared-{missing_map}"));
        let pin = f.pins[2];
        let id = map(&mut f, (!missing_map).then_some(pin));
        let old_part = f.model.objects[&f.part].object_revision.0;
        if missing_map {
            let mut part = raw(&f.model, f.part);
            part["entity"] = json!(f.other_entity);
            canonical(&mut f.model, &f.root, "parts", f.part, part, false);
        } else {
            let mut current = raw(&f.model, id);
            current["mappings"] = json!({});
            canonical(&mut f.model, &f.root, "pin_pad_maps", id, current, false);
        }
        let ci = Uuid::new_v4();
        let map_role = Uuid::new_v4();
        let bad_role = Uuid::new_v4();
        let payload = json!({"schema_version":1,"component_instance":{"uuid":ci,"object_revision":0,
            "part_ref":{"object_id":f.part,"object_revision":old_part},
            "library_bindings":{map_role.to_string():binding(id,if missing_map{99}else{0},"pin_pad_map"),
                bad_role.to_string():binding(Uuid::new_v4(),0,"package")},
            "placed_symbol_refs":[],"placed_package_refs":[]}});
        // Initial unavailable-declaration fixture, not a canonical CI creation.
        let path = f.root.join(format!(".datum/component_instances/{ci}.json"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        assert!(!path.exists());
        std::fs::write(path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert!(!f.model.component_instances.contains_key(&ci));
        assert_eq!(
            raw(&f.model, ci)["component_instance"]["part_ref"]["object_revision"],
            json!(old_part)
        );
        refusal_unchanged(
            &f,
            pin,
            if missing_map {
                "retained mapping source is unavailable"
            } else {
                "mapping"
            },
        );
    }
}

#[test]
fn default_nondefault_and_legacy_mapping_usage_refuses_without_anchor_masking() {
    for mode in 0..4 {
        let mut f = fixture(&format!("unit-remove-map-{mode}"));
        let pin = f.pins[2];
        let map = Uuid::new_v4();
        if mode == 3 {
            let mut part = raw(&f.model, f.part);
            part["pad_map"] = json!({f.pad.to_string():{"gate":f.gate,"pin":pin}});
            canonical(&mut f.model, &f.root, "parts", f.part, part, false);
        } else {
            let entries = if mode == 2 {
                json!({pin.to_string():{"pad":f.pad}})
            } else {
                json!({f.pad.to_string():{"gate":f.gate,"pin":pin}})
            };
            let object = json!({"schema_version":1,"uuid":map,"part":f.part,"footprint":null,"mappings":entries,"tags":[]});
            if mode == 2 {
                // Initial read-compatibility input; canonical creation intentionally
                // emits modern rows and does not create this obsolete shape.
                let path = f.root.join(format!("pool/pin_pad_maps/{map}.json"));
                assert!(!path.exists());
                std::fs::write(&path, serde_json::to_vec_pretty(&object).unwrap()).unwrap();
                f.model = ProjectResolver::new(&f.root).resolve().unwrap();
                assert_eq!(raw(&f.model, map)["mappings"], object["mappings"]);
            } else {
                canonical(&mut f.model, &f.root, "pin_pad_maps", map, object, true);
            }
            if mode == 0 {
                let mut part = raw(&f.model, f.part);
                part["default_pin_pad_map"] = json!(map);
                canonical(&mut f.model, &f.root, "parts", f.part, part, false);
            }
        }
        let original = raw(&f.model, f.unit);
        let bytes = std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap();
        let error = edit(
            &f.model,
            &f.root,
            "units",
            f.unit,
            json!({"pin_removals":[pin]}),
        )
        .unwrap_err();
        assert!(error.to_string().contains("mapping"), "mode{mode}: {error}");
        assert_eq!(raw(&f.model, f.unit), original);
        assert_eq!(
            std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap(),
            bytes
        );
    }
}
