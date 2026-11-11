//! Read-only ordinary App proof of owning-side names on the authored TERU circuit.
use super::*;
use datum_gui_protocol::{BoardReviewSceneV1, PointNm};
use serde_json::{Value, json};
use uuid::Uuid;

fn label(
    scene: &BoardReviewSceneV1,
    owner: Uuid,
    pin: Uuid,
    index: usize,
    name: &str,
    at: PointNm,
) -> Value {
    let id = Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum:schematic-projection-text:{owner}:pin-name:{index}").as_bytes(),
    )
    .to_string();
    let text = scene
        .board_texts
        .iter()
        .find(|text| text.text_uuid == id)
        .unwrap();
    assert_eq!(text.text, name);
    assert_eq!(text.source_object_uuid, Some(pin.to_string()));
    assert_eq!(
        text.position, at,
        "insufficient body space permits only own-pin ink clearance"
    );
    let geometry = scene
        .board_text_geometries
        .iter()
        .find(|geometry| geometry.text_uuid == id)
        .unwrap();
    assert!(!geometry.glyphs.is_empty());
    assert!(datum_gui_protocol::selection_geometry_catalog::layout_polygon(geometry).is_some());
    json!({"text":text,"geometry":geometry})
}

impl Fixture {
    pub(super) fn opposed_pin_labels(&self, stage: usize) {
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let record = if stage == 0 {
            let symbol = state
                .library
                .entries
                .iter()
                .find(|e| e.kind == "symbols" && e.name == "Teru component")
                .unwrap();
            let unit = state
                .library
                .entries
                .iter()
                .find(|e| e.id.to_string() == symbol.raw["unit"].as_str().unwrap())
                .unwrap();
            let scene = datum_gui_protocol::library_browser::symbol_geometry::preview::project(
                &symbol.raw,
                Some(&unit.raw),
            )
            .unwrap();
            let mut labels = Vec::new();
            for (index, anchor) in symbol.raw["pin_anchors"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
            {
                let pin: Uuid = anchor["pin"].as_str().unwrap().parse().unwrap();
                let name = unit.raw["pins"][pin.to_string()]["name"].as_str().unwrap();
                let x = if name == "TERU_IN" { -949_200 } else { 949_200 };
                labels.push(label(
                    &scene,
                    symbol.id,
                    pin,
                    index,
                    name,
                    PointNm { x, y: 0 },
                ));
            }
            assert_eq!(labels.len(), 2);
            json!({"symbol":symbol.raw,"unit":unit.raw,"labels":labels,"graphics":scene.board_graphics})
        } else {
            let child = stage == 11;
            let row = state
                .schematic_documents
                .rows
                .iter()
                .find(|r| r.parent.is_some() == child)
                .unwrap();
            let path = self
                .root
                .join(format!("schematic/sheets/{}.json", row.key.source_sheet));
            let sheet: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            let symbol = sheet["symbols"]
                .as_object()
                .unwrap()
                .values()
                .find(|s| s["reference"] == if child { "TC2" } else { "TC1" })
                .unwrap();
            let owner: Uuid = symbol["uuid"].as_str().unwrap().parse().unwrap();
            let scene =
                datum_gui_protocol::camera_scene_for_pane(state, state.ui.layout.focused).unwrap();
            let mut labels = Vec::new();
            for (index, pin) in symbol["pins"].as_array().unwrap().iter().enumerate() {
                let name = pin["name"].as_str().unwrap();
                let x = symbol["position"]["x"].as_i64().unwrap()
                    + if name == "TERU_IN" { -949_200 } else { 949_200 };
                labels.push(label(
                    &scene,
                    owner,
                    pin["uuid"].as_str().unwrap().parse().unwrap(),
                    index,
                    name,
                    PointNm {
                        x,
                        y: symbol["position"]["y"].as_i64().unwrap(),
                    },
                ));
            }
            assert_eq!(labels.len(), 2);
            json!({"sheet":row.key.source_sheet,"instance_path":row.key.instance_path,"placed":symbol,"labels":labels})
        };
        let path = self.output.join(format!("opposed-labels-{stage}.json"));
        if path.exists() {
            let old: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            assert_eq!(
                record, old,
                "cold reopen retains the same source-bound label geometry"
            );
        } else {
            std::fs::write(path, serde_json::to_vec_pretty(&record).unwrap()).unwrap();
        }
    }
}

#[test]
#[ignore = "requires saved TERU circuit and owned isolated display"]
fn ordinary_opposed_names_retain_owning_side_in_preview_root_child_and_cold() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let mut steps = vec![
        Step::Snapshot,
        Step::Open,
        Step::Kind(1),
        Step::Search("Teru component".into()),
        Step::Select("Teru component".into()),
        Step::OpposedPinLabels(0),
        Step::KeyControl(C::Edit),
        Step::KeyControl(C::Preview),
        Step::Capture("opposed-library-preview"),
        Step::KeyControl(C::Cancel),
        Step::Invariant,
    ];
    for cold in [false, true] {
        if cold {
            steps.extend([Step::Reopen, Step::Invariant]);
        }
        for child in [false, true] {
            steps.extend([
                Step::PaneContext(Expand),
                Step::PaneContext(Navigate(child)),
                Step::PinLabelClearance(2),
                Step::PaneContext(Fit),
                Step::OpposedPinLabels(if child { 11 } else { 10 }),
                Step::Capture(match (child, cold) {
                    (false, false) => "opposed-root-open",
                    (true, false) => "opposed-child-open",
                    (false, true) => "opposed-root-cold",
                    (true, true) => "opposed-child-cold",
                }),
                Step::Invariant,
            ]);
        }
    }
    scenario::run(root, output, steps, Uuid::nil());
}
