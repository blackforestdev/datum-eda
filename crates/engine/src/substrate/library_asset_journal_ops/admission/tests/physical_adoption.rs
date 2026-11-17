//! Unsupported physical uptake refuses through public planning and canonical
//! typed commit; real native transfer provides the existing frozen PCB sources.
use super::*;
use crate::ir::geometry::Point;
use native::schematic_board_transfer::{TransferRequest, prepare_transfer};
use std::path::PathBuf;

#[path = "physical_geometry.rs"]
mod physical_geometry;

#[path = "managed_rule_history.rs"]
mod managed_rule_history;

struct Fixture {
    root: PathBuf,
    model: DesignModel,
    part: Uuid,
    footprint: Uuid,
    stack: Uuid,
    map: Uuid,
    limits: LibraryAssetLimits,
}

impl Fixture {
    fn new(with_board: bool) -> Self {
        let root = std::env::temp_dir().join(format!("datum_physical_uptake_{}", Uuid::new_v4()));
        native::genesis::bootstrap_native_project(
            &root,
            native::genesis::GenesisSpec {
                project_name: "Physical uptake".into(),
                existing_ids: None,
            },
        )
        .unwrap();
        let mut model = ProjectResolver::new(&root).resolve().unwrap();
        let (unit, a, b, symbol, entity, gate, package, stack, footprint, p1, p2, part, map, sheet) = (
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        for (kind, id, fields) in [
            (
                "units",
                unit,
                serde_json::json!({"name":"Two pins","pins":{
                a.to_string():{"name":"A","electrical_type":"Input"},
                b.to_string():{"name":"B","electrical_type":"Passive"}}}),
            ),
            (
                "symbols",
                symbol,
                serde_json::json!({"name":"Body","unit":unit,"drawings":[
                {"Rect":{"min":{"x":-1000000,"y":-1000000},
                "max":{"x":1000000,"y":1000000},"width":100000}}],"pin_anchors":[
                {"pin":a,"position":{"x":-2000000,"y":0}},
                {"pin":b,"position":{"x":2000000,"y":0}}]}),
            ),
            (
                "entities",
                entity,
                serde_json::json!({"name":"Device","manufacturer":"Designer","prefix":"U","gates":{
                gate.to_string():{"uuid":gate,"name":"A","unit":unit,"symbol":symbol}}}),
            ),
            ("packages", package, serde_json::json!({"name":"Package"})),
            (
                "padstacks",
                stack,
                serde_json::json!({"name":"Copper","aperture":{"circle":{"diameter_nm":1000000}}}),
            ),
            (
                "footprints",
                footprint,
                serde_json::json!({"name":"Land pattern","package":package,"pads":{
                p1.to_string():{"uuid":p1,"name":"1","position":{"x":0,"y":0},"padstack":stack,"layer":1},
                p2.to_string():{"uuid":p2,"name":"2","position":{"x":2000000,"y":0},"padstack":stack,"layer":1}}}),
            ),
            (
                "parts",
                part,
                serde_json::json!({"manufacturer":"Designer","mpn":"PHYSICAL","value":"Test",
                "description":"","datasheet":"",
                "entity":entity,"package":package,"default_footprint":footprint}),
            ),
            (
                "pin_pad_maps",
                map,
                serde_json::json!({"part":part,"footprint":footprint,"assign_default":true,
                "mappings":{p1.to_string():{"gate":gate,"pin":a},p2.to_string():{"gate":gate,"pin":b}}}),
            ),
        ] {
            write(
                &root,
                &mut model,
                "datum.library.create",
                serde_json::json!({
                "kind":kind,"object_id":id,"pool_path":"pool","fields":fields}),
            );
        }
        write(
            &root,
            &mut model,
            "datum.schematic.create_sheet",
            serde_json::json!({"sheet":sheet,"name":"Root"}),
        );
        let choice = native::schematic_library::part_placement::CompletePartSelection::capture(
            &model, part, gate, symbol,
        )
        .unwrap();
        write(
            &root,
            &mut model,
            "datum.schematic.place_symbol",
            serde_json::json!({
            "sheet":sheet,"library_symbol":symbol,"component":choice,"reference":"U1","value":"Test","x_nm":0,"y_nm":0}),
        );
        if with_board {
            let board = model
                .materialized_source_shard_value(SourceShardKind::BoardRoot)
                .unwrap();
            let board_id = serde_json::from_value(board["uuid"].clone()).unwrap();
            let prepared = native::board_layout::source_profile::build_adopt_board_source_profile(
                &model,
                public_provenance(),
                board_id,
            )
            .unwrap();
            native::commit_prepared(&mut model, &root, prepared).unwrap();
            let prepared = prepare_transfer(
                &model,
                public_provenance(),
                &TransferRequest {
                    sheet,
                    choices: vec![],
                    edit_seed: Uuid::new_v4(),
                    origin: Point::zero(),
                    pitch_nm: 5000000,
                    layer: 1,
                },
            )
            .unwrap();
            native::commit_prepared(&mut model, &root, prepared).unwrap();
            let board = model
                .materialized_source_shard_value(SourceShardKind::BoardRoot)
                .unwrap();
            assert_eq!(board["packages"].as_object().unwrap().len(), 1);
            assert_eq!(
                board["component_pads"]
                    .as_object()
                    .unwrap()
                    .values()
                    .map(|v| v.as_array().unwrap().len())
                    .sum::<usize>(),
                2
            );
            assert!(model.component_instances.values().all(|ci| {
                use crate::substrate::LibraryBindingRole as Role;
                ci.library_bindings
                    .values()
                    .any(|b| b.binding_role == Role::Part)
                    && ci
                        .library_bindings
                        .values()
                        .any(|b| b.binding_role == Role::PinPadMap)
                    && ci
                        .library_bindings
                        .values()
                        .all(|b| matches!(b.binding_role, Role::Part | Role::PinPadMap))
            }));
        }
        fn inventory(path: &Path) -> (usize, usize, usize) {
            if path.is_file() {
                return (std::fs::metadata(path).unwrap().len() as usize, 1, 0);
            }
            std::fs::read_dir(path)
                .unwrap()
                .map(|e| inventory(&e.unwrap().path()))
                .fold((0, 0, 0), |(b, f, d), (nb, nf, nd)| {
                    (b + nb, f + nf, d.max(nd + 1))
                })
        }
        let (bytes, files, depth) = inventory(&root);
        let transitions = [
            "format",
            "capture",
            "adopt",
            "rename",
            "adopt-name",
            "undo",
            "redo",
            "footprint-candidate",
            "stack-candidate",
            "map-candidate",
            "restore-candidate",
            "restore-adopt",
        ];
        let limits = LibraryAssetLimits {
            metadata_bytes: bytes * transitions.len(),
            blob_bytes: bytes * transitions.len(),
            files: files * transitions.len(),
            depth: depth + 1,
            total_bytes: bytes * files * transitions.len(),
        };
        crate::substrate::library_asset_resolver::configure_server_policy(limits).unwrap();
        Self {
            root,
            model,
            part,
            footprint,
            stack,
            map,
            limits,
        }
    }
    fn baseline(&mut self) {
        public_change(
            &self.root,
            &mut self.model,
            serde_json::json!({"action":"adopt_profile"}),
        );
        let catalog = snapshot(&self.root, self.limits)
            .catalog()
            .manifest_ref
            .clone();
        let summary = public_change(
            &self.root,
            &mut self.model,
            serde_json::json!({"action":"save_candidate","request":{
            "expected_catalog":catalog,"change":{"action":"capture_native","asset_id":self.part}}}),
        );
        let root = serde_json::from_value(summary["candidate"].clone()).unwrap();
        adopt(&self.root, &mut self.model, self.limits, &root);
    }
    fn active(&self, id: Uuid) -> AssetRevisionRef {
        snapshot(&self.root, self.limits)
            .lock()
            .assets
            .iter()
            .find(|a| a.asset.asset_id == id)
            .unwrap()
            .asset
            .clone()
    }
    fn revise(&mut self, base: &AssetRevisionRef, payload: Value) -> AssetRevisionRef {
        let catalog = snapshot(&self.root, self.limits)
            .catalog()
            .manifest_ref
            .clone();
        let summary = public_change(
            &self.root,
            &mut self.model,
            serde_json::json!({"action":"save_candidate","request":{
            "expected_catalog":catalog,"change":{"action":"revise","base":base,"payload_json":serde_json::to_string(&payload).unwrap()}}}),
        );
        serde_json::from_value(summary["candidate"].clone()).unwrap()
    }
    fn request(&self, reference: &AssetRevisionRef) -> Value {
        serde_json::json!({"action":"adopt_selection","request":{
            "expected_catalog":snapshot(&self.root,self.limits).catalog().manifest_ref,"selected":reference}})
    }
    fn select(&mut self, reference: &AssetRevisionRef) {
        let request = self.request(reference);
        public_change(&self.root, &mut self.model, request);
    }
}
fn write(root: &Path, model: &mut DesignModel, verb: &str, params: Value) {
    let prepared = (native::registry::find_native_write_verb(verb)
        .unwrap()
        .build)(
        &native::registry::NativeWriteContext {
            model,
            project_root: root,
        },
        public_provenance(),
        params,
    )
    .unwrap();
    native::commit_prepared(model, root, prepared).unwrap();
}
fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(base: &Path, path: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for e in std::fs::read_dir(path).unwrap() {
            let path = e.unwrap().path();
            if path.is_dir() {
                if path != base.join(".datum/stage") {
                    walk(base, &path, out);
                }
            } else {
                out.insert(
                    path.strip_prefix(base).unwrap().to_owned(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

#[test]
fn public_physical_adoption_refuses_sparse_board_users_and_retains_saved_candidates() {
    const NAME: &str = concat!(
        module_path!(),
        "::public_physical_adoption_refuses_sparse_board_users_and_retains_saved_candidates"
    );
    if isolated(NAME) {
        return;
    }
    let mut f = Fixture::new(true);
    f.baseline();
    let active = f.active(f.footprint);
    let mut name = snapshot(&f.root, f.limits).assets()[&active]
        .payload()
        .value()
        .clone();
    name["name"] = "Readable name".into();
    let rename = f.revise(&active, name);
    f.select(&rename);
    f.model
        .commit_journal_undo(&f.root, public_provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.active(f.footprint), active);
    f.model
        .commit_journal_redo(&f.root, public_provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.active(f.footprint), rename);
    for id in [f.footprint, f.stack, f.map] {
        let old_active = f.active(id);
        let current = snapshot(&f.root, f.limits);
        let catalog = crate::library_assets::inspect_closure(
            &current.manifest().roots,
            current.assets(),
            crate::library_assets::ClosureLimits {
                max_nodes: f.limits.files,
                max_depth: f.limits.depth,
            },
        )
        .unwrap();
        let base = catalog
            .assets()
            .keys()
            .find(|r| r.asset_id == id)
            .unwrap()
            .clone();
        let mut next = current.assets()[&base].payload().value().clone();
        if id == f.footprint {
            next["pads"]
                .as_object_mut()
                .unwrap()
                .values_mut()
                .next()
                .unwrap()["position"]["x"] = 100000.into();
        } else if id == f.stack {
            next["aperture"]["circle"]["diameter_nm"] = 1100000.into();
        } else {
            let rows = next["mappings"].as_object_mut().unwrap();
            let keys: Vec<_> = rows.keys().cloned().collect();
            let a = rows[&keys[0]].clone();
            let b = rows[&keys[1]].clone();
            rows.insert(keys[0].clone(), b);
            rows.insert(keys[1].clone(), a);
        }
        let candidate = f.revise(&base, next);
        let before = files(&f.root);
        let revision = f.model.model_revision.clone();
        let result = native::library_assets::prepare_library_change(
            &native::registry::NativeWriteContext {
                model: &f.model,
                project_root: &f.root,
            },
            public_provenance(),
            f.request(&candidate),
        );
        let error = result
            .err()
            .expect("unsupported physical adoption must refuse")
            .to_string();
        assert!(error.contains("unsupported reviewed PCB update"), "{error}");
        assert!(error.contains(&id.to_string()), "{error}");
        assert!(
            error.contains("U1") && error.contains("PCB placement"),
            "{error}"
        );
        let user = f
            .model
            .component_instances
            .values()
            .find(|ci| ci.part_ref == Some(f.part))
            .unwrap();
        assert!(
            error.contains(&user.placed_package_refs[0].to_string()),
            "{error}"
        );
        assert!(error.contains(&user.id.to_string()), "{error}");
        assert_eq!(files(&f.root), before);
        assert_eq!(f.model.model_revision, revision);
        if id == f.footprint {
            // A caller bypassing the native planner still reaches the same
            // refusal through the canonical typed-operation admission owner.
            let raw = raw_adoption(&f);
            let error = native::commit_prepared(&mut f.model, &f.root, raw)
                .unwrap_err()
                .to_string();
            assert!(error.contains("unsupported reviewed PCB update"), "{error}");
            assert_eq!(files(&f.root), before);
            assert_eq!(f.model.model_revision, revision);
        }
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(f.model.model_revision, revision);
        assert_eq!(f.active(id), old_active);
        assert!(
            snapshot(&f.root, f.limits)
                .assets()
                .contains_key(&candidate)
        );
    }
    std::fs::remove_dir_all(f.root).unwrap();
}

fn raw_adoption(f: &Fixture) -> native::PreparedWrite {
    let current = snapshot(&f.root, f.limits);
    let closure = crate::library_assets::inspect_closure(
        &current.manifest().roots,
        current.assets(),
        crate::library_assets::ClosureLimits {
            max_nodes: f.limits.files,
            max_depth: f.limits.depth,
        },
    )
    .unwrap();
    assert!(closure.graph_report().valid);
    let previous = read_state(&f.root, f.limits).unwrap();
    let (mut lock, _) = previous.decode(f.limits.metadata_bytes).unwrap();
    let mut operations = Vec::new();
    for entry in &mut lock.assets {
        let (reference, asset) = closure
            .assets()
            .iter()
            .find(|(r, _)| r.asset_id == entry.asset.asset_id)
            .unwrap();
        let old = f
            .model
            .materialized_source_shard_value_by_relative_path(&entry.source_path)
            .unwrap();
        let policy =
            crate::substrate::library_asset_resolver::counter_policy(asset.revision().kind, 1)
                .unwrap();
        if asset
            .payload()
            .verify_materialization(&old, policy)
            .is_err()
        {
            let mut object = asset.payload().value().clone();
            if old.get("object_revision").is_some() {
                object["object_revision"] =
                    (f.model.objects[&entry.local_object_id].object_revision.0 + 1).into();
            }
            operations.push(Operation::SetPoolLibraryObject {
                object_id: entry.local_object_id,
                relative_path: entry.source_path.clone(),
                object_kind: asset.revision().kind.pool_kind().unwrap().into(),
                previous_object: old,
                object,
            });
        }
        entry.asset = (*reference).clone();
        entry.payload_ref = asset.revision().payload_ref.clone();
    }
    lock.assets.sort_by(|a, b| a.asset.cmp(&b.asset));
    for binding in &mut lock.bindings {
        binding.component = lock
            .assets
            .iter()
            .find(|a| a.asset.asset_id == binding.component.asset_id)
            .unwrap()
            .asset
            .clone();
    }
    lock.closure_digest = closure.digest().clone();
    lock.source_manifests
        .push(current.catalog().manifest_ref.clone());
    lock.source_manifests.sort();
    lock.source_manifests.dedup();
    lock.verify_closure(&closure).unwrap();
    operations.push(Operation::LibraryAssets {
        operation: LibraryAssetOperation::SetState {
            project_id: f.model.project.project_id,
            previous,
            state: crate::substrate::library_asset_operations::LibraryStateValues {
                lock: crate::library_assets::encode_metadata(&lock).unwrap(),
                catalog: read_state(&f.root, f.limits).unwrap().catalog,
            },
            content: vec![],
        },
    });
    native::BatchComposer::compose(&f.model, public_provenance())
        .push_ops(operations)
        .finish()
        .unwrap()
}

#[test]
fn pre_pcb_physical_updates_remain_deliberate_and_reversible() {
    const NAME: &str = concat!(
        module_path!(),
        "::pre_pcb_physical_updates_remain_deliberate_and_reversible"
    );
    if isolated(NAME) {
        return;
    }
    let mut f = Fixture::new(false);
    f.baseline();
    let old = f.active(f.stack);
    let mut next = snapshot(&f.root, f.limits).assets()[&old]
        .payload()
        .value()
        .clone();
    next["aperture"]["circle"]["diameter_nm"] = 1200000.into();
    let candidate = f.revise(&old, next);
    f.select(&candidate);
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.active(f.stack), candidate);
    f.model
        .commit_journal_undo(&f.root, public_provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.active(f.stack), old);
    f.model
        .commit_journal_redo(&f.root, public_provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.active(f.stack), candidate);
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn physical_target_matching_authenticated_frozen_basis_can_restore_existing_stale_design() {
    const NAME: &str = concat!(
        module_path!(),
        "::physical_target_matching_authenticated_frozen_basis_can_restore_existing_stale_design"
    );
    if isolated(NAME) {
        return;
    }
    let mut f = Fixture::new(true);
    let path = format!("pool/footprints/{}.json", f.footprint);
    let original = f
        .model
        .materialized_source_shard_value_by_relative_path(&path)
        .unwrap();
    let mut edited = original.clone();
    edited["pads"]
        .as_object_mut()
        .unwrap()
        .values_mut()
        .next()
        .unwrap()["position"]["x"] = 200000.into();
    // This is a real lawful non-library legacy edit, before managed adoption.
    // It intentionally leaves the already-journaled Board geometry frozen.
    let write = native::library::build_pool_library_write(
        &f.model,
        public_provenance(),
        None,
        vec![native::library::PoolLibraryOperationSpec::Set {
            target: native::library::PoolLibraryObjectTarget::new(
                "pool",
                "footprints",
                f.footprint,
            ),
            object: edited,
        }],
    )
    .unwrap();
    native::commit_prepared(&mut f.model, &f.root, write).unwrap();
    let board = f.root.join("board/board.json");
    let frozen_board = std::fs::read(&board).unwrap();
    f.baseline(); // Existing stale native content is not a newly introduced change.
    let active = f.active(f.footprint);
    let mut restored = snapshot(&f.root, f.limits).assets()[&active]
        .payload()
        .value()
        .clone();
    restored["pads"] = original["pads"].clone();
    let matching = f.revise(&active, restored);
    f.select(&matching);
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.active(f.footprint), matching);
    assert_eq!(std::fs::read(&board).unwrap(), frozen_board);
    assert_eq!(
        f.model
            .materialized_source_shard_value_by_relative_path(&path)
            .unwrap()["pads"],
        original["pads"]
    );
    // This verifies an authenticated matching target, not an automatic Board
    // rebase or a claim that its historical technical pins became current.
    std::fs::remove_dir_all(f.root).unwrap();
}
