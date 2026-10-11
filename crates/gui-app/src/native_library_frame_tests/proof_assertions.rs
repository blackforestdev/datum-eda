//! Independent explicit-intent and full-payload history assertions.
use super::*;
impl Fixture {
    pub(super) fn record_placed(&mut self, stack: bool) {
        let s = self.app.runtime.as_ref().unwrap().workspace();
        let e = s.library.selected(&s.selection).unwrap();
        assert_eq!(e.kind, if stack { "padstacks" } else { "footprints" });
        self.placed_source = Some((e.id, e.raw.clone(), e.revision));
    }
    pub(super) fn check_placed(&self, stack: bool) {
        let (id, before, revision) = self.placed_source.as_ref().unwrap();
        let s = self.app.runtime.as_ref().unwrap().workspace();
        let e = s.library.entries.iter().find(|e| e.id == *id).unwrap();
        let mut expected = before.clone();
        if stack {
            if expected["aperture"].get("circle").is_some() {
                expected["aperture"]["circle"]["diameter_nm"] = serde_json::json!(1200000);
            } else {
                expected["aperture"]["rect"]["width_nm"] = serde_json::json!(1200000);
            }
        } else {
            let pad = expected["pads"]
                .as_object()
                .unwrap()
                .keys()
                .next()
                .unwrap()
                .clone();
            expected["pads"][pad]["position"]["x"] = serde_json::json!(-2000000);
        }
        assert_ne!(
            &expected, before,
            "explicit edit must have a material source effect"
        );
        assert_eq!(
            e.raw, expected,
            "only intended source field changes; UUIDs, raw siblings, process basis and optional absence remain exact"
        );
        assert!(e.revision.unwrap() > revision.unwrap());
    }
    pub(super) fn check_process(&self, index: usize) {
        use eda_engine::pool::process::EffectiveProcessAperture as A;
        let s = self.app.runtime.as_ref().unwrap().workspace();
        assert_eq!(s.library.preview_pad, index);
        let e = s.library.selected(&s.selection).unwrap();
        assert_eq!(e.name, "Native process footprint");
        let p = s
            .library
            .process_inspection(&e.kind, &e.raw, index)
            .unwrap();
        fn dimensions(aperture: &A) -> (&'static str, i64, i64) {
            match aperture {
                A::Circle { diameter_nm } => (
                    "Circle",
                    diameter_nm.integer_nm().unwrap(),
                    diameter_nm.integer_nm().unwrap(),
                ),
                A::Rect {
                    width_nm,
                    height_nm,
                } => (
                    "Rect",
                    width_nm.integer_nm().unwrap(),
                    height_nm.integer_nm().unwrap(),
                ),
                A::Suppressed => ("Suppressed", 0, 0),
            }
        }
        if index == 0 {
            assert_eq!(dimensions(&p.mask), ("Circle", 1000001, 1000001));
            assert_eq!(dimensions(&p.paste), ("Circle", 999997, 999997));
            assert!(
                p.resolved_basis.is_none(),
                "explicit policies do not inherit"
            );
        } else {
            assert_eq!(dimensions(&p.mask), ("Rect", 2000020, 1000020));
            assert_eq!(dimensions(&p.paste), ("Rect", 2399960, 1199960));
            let basis = p.resolved_basis.as_ref().unwrap();
            assert_eq!(basis.declared_basis, "Designer process profile");
            assert_eq!(basis.paste_ratio_ppm, 200000);
            assert!(p.selected_basis.is_some());
        }
        eprintln!(
            "selected pad {index} effective process: {}",
            serde_json::to_string(&p).unwrap()
        );
    }
    pub(super) fn check_removed_preview(&self) {
        let s = self.app.runtime.as_ref().unwrap().workspace();
        let draft = s.library.draft.as_ref().unwrap();
        let preview = draft.preview.as_ref().expect("Remove Pad is previewable");
        let mut expected = self.original_footprint.as_ref().unwrap().clone();
        let first = expected["pads"]
            .as_object()
            .unwrap()
            .keys()
            .next()
            .unwrap()
            .clone();
        expected["pads"].as_object_mut().unwrap().remove(&first);
        assert_eq!(
            preview, &expected,
            "remove only selected PadID and retain every other authored field"
        );
        assert_eq!(
            source_bytes(&self.root),
            self.snapshot,
            "removal preview cannot write"
        );
    }
    pub(super) fn record_footprint(&mut self) {
        let e = self
            .app
            .runtime
            .as_ref()
            .unwrap()
            .workspace()
            .library
            .entries
            .iter()
            .find(|e| e.name == "GUI two pad resistor")
            .unwrap();
        let library = &self.app.runtime.as_ref().unwrap().workspace().library;
        let stack = library
            .entries
            .iter()
            .find(|e| e.name == "GUI resistor copper" && e.kind == "padstacks")
            .unwrap();
        assert_eq!(
            stack.raw,
            serde_json::json!({"schema_version":1,"uuid":stack.id,"name":"GUI resistor copper","aperture":{"rect":{"width_nm":1000000,"height_nm":500000}},"drill_nm":200000,"mask_policy":"exposed","paste_policy":"aperture"}),
            "new Padstack must match explicit intent and retain omitted optional fields"
        );
        assert_eq!(e.raw["package"], serde_json::json!(self.expected_package));
        let pads = e.raw["pads"].as_object().unwrap();
        assert_eq!(pads.len(), 2);
        for (name, x) in [("1", -1000000), ("2", 1000000)] {
            let (id, pad) = pads
                .iter()
                .find(|(_, p)| p["name"] == name)
                .expect("intended named pad exists");
            assert_eq!(
                pad,
                &serde_json::json!({"uuid":id,"name":name,"position":{"x":x,"y":0},"padstack":stack.id,"layer":1}),
                "coordinates, identity and named Padstack reference must match intent"
            );
        }
        self.original_footprint = Some(e.raw.clone());
        self.original_revision = e.revision;
    }
    pub(super) fn check_footprint_value(&mut self, edited: bool) {
        let e = self
            .app
            .runtime
            .as_ref()
            .unwrap()
            .workspace()
            .library
            .entries
            .iter()
            .find(|e| e.name == "GUI two pad resistor")
            .unwrap();
        let before = self.original_footprint.as_ref().unwrap();
        let mut expected = before.clone();
        if edited {
            let pad = expected["pads"]
                .as_object()
                .unwrap()
                .keys()
                .next()
                .unwrap()
                .clone();
            expected["pads"][pad]["position"]["x"] = serde_json::json!(-1500000);
            assert!(e.revision.unwrap() > self.original_revision.unwrap());
            self.edited_footprint = Some(expected.clone());
        }
        assert_eq!(
            e.raw, expected,
            "complete payload including IDs, references, optional absence and geometry must match history effect"
        );
    }
}
