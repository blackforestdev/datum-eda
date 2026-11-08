//! Initial named Part/gate placement through ordinary presented App controls.
use super::*;
use eda_engine::{
    api::native_write::{
        self as native, commit_prepared,
        context::WriteProvenance,
        registry::{NativeWriteContext, find_native_write_verb},
    },
    substrate::{
        CommitSource, DesignModel, ElectricalSelectionSnapshot, ProjectResolver, SourceShardKind,
    },
};
use serde_json::{Value, json};
use uuid::Uuid;
#[derive(Clone)]
struct Placed {
    raw: Value,
    ci: Uuid,
    ci_bytes: Vec<u8>,
    net: Uuid,
    members: std::collections::BTreeSet<eda_engine::substrate::ElectricalOccurrence>,
}
pub(super) struct Proof {
    intent: Value,
    fixed: BTreeMap<String, Vec<u8>>,
    electrical: BTreeMap<Uuid, Value>,
    net_members:
        BTreeMap<Uuid, std::collections::BTreeSet<eda_engine::substrate::ElectricalOccurrence>>,
    sheets: [Value; 2],
    placed: [Option<Placed>; 2],
}
fn prov() -> WriteProvenance {
    WriteProvenance::new(
        "test",
        CommitSource::Manual,
        "Native Part placement fixture",
    )
}
fn write(root: &Path, model: &mut DesignModel, verb: &str, params: Value) -> Uuid {
    let p = (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        prov(),
        params,
    )
    .unwrap();
    let id = p.primary_object_id.unwrap_or(Uuid::nil());
    commit_prepared(model, root, p).unwrap();
    *model = ProjectResolver::new(root).resolve().unwrap();
    id
}
impl Fixture {
    fn placement_choice(&self, which: usize) -> usize {
        let proof = self.placement.as_ref().unwrap();
        let p = proof.intent[if which == 0 { "part2" } else { "part1" }]
            .as_str()
            .unwrap();
        let gate = proof.intent[if which == 0 { "b" } else { "a" }]
            .as_str()
            .unwrap();
        self.app
            .runtime
            .as_ref()
            .unwrap()
            .workspace()
            .schematic_authoring
            .choices
            .iter()
            .position(|c| {
                c.explicit_part
                    .as_ref()
                    .is_some_and(|s| s.part.to_string() == p && s.gate.to_string() == gate)
            })
            .unwrap()
    }
    fn placement_navigate(&mut self, events: &ActiveEventLoop, which: usize, expand: bool) {
        let id: Uuid = serde_json::from_value(
            self.placement.as_ref().unwrap().intent[if which == 0 { "root" } else { "child" }]
                .clone(),
        )
        .unwrap();
        let r = self.app.runtime.as_ref().unwrap();
        if expand && r.presented_hits.regions().iter().any(|h|matches!(&h.target,HitTarget::SchematicDocumentRow{document,..} if document.source_sheet==id)){return;}
        let target = r
            .presented_hits
            .regions()
            .iter()
            .find_map(|h| match &h.target {
                HitTarget::SchematicDocumentRow { document, .. }
                    if !expand && document.source_sheet == id =>
                {
                    Some(h.target.clone())
                }
                HitTarget::SchematicDocumentExpand { document, .. }
                    if expand
                        && document.source_sheet.to_string()
                            == self.placement.as_ref().unwrap().intent["root"]
                                .as_str()
                                .unwrap() =>
                {
                    Some(h.target.clone())
                }
                _ => None,
            })
            .expect("qualified document row/expander must be presented");
        self.control(events, target.clone());
        if !expand {
            self.control(events, target);
        }
    }
    fn placement_check(&mut self, which: usize, present: bool, record: bool) {
        let r = self.app.runtime.as_ref().unwrap();
        let model = r.workspace().library.basis.as_ref().unwrap();
        let source = ElectricalSelectionSnapshot::capture(model).unwrap();
        let schematic = source.schematic_source().unwrap();
        let proof = self.placement.as_mut().unwrap();
        for (path, old) in &proof.fixed {
            assert_eq!(
                std::fs::read(self.root.join(path)).unwrap(),
                *old,
                "unrelated source {path}"
            );
        }
        for (id, old) in &proof.electrical {
            let mut expected = old.clone();
            if (which == 1 || present)
                && old["identity"]["kind"] == "net"
                && old["identity"]["anchor_reason"] == "unclaimed_final_group"
            {
                expected["identity"]["anchor_reason"] = json!("surviving_anchor");
                expected["identity"]["predecessors"] = json!([id]);
                expected["object_revision"] = json!(old["object_revision"].as_u64().unwrap() + 1);
                assert!(model.journal.iter().flat_map(|t|&t.operations).any(|op|matches!(op,
                    eda_engine::substrate::Operation::SetElectricalIdentity {previous,record}
                    if serde_json::to_value(previous).unwrap()==*old && serde_json::to_value(record).unwrap()==expected)),
                    "existing initial lineage transition is recorded canonically");
            }
            assert_eq!(
                serde_json::to_value(&model.electrical_identities[id]).unwrap(),
                expected,
                "PM053 preserves old identity/anchor/active state; only the exact initial lineage transition is allowed"
            );
        }
        for (id, members) in &proof.net_members {
            let anchor =
                serde_json::from_value(proof.electrical[id]["identity"]["anchor"].clone()).unwrap();
            let actual = source.schematic_run(source.revision(), &anchor).unwrap();
            assert_eq!(actual.subject_id, *id);
            assert_eq!(
                actual.owned, *members,
                "old actual schematic net run is unchanged"
            );
        }
        if which == 1 {
            let previous = proof.placed[0].as_ref().unwrap();
            let root: Uuid = serde_json::from_value(proof.intent["root"].clone()).unwrap();
            let id: Uuid = serde_json::from_value(previous.raw["uuid"].clone()).unwrap();
            assert_eq!(
                serde_json::to_value(&schematic.sheets[&root].symbols[&id]).unwrap(),
                previous.raw
            );
            assert_eq!(
                std::fs::read(
                    self.root
                        .join(format!(".datum/component_instances/{}.json", previous.ci))
                )
                .unwrap(),
                previous.ci_bytes
            );
            let origin = eda_engine::substrate::ElectricalOccurrence {
                class: "pins".into(),
                source_id: serde_json::from_value(previous.raw["pins"][0]["uuid"].clone()).unwrap(),
                instance_path: vec![],
            };
            let actual = source.schematic_run(source.revision(), &origin).unwrap();
            assert_eq!(
                (actual.subject_id, actual.owned),
                (previous.net, previous.members.clone()),
                "first component's schematic run survives child edits/history"
            );
        }
        for index in 0..2 {
            let sheet: Uuid = serde_json::from_value(
                proof.intent[if index == 0 { "root" } else { "child" }].clone(),
            )
            .unwrap();
            let mut raw = serde_json::to_value(&schematic.sheets[&sheet]).unwrap();
            raw["symbols"]
                .as_object_mut()
                .unwrap()
                .retain(|_, s| s["reference"] == "OLD1");
            assert_eq!(
                raw, proof.sheets[index],
                "all preexisting sheet fields and symbols stay exact"
            );
        }
        let sheet: Uuid =
            serde_json::from_value(proof.intent[if which == 0 { "root" } else { "child" }].clone())
                .unwrap();
        let found = schematic.sheets[&sheet]
            .symbols
            .values()
            .find(|s| s.reference == if which == 0 { "U1" } else { "U2" });
        if !present {
            assert!(found.is_none());
            if let Some(old) = &proof.placed[which] {
                assert!(!model.component_instances.contains_key(&old.ci));
            }
            return;
        }
        let placed = found.unwrap();
        let part: Uuid = serde_json::from_value(
            proof.intent[if which == 0 { "part2" } else { "part1" }].clone(),
        )
        .unwrap();
        let gate: Uuid =
            serde_json::from_value(proof.intent[if which == 0 { "b" } else { "a" }].clone())
                .unwrap();
        let symbol: Uuid = serde_json::from_value(proof.intent["symbol"].clone()).unwrap();
        let entity: Uuid = serde_json::from_value(proof.intent["entity"].clone()).unwrap();
        let pin: Uuid = serde_json::from_value(proof.intent["pin"].clone()).unwrap();
        assert_eq!(
            (placed.part, placed.entity, placed.gate),
            (Some(part), Some(entity), Some(gate))
        );
        assert_eq!(placed.lib_id, Some(symbol.to_string()));
        assert_eq!(
            placed.position,
            eda_engine::ir::geometry::Point { x: 0, y: 0 }
        );
        assert_eq!(placed.pins.len(), 1);
        assert_eq!(placed.pins[0].library_pin, Some(pin));
        assert_eq!(
            placed.value,
            if which == 0 {
                "Second value"
            } else {
                "First value"
            }
        );
        let ci = native::schematic_symbols::placed_symbol_component_instance_id(
            &model.project.project_id,
            symbol,
            placed.uuid,
        );
        let owner = &model.component_instances[&ci];
        assert_eq!(owner.part_ref, Some(part));
        assert_eq!(owner.library_bindings.len(), 1);
        assert_eq!(owner.placed_symbol_refs, vec![placed.uuid]);
        assert!(owner.placed_package_refs.is_empty());
        let bytes = std::fs::read(
            self.root
                .join(format!(".datum/component_instances/{ci}.json")),
        )
        .unwrap();
        let persisted: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            persisted["component_instance"]["part_ref"]["object_id"],
            json!(part)
        );
        assert_eq!(
            persisted["component_instance"]["part_ref"]["object_revision"],
            proof.intent[if which == 0 {
                "part2_revision"
            } else {
                "part1_revision"
            }]
        );
        let raw = serde_json::to_value(placed).unwrap();
        let path = if which == 0 {
            vec![]
        } else {
            vec![serde_json::from_value(proof.intent["instance"].clone()).unwrap()]
        };
        let origin = eda_engine::substrate::ElectricalOccurrence {
            class: "pins".into(),
            source_id: placed.pins[0].uuid,
            instance_path: path,
        };
        let membership = source.schematic_run(source.revision(), &origin).unwrap();
        assert!(membership.owned.contains(&origin));
        assert!(
            !proof.net_members.contains_key(&membership.subject_id),
            "new unconnected pin never joins an old net"
        );
        if which == 1 {
            assert_ne!(
                membership.subject_id,
                proof.placed[0].as_ref().unwrap().net,
                "separate components/sheets retain distinct unconnected nets"
            );
        }
        if record {
            assert!(proof.placed[which].is_none());
            proof.placed[which] = Some(Placed {
                raw,
                ci,
                ci_bytes: bytes,
                net: membership.subject_id,
                members: membership.owned,
            });
        } else {
            let old = proof.placed[which].as_ref().unwrap();
            assert_eq!(
                (membership.subject_id, membership.owned),
                (old.net, old.members.clone())
            );
            assert_eq!(
                (raw, ci, bytes),
                (old.raw.clone(), old.ci, old.ci_bytes.clone()),
                "pins/pose/CI identity and pinned library binding survive history and cold reopen"
            );
        }
    }
    pub(super) fn placement_stage(&mut self, events: &ActiveEventLoop, stage: usize) {
        if stage == 0 {
            let intent: Value = serde_json::from_slice(
                &std::fs::read(self.output.join("placement-intent.json")).unwrap(),
            )
            .unwrap();
            let r = self.app.runtime.as_ref().unwrap();
            let model = r.workspace().library.basis.as_ref().unwrap();
            let snap = ElectricalSelectionSnapshot::capture(model).unwrap();
            let source = snap.schematic_source().unwrap();
            let sheet = |key: &str| {
                let id: Uuid = serde_json::from_value(intent[key].clone()).unwrap();
                serde_json::to_value(&source.sheets[&id]).unwrap()
            };
            let sheets = [sheet("root"), sheet("child")];
            let fixed = model
                .source_shards
                .iter()
                .filter(|s| {
                    !matches!(
                        s.kind,
                        SourceShardKind::SchematicSheet | SourceShardKind::ElectricalIdentity
                    )
                })
                .map(|s| {
                    (
                        s.relative_path.clone(),
                        std::fs::read(self.root.join(&s.relative_path)).unwrap(),
                    )
                })
                .collect();
            self.placement = Some(Proof {
                net_members: snap
                    .net_ids()
                    .into_iter()
                    .map(|id| {
                        let eda_engine::substrate::ElectricalIdentity::Net { anchor, .. } =
                            &model.electrical_identities[&id].identity
                        else {
                            unreachable!()
                        };
                        let run = snap.schematic_run(snap.revision(), anchor).unwrap();
                        assert_eq!(run.subject_id, id);
                        (id, run.owned)
                    })
                    .collect(),
                electrical: model
                    .electrical_identities
                    .iter()
                    .map(|(id, r)| (*id, serde_json::to_value(r).unwrap()))
                    .collect(),
                intent,
                fixed,
                sheets,
                placed: [None, None],
            });
            return;
        }
        let which = usize::from(stage >= 20);
        let command = if stage >= 20 { stage - 20 } else { stage };
        match command {
            1 => self.placement_navigate(events, which, false),
            2 => {
                assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                    "Place",
                    "Symbol",
                    EntrySurface::MenuPointer
                ));
            }
            3 => self.control(events, HitTarget::Library(C::ProjectSymbols)),
            4 => {
                let index = self.placement_choice(which);
                if index >= 4 {
                    self.control(events, HitTarget::SchematicLibraryPage(true));
                }
            }
            5 => {
                let index = self.placement_choice(which);
                self.control(events, HitTarget::SchematicLibraryChoice(index));
                let e = &self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .schematic_authoring;
                let proof = self.placement.as_ref().unwrap();
                assert_eq!(
                    e.placement_sheet(),
                    Some(
                        serde_json::from_value(
                            proof.intent[if which == 0 { "root" } else { "child" }].clone()
                        )
                        .unwrap()
                    )
                );
                let path: Vec<Uuid> = if which == 0 {
                    vec![]
                } else {
                    vec![serde_json::from_value(proof.intent["instance"].clone()).unwrap()]
                };
                assert_eq!(
                    e.placement_path(),
                    path,
                    "picker retains exact root or unique child occurrence"
                );
            }
            6 => self.control(events, HitTarget::SchematicPlace),
            7 => self.placement_mouse(events, datum_gui_protocol::PointNm { x: 0, y: 0 }, false),
            8 => self.control(events, HitTarget::SchematicCancel),
            9 => self.placement_mouse(events, datum_gui_protocol::PointNm { x: 0, y: 0 }, true),
            10 => self.placement_check(which, true, true),
            11 => self.placement_check(which, false, false),
            12 => self.placement_check(which, true, false),
            13 => self.placement_navigate(events, 1, true),
            14 => assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                "View",
                "Fit to Board",
                EntrySurface::MenuPointer
            )),
            _ => panic!("unknown placement proof stage{stage}"),
        }
    }
}
#[test]
#[ignore = "requires isolated owned Wayland/Vulkan and freshly matched companions"]
fn ordinary_explicit_part_gate_root_child_cancel_history_save_and_cold() {
    let root = PathBuf::from(std::env::var("DATUM_PART_PLACEMENT_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    assert!(!root.exists());
    std::fs::create_dir_all(&output).unwrap();
    let eda_engine::ir::units::PreFeatureProjectUnitsMigration::Create { profile, receipt } =
        eda_engine::ir::units::migrate_pre_feature_project_units(None).unwrap()
    else {
        panic!("fresh Project units profile expected");
    };
    native::genesis::bootstrap_native_project_with_units(
        &root,
        native::genesis::GenesisSpec {
            project_name: "Explicit Part placement".into(),
            existing_ids: None,
        },
        profile,
        receipt,
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let (
        unit,
        pin,
        symbol,
        entity,
        a,
        b,
        package,
        stack,
        foot,
        part1,
        part2,
        map,
        parent,
        child,
        definition,
        instance,
    ) = (
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
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let pad = Uuid::from_u128(1);
    let pad2 = Uuid::from_u128(2);
    for (kind, id, fields) in [
        (
            "units",
            unit,
            json!({"name":"Logic","pins":{pin.to_string():{"name":"Input","electrical_type":"Input"}}}),
        ),
        (
            "symbols",
            symbol,
            json!({"name":"Shared body","unit":unit,"drawings":[{"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}}],"pin_anchors":[{"pin":pin,"position":{"x":-2000000,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]}),
        ),
        ("packages", package, json!({"name":"Body"})),
        (
            "padstacks",
            stack,
            json!({"name":"Copper","aperture":{"circle":{"diameter_nm":1000000}}}),
        ),
        (
            "footprints",
            foot,
            json!({"name":"Land","package":package,"pads":{pad.to_string():{"uuid":pad,"name":"1","position":{"x":0,"y":0},"padstack":stack,"layer":0},pad2.to_string():{"uuid":pad2,"name":"2","position":{"x":2000000,"y":0},"padstack":stack,"layer":0}}}),
        ),
        (
            "entities",
            entity,
            json!({"name":"Dual","manufacturer":"","prefix":"U","gates":{a.to_string():{"uuid":a,"name":"A","unit":unit,"symbol":symbol},b.to_string():{"uuid":b,"name":"B","unit":unit,"symbol":symbol}}}),
        ),
        (
            "parts",
            part1,
            json!({"manufacturer":"Maker","mpn":"FIRST","description":"","datasheet":"","value":"First value","entity":entity,"package":package,"default_footprint":foot}),
        ),
        (
            "parts",
            part2,
            json!({"manufacturer":"Maker","mpn":"SECOND","description":"","datasheet":"","value":"Second value","entity":entity,"package":package,"default_footprint":foot}),
        ),
        (
            "pin_pad_maps",
            map,
            json!({"part":part2,"footprint":foot,"mappings":{pad.to_string():{"gate":a,"pin":pin},pad2.to_string():{"gate":b,"pin":pin}},"assign_default":true}),
        ),
    ] {
        write(
            &root,
            &mut model,
            "datum.library.create",
            json!({"kind":kind,"object_id":id,"pool_path":"pool","fields":fields}),
        );
    }
    write(
        &root,
        &mut model,
        "datum.schematic.create_sheet",
        json!({"sheet":parent,"name":"Root"}),
    );
    let child_request = native::schematic_child_sheet::ChildSheetRequest {
        proposal_id: Uuid::new_v4(),
        parent,
        instance_path: vec![],
        sheet: child,
        definition,
        instance,
        name: "Child".into(),
        position: eda_engine::ir::geometry::Point {
            x: -10000000,
            y: 10000000,
        },
    };
    use native::schematic_delete_proposal::{
        self as proposal, NativeProposalAction as A, NativeProposalRequest,
    };
    for action in [
        A::CreateChildSheet(child_request.clone()),
        A::AcceptApply {
            proposal_id: child_request.proposal_id,
        },
    ] {
        let request = NativeProposalRequest {
            project_root: root.clone(),
            expected_model_revision: model.model_revision.0.clone(),
            expected_transaction_tip: eda_engine::revision::transaction_tip(&model.journal),
            action,
        };
        proposal::execute(&mut model, &request).unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
    }
    write(
        &root,
        &mut model,
        "datum.schematic.place_symbol",
        json!({"sheet":parent,"library_symbol":symbol,"reference":"OLD1","value":"Existing","x_nm":-4000000,"y_nm":0}),
    );
    std::fs::write(output.join("placement-intent.json"),serde_json::to_vec_pretty(&json!({"unit":unit,"pin":pin,"symbol":symbol,"entity":entity,"a":a,"b":b,"part1":part1,"part2":part2,"root":parent,"child":child,"instance":instance,"part1_revision":model.objects[&part1].object_revision.0,"part2_revision":model.objects[&part2].object_revision.0})).unwrap()).unwrap();
    let mut steps = vec![Step::Placement(0)];
    for which in 0usize..2 {
        let offset = which * 20;
        let p = |n| Step::Placement(offset + n);
        if which == 1 {
            steps.push(Step::Placement(13));
        }
        steps.extend([
            p(1),
            p(14),
            Step::Snapshot,
            p(2),
            p(3),
            p(4),
            p(5),
            Step::Capture(if which == 0 {
                "part-root-picker"
            } else {
                "part-child-picker"
            }),
            p(6),
            p(7),
            Step::Capture(if which == 0 {
                "part-root-ghost"
            } else {
                "part-child-ghost"
            }),
            Step::Invariant,
            p(8),
            Step::Invariant,
            p(2),
            p(3),
            p(4),
            p(5),
            p(6),
            p(9),
            p(10),
            Step::Capture(if which == 0 {
                "part-root-placed"
            } else {
                "part-child-placed"
            }),
            Step::History(false),
            p(11),
            Step::History(true),
            p(12),
            Step::Save,
            p(12),
            Step::Reopen,
            p(12),
        ]);
    }
    steps.extend([Step::Placement(12), Step::Capture("part-reopened")]);
    scenario::run(root, output, steps, Uuid::nil());
}
