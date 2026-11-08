//! Named map rows and explicit protected Part default through the ordinary App.
use super::*;
use eda_engine::api::native_write::{
    self as native, WriteProvenance, registry::NativeWriteContext,
};
use eda_engine::substrate::{CommitSource, ProjectResolver, SourceShardKind};
use serde_json::{Value, json};
use uuid::Uuid;
fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "test",
        CommitSource::Manual,
        "Prepare native mapping project",
    )
}
impl Fixture {
    pub(super) fn mapping_result_index(&self) -> usize {
        let id = self.edited_footprint.as_ref().unwrap()["uuid"]
            .as_str()
            .unwrap();
        self.app
            .runtime
            .as_ref()
            .unwrap()
            .workspace()
            .library
            .entries
            .iter()
            .position(|e| e.id.to_string() == id)
            .unwrap()
    }
    pub(super) fn check_mapping(&mut self, stage: usize, assigned: bool) {
        let model = ProjectResolver::new(&self.root).resolve().unwrap();
        let browser = datum_gui_protocol::library_browser::LibraryBrowser::capture(
            std::sync::Arc::new(model.clone()),
        );
        let eligible = browser
            .entries
            .iter()
            .find(|e| e.kind == "parts" && e.name == "ELIGIBLE")
            .unwrap();
        let watched: std::collections::BTreeSet<_> = model
            .source_shards
            .iter()
            .filter(|s| {
                matches!(
                    s.kind,
                    SourceShardKind::Pool
                        | SourceShardKind::BoardRoot
                        | SourceShardKind::SchematicRoot
                        | SourceShardKind::SchematicSheet
                        | SourceShardKind::SchematicDefinition
                        | SourceShardKind::ComponentInstance
                        | SourceShardKind::ElectricalIdentity
                )
            })
            .map(|s| s.relative_path.clone())
            .collect();
        let bytes: BTreeMap<_, _> = source_bytes(&self.root)
            .into_iter()
            .filter(|(p, _)| watched.contains(p))
            .collect();
        let now = json!(bytes);
        if stage == 0 || stage == 4 {
            self.original_footprint = Some(
                json!({"files":now,"eligible":eligible.raw,"eligible_path":eligible.source_path}),
            );
            return;
        }
        let before = self.original_footprint.as_ref().unwrap();
        for (path, expected) in before["files"].as_object().unwrap() {
            if assigned && Some(path.as_str()) == before["eligible_path"].as_str() {
                let mut raw = before["eligible"].clone();
                if stage == 2 {
                    raw["default_pin_pad_map"] =
                        self.edited_footprint.as_ref().unwrap()["uuid"].clone();
                }
                assert_eq!(
                    eligible.raw, raw,
                    "only explicitly assigned Part default changes; unknown/absent fields stay exact"
                );
            } else {
                assert_eq!(
                    &now[path], expected,
                    "existing source {path} stays byte-exact"
                );
            }
        }
        let state = &self.app.runtime.as_ref().unwrap().workspace().library;
        if stage == 5 {
            assert_eq!(state.draft.as_ref().unwrap().value("map_pin"), "");
            return;
        }
        if stage == 1 {
            let d = state.draft.as_ref().unwrap();
            let raw = d.preview.as_ref().unwrap();
            let part = browser
                .entries
                .iter()
                .find(|e| {
                    e.kind == "parts" && e.name == if assigned { "ELIGIBLE" } else { "PLACED" }
                })
                .unwrap();
            let foot = browser
                .entries
                .iter()
                .find(|e| e.kind == "footprints" && e.name == "Land")
                .unwrap();
            let entity = browser
                .entries
                .iter()
                .find(|e| e.kind == "entities")
                .unwrap();
            let unit = browser
                .entries
                .iter()
                .find(|e| e.kind == "units" && e.name == "Logic")
                .unwrap();
            let named = |rows: &Value, name: &str| -> Uuid {
                rows.as_object()
                    .unwrap()
                    .iter()
                    .find(|(_, row)| row["name"].as_str() == Some(name))
                    .unwrap()
                    .0
                    .parse()
                    .unwrap()
            };
            let pin = named(&unit.raw["pins"], "Input");
            let pad = named(&foot.raw["pads"], "1");
            let gate = named(&entity.raw["gates"], "A");
            let mut expected = json!({pad.to_string():{"gate":gate,"pin":pin}});
            if assigned {
                let pad2 = named(&foot.raw["pads"], "2");
                let gate2 = named(&entity.raw["gates"], "B");
                expected[pad2.to_string()] = json!({"gate":gate2,"pin":pin});
            }
            assert_eq!(raw["uuid"], json!(d.id));
            assert_eq!(raw["part"], json!(part.id));
            assert_eq!(raw["footprint"], json!(foot.id));
            assert_eq!(raw["mappings"], expected);
            assert_eq!(
                d.request().unwrap().fields["assign_default"],
                json!(assigned)
            );
            assert!(!raw.as_object().unwrap().contains_key("name"));
            self.edited_footprint = Some(raw.clone());
            return;
        }
        let expected = self.edited_footprint.as_ref().unwrap();
        let found = browser
            .entries
            .iter()
            .find(|e| e.id.to_string() == expected["uuid"].as_str().unwrap());
        if stage == 3 {
            assert!(found.is_none());
        } else {
            assert_eq!(&found.unwrap().raw, expected);
        }
    }
}
#[test]
#[ignore = "requires fresh owned project/output and isolated native Wayland with matching companions"]
fn ordinary_map_named_rows_map_only_and_explicit_default_history_save_and_cold() {
    let root = PathBuf::from(std::env::var("DATUM_PIN_PAD_MAP_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    assert!(!root.exists());
    std::fs::create_dir_all(&output).unwrap();
    native::genesis::bootstrap_native_project(
        &root,
        native::genesis::GenesisSpec {
            project_name: "Pin-pad mapping".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let (
        unit,
        pin,
        symbol,
        power,
        power_pin,
        power_symbol,
        entity,
        a,
        b,
        c,
        package,
        stack,
        foot,
        eligible,
        placed_part,
        sheet,
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
    let mut old_symbol = Uuid::nil();
    for (verb, params) in [
        (
            "datum.library.create",
            json!({"kind":"units","object_id":unit,"pool_path":"pool","fields":{"name":"Logic","pins":{pin.to_string():{"name":"Input","electrical_type":"Input"}}}}),
        ),
        (
            "datum.library.create",
            json!({"kind":"symbols","object_id":symbol,"pool_path":"pool","fields":{"name":"Body","unit":unit,"drawings":[{"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}}],"pin_anchors":[{"pin":pin,"position":{"x":-2000000,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]}}),
        ),
        (
            "datum.schematic.create_sheet",
            json!({"sheet":sheet,"name":"Root"}),
        ),
        (
            "datum.schematic.place_symbol",
            json!({"sheet":sheet,"library_symbol":symbol,"reference":"U1","value":"Original","x_nm":0,"y_nm":0}),
        ),
        (
            "datum.library.create",
            json!({"kind":"units","object_id":power,"pool_path":"pool","fields":{"name":"Power","pins":{power_pin.to_string():{"name":"VCC","electrical_type":"PowerIn"}}}}),
        ),
        (
            "datum.library.create",
            json!({"kind":"symbols","object_id":power_symbol,"pool_path":"pool","fields":{"name":"Supply body","unit":power,"drawings":[{"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}}],"pin_anchors":[{"pin":power_pin,"position":{"x":-2000000,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]}}),
        ),
        (
            "datum.library.create",
            json!({"kind":"entities","object_id":entity,"pool_path":"pool","fields":{"name":"Multi-gate","prefix":"U","manufacturer":"","gates":{a.to_string():{"uuid":a,"name":"A","unit":unit,"symbol":symbol},b.to_string():{"uuid":b,"name":"B","unit":unit,"symbol":symbol},c.to_string():{"uuid":c,"name":"PWR","unit":power,"symbol":power_symbol}}}}),
        ),
        (
            "datum.library.create",
            json!({"kind":"packages","object_id":package,"pool_path":"pool","fields":{"name":"Package"}}),
        ),
        (
            "datum.library.create",
            json!({"kind":"padstacks","object_id":stack,"pool_path":"pool","fields":{"name":"Copper","aperture":{"circle":{"diameter_nm":1000000}}}}),
        ),
        (
            "datum.library.create",
            json!({"kind":"footprints","object_id":foot,"pool_path":"pool","fields":{"name":"Land","package":package,"pads":{pad.to_string():{"uuid":pad,"name":"1","position":{"x":0,"y":0},"padstack":stack,"layer":0},pad2.to_string():{"uuid":pad2,"name":"2","position":{"x":2000000,"y":0},"padstack":stack,"layer":0}}}}),
        ),
    ] {
        let p = (native::registry::find_native_write_verb(verb)
            .unwrap()
            .build)(
            &NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            provenance(),
            params,
        )
        .unwrap();
        if verb == "datum.schematic.place_symbol" {
            old_symbol = p.primary_object_id.unwrap();
        }
        native::commit_prepared(&mut model, &root, p).unwrap();
    }
    for (part, mpn) in [(eligible, "ELIGIBLE"), (placed_part, "PLACED")] {
        let mut raw = native::library::pool_part_payload(
            part,
            entity,
            package,
            mpn,
            "Designer",
            "Original",
            "Description",
            "reference.pdf",
            "Unknown",
        );
        raw["default_footprint"] = json!(foot);
        raw["x_vendor"] = json!({"opaque":["keep",17]});
        raw.as_object_mut().unwrap().remove("packaging_options");
        let p = native::library::build_pool_library_write(
            &model,
            provenance(),
            Some("pool"),
            vec![native::library::PoolLibraryOperationSpec::Create {
                target: native::library::PoolLibraryObjectTarget::new("pool", "parts", part),
                object: raw,
            }],
        )
        .unwrap();
        native::commit_prepared(&mut model, &root, p).unwrap();
    }
    let ci = native::component_instances::build_placed_symbol_component_instance_op(
        &model,
        old_symbol,
        symbol,
        placed_part,
    )
    .unwrap();
    let batch = native::build_batch(&model, provenance(), vec![ci]).unwrap();
    model.commit_journaled(&root, batch).unwrap();
    let mut steps = vec![
        Step::Mapping(0, false),
        Step::Open,
        Step::Kind(7),
        Step::Control(C::New),
        Step::Snapshot,
        Step::KeyControl(C::Preview),
        Step::Refusal,
        Step::Capture("map-empty-refusal"),
        Step::Invariant,
        Step::Choice("map_part", "ELIGIBLE".into()),
        Step::Choice("map_footprint", "Land".into()),
        Step::KeyControl(C::MappingPad(0)),
        Step::Choice("map_gate", "A".into()),
        Step::Choice("map_pin", "Input".into()),
        Step::KeyControl(C::UnmapPad),
        Step::KeyControl(C::Preview),
        Step::Refusal,
        Step::Invariant,
        Step::Choice("map_gate", "A".into()),
        Step::Choice("map_pin", "Input".into()),
        Step::KeyControl(C::Preview),
        Step::Capture("map-cancel-preview"),
        Step::Invariant,
        Step::KeyControl(C::Cancel),
        Step::Invariant,
        Step::KeyControl(C::New),
        Step::Snapshot,
        Step::Choice("map_part", "PLACED".into()),
        Step::Choice("map_footprint", "Land".into()),
        Step::KeyControl(C::MappingPad(0)),
        Step::Choice("map_gate", "A".into()),
        Step::Choice("map_pin", "Input".into()),
        Step::KeyControl(C::Preview),
        Step::Mapping(1, false),
        Step::Capture("map-only-preview"),
        Step::Focus(C::MappingPad(0)),
        Step::Capture("map-partial-table"),
        Step::Choice("map_default", "Explicitly set".into()),
        Step::KeyControl(C::Preview),
        Step::Refusal,
        Step::Capture("map-placed-default-refusal"),
        Step::Invariant,
        Step::Choice("map_default", "Create map only".into()),
        Step::KeyControl(C::Preview),
        Step::Mapping(1, false),
        Step::Invariant,
        Step::KeyControl(C::Commit),
        Step::Mapping(2, false),
        Step::History(false),
        Step::Mapping(3, false),
        Step::History(true),
        Step::Mapping(2, false),
        Step::Save,
        Step::Mapping(2, false),
        Step::Reopen,
        Step::Mapping(2, false),
        Step::Mapping(4, false),
        Step::Open,
        Step::Kind(7),
        Step::KeyControl(C::New),
        Step::Snapshot,
        Step::Choice("map_part", "ELIGIBLE".into()),
        Step::Choice("map_footprint", "Land".into()),
        Step::KeyControl(C::MappingPad(0)),
        Step::Choice("map_gate", "A".into()),
        Step::Choice("map_pin", "Input".into()),
        Step::KeyControl(C::MappingPad(1)),
        Step::Choice("map_gate", "A".into()),
        Step::Choice("map_pin", "Input".into()),
        Step::KeyControl(C::Preview),
        Step::Refusal,
        Step::Capture("map-duplicate-pin-refusal"),
        Step::Invariant,
        Step::Choice("map_gate", "PWR".into()),
        Step::Mapping(5, true),
        Step::Choice("map_pin", "VCC".into()),
        Step::Choice("map_gate", "B".into()),
        Step::Mapping(5, true),
        Step::Choice("map_pin", "Input".into()),
        Step::Choice("map_default", "Explicitly set".into()),
        Step::KeyControl(C::Preview),
        Step::Mapping(1, true),
        Step::Capture("map-default-preview"),
        Step::Invariant,
        Step::Focus(C::MappingPad(0)),
        Step::Capture("map-complete-table"),
        Step::KeyControl(C::Commit),
        Step::Mapping(2, true),
        Step::History(false),
        Step::Mapping(3, true),
        Step::History(true),
        Step::Mapping(2, true),
        Step::Save,
        Step::Mapping(2, true),
        Step::Reopen,
        Step::Mapping(2, true),
    ];
    steps.extend([
        Step::Open,
        Step::Kind(7),
        Step::SelectMapping,
        Step::Capture("map-reopened"),
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
