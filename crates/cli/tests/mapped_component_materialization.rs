//! Actual CLI binding/materialization choices and source/history boundaries.
#[path = "mapped_component_materialization/fixture.rs"]
mod fixture;
use fixture::*;
use serde_json::json;
use uuid::Uuid;

#[test]
fn manual_map_binding_materialization_history_and_guarded_refusals() {
    let root = fixture();
    let (symbol, package) = seed(&root, 1);
    let ci = Uuid::new_v4();
    success(cli(&bind_args(
        &root,
        "bind-component-instance",
        symbol,
        package,
        ci,
        Some(id(MAP)),
    )));
    refuse(&root, &materialize_args(&root, package, ci));
    adopt(&root);
    let declaration = model(&root).component_instances[&ci]
        .library_bindings
        .clone();
    success(cli(&bind_args(
        &root,
        "set-component-instance",
        symbol,
        package,
        ci,
        None,
    )));
    assert_eq!(
        model(&root).component_instances[&ci].library_bindings,
        declaration
    );
    let args = materialize_args(&root, package, ci);
    let report = success(cli(&args));
    assert_eq!(report["persisted_component_pad_count"], 8);
    assert_sources(&root, package, ci);
    let initial = pads(&root, package);
    let ids = initial
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["uuid"].clone())
        .collect::<Vec<_>>();
    for _ in 0..2 {
        history(&root, "undo");
        let m = model(&root);
        assert!(
            board(&m)
                .get("component_pads")
                .and_then(|v| v.get(package.to_string()))
                .is_none()
        );
        for pad in &ids {
            assert!(
                !m.objects
                    .contains_key(&serde_json::from_value::<Uuid>(pad.clone()).unwrap())
            );
        }
        history(&root, "redo");
        assert_sources(&root, package, ci);
        assert_eq!(
            pads(&root, package)
                .as_array()
                .unwrap()
                .iter()
                .map(|p| p["uuid"].clone())
                .collect::<Vec<_>>(),
            ids
        );
    }
    let (old_symbol, old_package) = seed(&root, 2);
    let old_ci = Uuid::new_v4();
    success(cli(&bind_args(
        &root,
        "bind-component-instance",
        old_symbol,
        old_package,
        old_ci,
        Some(id(MAP)),
    )));
    success(cli(&materialize_args(&root, old_package, old_ci)));
    let old_net = connect(&root, old_symbol, old_package, old_ci);
    let net = connect(&root, symbol, package, ci);
    let other_pads = pads(&root, old_package);
    let other_ci = raw(&model(&root), old_ci);
    let other_identities = model(&root)
        .electrical_identities
        .values()
        .filter(|r| match &r.identity {
            eda_engine::substrate::ElectricalIdentity::NetRelationship { logical_net, .. } => {
                *logical_net == old_net
            }
            _ => false,
        })
        .cloned()
        .collect::<Vec<_>>();
    // Actual connected pads require explicit continuity, not equal geometry or names.
    refuse(&root, &args);
    let before_refresh = pads(&root, package);
    let mut mapped = args.clone();
    for p in before_refresh.as_array().unwrap() {
        mapped.extend([
            "--pad-correspondence".into(),
            format!(
                "{}={}",
                p["uuid"].as_str().unwrap(),
                p["source_binding"]["library_pad"]["object_id"]
                    .as_str()
                    .unwrap()
            ),
        ]);
    }
    success(cli(&mapped));
    assert_sources(&root, package, ci);
    let retained = pads(&root, package)
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["uuid"].clone())
        .collect::<Vec<_>>();
    assert_eq!(retained, ids);
    history(&root, "undo");
    assert_sources(&root, package, ci);
    history(&root, "redo");
    assert_sources(&root, package, ci);
    let m = model(&root);
    assert_eq!(pads(&root, old_package), other_pads);
    assert_eq!(raw(&m, old_ci), other_ci);
    for record in other_identities {
        assert_eq!(m.electrical_identities[&record.id], record);
    }
    for logical in [net, old_net] {
        assert_eq!(
            eda_engine::substrate::net_correspondence_status(&m, logical),
            eda_engine::substrate::NetCorrespondenceStatus::Complete
        );
    }
    for bad in [
        {
            let mut v = args.clone();
            v.pop();
            v.pop();
            v
        },
        {
            let mut v = args.clone();
            v[8] = Uuid::new_v4().to_string();
            v
        },
        {
            let mut v = args.clone();
            v[10] = PART.into();
            v
        },
        {
            let mut v = args.clone();
            v.extend(["--pad-correspondence".into(), "bad=uuid".into()]);
            v
        },
        {
            let mut v = mapped.clone();
            v.extend(["--pad-correspondence".into(), mapped[12].clone()]);
            v
        },
    ] {
        refuse(&root, &bad);
    }
    let before = files(&root);
    let _ = model(&root);
    assert_eq!(files(&root), before);
    for command in [
        "bind-component-instance",
        "set-component-instance",
        "set-board-component-package",
    ] {
        let output = cli(&["project".into(), command.into(), "--help".into()]);
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("--pin-pad-map"));
    }
}

#[test]
fn manual_map_selection_is_explicit_compatible_and_revision_pinned() {
    let root = fixture();
    let (symbol, package) = seed(&root, 1);
    adopt(&root);
    let ci = Uuid::new_v4();
    success(cli(&bind_args(
        &root,
        "bind-component-instance",
        symbol,
        package,
        ci,
        None,
    )));
    assert!(
        model(&root).component_instances[&ci]
            .library_bindings
            .values()
            .all(|b| b.binding_role != eda_engine::substrate::LibraryBindingRole::PinPadMap)
    );
    // A Part default never declares a Map on behalf of a manual caller.
    refuse(&root, &materialize_args(&root, package, ci));
    let mut part = raw(&model(&root), id(PART));
    let other = Uuid::new_v4();
    part["uuid"] = json!(other);
    part["default_pin_pad_map"] = json!(null);
    create(&root, "parts", part);
    let mut foreign = raw(&model(&root), id(MAP));
    foreign["uuid"] = json!(Uuid::new_v4());
    foreign["part"] = json!(other);
    let foreign = create(&root, "pin_pad_maps", foreign);
    let mut absent = raw(&model(&root), id(MAP));
    absent["uuid"] = json!(Uuid::new_v4());
    absent["footprint"] = json!(null);
    absent["mappings"] = json!({});
    let absent = create(&root, "pin_pad_maps", absent);
    for bad in [foreign, absent, id(PART), Uuid::new_v4()] {
        refuse(
            &root,
            &bind_args(
                &root,
                "set-component-instance",
                symbol,
                package,
                ci,
                Some(bad),
            ),
        );
    }
    let mut incomplete = bind_args(
        &root,
        "set-component-instance",
        symbol,
        package,
        ci,
        Some(id(MAP)),
    );
    incomplete.drain(9..11); // Set preserves its existing Part when omitted.
    let previous_part = model(&root).component_instances[&ci]
        .library_bindings
        .clone();
    success(cli(&incomplete));
    for (id, binding) in previous_part {
        assert_eq!(
            model(&root).component_instances[&ci].library_bindings[&id],
            binding
        );
    }
    success(cli(&bind_args(
        &root,
        "set-component-instance",
        symbol,
        package,
        ci,
        Some(id(MAP)),
    )));
    let pinned = model(&root).component_instances[&ci]
        .library_bindings
        .clone();
    let mut m = model(&root);
    let p=(eda_engine::api::native_write::registry::find_native_write_verb("datum.library.edit").unwrap().build)(
        &eda_engine::api::native_write::registry::NativeWriteContext{model:&m,project_root:&root},
        provenance(),json!({"kind":"parts","object_id":id(PART),"pool_path":"pool","fields":{"mpn":"New current MPN"}})).unwrap();
    eda_engine::api::native_write::commit_prepared(&mut m, &root, p).unwrap();
    assert_eq!(m.objects[&id(PART)].object_revision.0, 1);
    let mut cells = raw(&m, id(MAP))["mappings"].as_object().unwrap().clone();
    let first = cells.keys().next().unwrap().clone();
    let second = cells
        .iter()
        .find(|(_, v)| **v != cells[&first])
        .unwrap()
        .0
        .clone();
    let a = cells[&first].clone();
    let b = cells[&second].clone();
    cells.insert(first, b);
    cells.insert(second, a);
    let p = (eda_engine::api::native_write::registry::find_native_write_verb("datum.library.edit")
        .unwrap()
        .build)(
        &eda_engine::api::native_write::registry::NativeWriteContext {
            model: &m,
            project_root: &root,
        },
        provenance(),
        json!({"kind":"pin_pad_maps","object_id":id(MAP),"pool_path":"pool","fields":{
            "mappings":cells}}),
    )
    .unwrap();
    eda_engine::api::native_write::commit_prepared(&mut m, &root, p).unwrap();
    assert_eq!(m.objects[&id(MAP)].object_revision.0, 1);
    refuse(&root, &materialize_args(&root, package, ci));
    let mut omitted_part = bind_args(&root, "set-component-instance", symbol, package, ci, None);
    omitted_part.drain(9..11);
    success(cli(&omitted_part));
    assert_eq!(
        model(&root).component_instances[&ci].library_bindings,
        pinned
    );
    refuse(
        &root,
        &bind_args(
            &root,
            "set-component-instance",
            symbol,
            package,
            ci,
            Some(id(MAP)),
        ),
    );
    assert_eq!(
        model(&root).component_instances[&ci].library_bindings,
        pinned
    );
    // A new component can explicitly declare the new sources; old placed
    // bindings cannot be updated by pretending the absence of pads is fresh.
    let (next_symbol, next_package) = seed(&root, 2);
    let next_ci = Uuid::new_v4();
    success(cli(&bind_args(
        &root,
        "bind-component-instance",
        next_symbol,
        next_package,
        next_ci,
        Some(id(MAP)),
    )));
    let bound = model(&root).component_instances[&next_ci]
        .library_bindings
        .values()
        .find(|b| b.binding_role == eda_engine::substrate::LibraryBindingRole::PinPadMap)
        .unwrap()
        .clone();
    assert_eq!(bound.pinned_object_revision.0, 1);
    success(cli(&materialize_args(&root, next_package, next_ci)));
    assert_sources(&root, next_package, next_ci);
}
