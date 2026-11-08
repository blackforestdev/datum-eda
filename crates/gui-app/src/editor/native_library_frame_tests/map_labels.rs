//! Saved native maps: discoverable captions, exact relationships, read-only cold use.
use super::*;
use serde_json::{Value, json};
use uuid::Uuid;

const MAPS: [&str; 2] = [
    "25a2d6b3-6552-4682-9e36-1b030b5674d3",
    "d8fa6259-f928-4674-9994-682f420b0cca",
];
const PARTS: [&str; 2] = [
    "60c35837-8b87-4aa8-a20a-2f4eb2bcf0e2",
    "ad6034be-ec64-4e72-8469-609c9256b29b",
];
const FOOT: &str = "745fac73-f47a-44e3-9e06-3268a7de2dc7";
fn caption(which: usize) -> String {
    format!(
        "Part: Designer {} · Footprint: Land",
        if which == 0 { "PLACED" } else { "ELIGIBLE" }
    )
}

impl Fixture {
    pub(super) fn check_map_labels(&mut self, stage: usize) {
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let browser = &state.library;
        let inventory:Vec<_>=browser.entries.iter().map(|entry|json!({"id":entry.id,"kind":entry.kind,"caption":entry.name,"key":entry.key(),"path":entry.source_path,"revision":entry.revision,"warning":entry.warning,"raw":entry.raw})).collect();
        if stage == 0 {
            std::fs::write(
                self.output.join("map-label-basis.json"),
                serde_json::to_vec_pretty(&inventory).unwrap(),
            )
            .unwrap();
        } else if stage == 5 {
            let before: Value = serde_json::from_slice(
                &std::fs::read(self.output.join("map-label-basis.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(
                json!(inventory),
                before,
                "source order, separate keys, raw payload, warnings, revisions and captions survive browsing and cold open"
            );
        }
        let maps: Vec<_> = browser
            .entries
            .iter()
            .filter(|entry| entry.kind == "pin_pad_maps")
            .collect();
        assert_eq!(maps.len(), 2);
        for which in 0..2 {
            let id = Uuid::parse_str(MAPS[which]).unwrap();
            let map = maps.iter().find(|entry| entry.id == id).unwrap();
            assert_eq!(map.name, caption(which));
            assert!(map.warning.is_none());
            assert_eq!(map.raw["part"], json!(PARTS[which]));
            assert_eq!(map.raw["footprint"], json!(FOOT));
            assert!(
                map.raw.get("name").is_none(),
                "caption is not an authored map name"
            );
            let links = browser.links(map);
            for (role, target) in [("part", PARTS[which]), ("footprint", FOOT)] {
                let link = links.iter().find(|link| link.role == role).unwrap();
                assert_eq!(link.id.to_string(), target);
                assert_eq!(link.targets.len(), 1);
                assert_eq!(browser.entries[link.targets[0]].id, link.id);
                assert!(browser.entries[link.targets[0]].warning.is_none());
            }
        }
        if matches!(stage, 1 | 2) {
            let which = stage - 1;
            let selected = browser.selected(&state.selection).unwrap();
            assert_eq!(selected.id.to_string(), MAPS[which]);
            assert_eq!(selected.name, caption(which));
            assert_eq!(
                browser.results().len(),
                1,
                "declared human identity finds one exact saved map"
            );
            assert_eq!(browser.entries[browser.results()[0]].id, selected.id);
        } else if matches!(stage, 3 | 4) {
            let selected = browser.selected(&state.selection).unwrap();
            assert_eq!(
                selected.id.to_string(),
                if stage == 3 { PARTS[0] } else { FOOT }
            );
            assert_eq!(
                selected.kind,
                if stage == 3 { "parts" } else { "footprints" }
            );
        }
    }
}

#[test]
#[ignore = "requires copied saved run111 native maps and isolated native Wayland"]
fn ordinary_saved_map_captions_search_relationships_and_cold_reopen_without_writes() {
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let steps = vec![
        Step::MapLabels(0),
        Step::Snapshot,
        Step::Open,
        Step::Kind(7),
        Step::Capture("map-label-results"),
        Step::Search("PLACED".into()),
        Step::Select(caption(0)),
        Step::MapLabels(1),
        Step::Capture("map-label-placed"),
        Step::Invariant,
        Step::Relationship,
        Step::MapLabels(3),
        Step::Invariant,
        Step::Kind(7),
        Step::Search("ELIGIBLE".into()),
        Step::Select(caption(1)),
        Step::MapLabels(2),
        Step::Capture("map-label-eligible"),
        Step::Invariant,
        Step::KeyControl(C::Relationship(1)),
        Step::MapLabels(4),
        Step::Invariant,
        Step::Kind(7),
        Step::Search("".into()),
        Step::MapLabels(5),
        Step::Invariant,
        Step::Reopen,
        Step::MapLabels(5),
        Step::Invariant,
        Step::Open,
        Step::Kind(7),
        Step::Search("PLACED".into()),
        Step::Select(caption(0)),
        Step::MapLabels(1),
        Step::Capture("map-label-cold-placed"),
        Step::Invariant,
        Step::Search("ELIGIBLE".into()),
        Step::Select(caption(1)),
        Step::MapLabels(2),
        Step::Capture("map-label-cold-eligible"),
        Step::Invariant,
        Step::MapLabels(5),
    ];
    scenario::run(root, output, steps, Uuid::nil());
}
