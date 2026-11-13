//! Padstack form ownership: exact copper shape and explicitly edited process inputs.
use super::*;

impl LibraryDraft {
    pub(super) fn padstack_fields(&mut self, raw: &Value) {
        let aperture = raw.get("aperture").unwrap_or(&Value::Null);
        let shape = if aperture.get("circle").is_some() {
            "circle"
        } else if aperture.get("rect").is_some() {
            "rect"
        } else if aperture.get("round_rect").is_some() {
            "round_rect"
        } else {
            ""
        };
        self.field(
            "shape",
            "Copper shape",
            Some(&json!(shape)),
            choices(&[
                ("circle", "Circle"),
                ("rect", "Rect"),
                ("round_rect", "Rounded rectangle"),
            ]),
        );
        self.field(
            "width",
            "Diameter / width (nm)",
            aperture
                .pointer("/circle/diameter_nm")
                .or_else(|| aperture.pointer("/rect/width_nm"))
                .or_else(|| aperture.pointer("/round_rect/width_nm")),
            vec![],
        );
        self.field(
            "height",
            "Rectangle height (nm)",
            aperture
                .pointer("/rect/height_nm")
                .or_else(|| aperture.pointer("/round_rect/height_nm")),
            vec![],
        );
        self.field(
            "drill",
            "Drill diameter (nm; blank = absent)",
            raw.get("drill_nm"),
            vec![],
        );
        self.field(
            "plated",
            "Plating",
            raw.get("plated"),
            choices(&[("true", "Plated"), ("false", "Unplated")]),
        );
        self.field(
            "span",
            "Layer span",
            raw.get("layer_span"),
            choices(&[("pad_layer", "Pad layer"), ("through", "Through")]),
        );
        for (key, label, options) in [
            (
                "mask",
                "Mask policy",
                choices(&[
                    ("inherit", "Inherit"),
                    ("exposed", "Exposed"),
                    ("tented", "Tented"),
                    ("expansion_nm", "Expansion"),
                ]),
            ),
            (
                "paste",
                "Paste policy",
                choices(&[
                    ("inherit", "Inherit"),
                    ("none", "Suppressed"),
                    ("aperture", "Copper aperture"),
                    ("expansion_nm", "Expansion"),
                ]),
            ),
        ] {
            let source = raw.get(if key == "mask" {
                "mask_policy"
            } else {
                "paste_policy"
            });
            let policy = source
                .and_then(Value::as_str)
                .map(str::to_owned)
                .or_else(|| {
                    source
                        .filter(|v| v.get("expansion_nm").is_some())
                        .map(|_| "expansion_nm".into())
                })
                .unwrap_or_default();
            self.field(key, label, Some(&json!(policy)), options);
            self.field(
                if key == "mask" {
                    "mask_margin"
                } else {
                    "paste_margin"
                },
                if key == "mask" {
                    "Mask expansion (nm)"
                } else {
                    "Paste expansion (nm)"
                },
                source.and_then(|v| v.get("expansion_nm")),
                vec![],
            );
        }
        self.field(
            "rratio_ppm",
            "Rounded corner ratio (ppm; 0–500000)",
            aperture.pointer("/round_rect/rratio_ppm"),
            vec![],
        );
    }

    pub(super) fn padstack_request(&self, fields: &mut Map<String, Value>) -> Result<(), String> {
        let changed = |keys: &[&str]| self.create || keys.iter().any(|k| self.dirty.contains(k));
        if self.dirty.contains("rratio_ppm") && self.value("shape") != "round_rect" {
            return Err("Corner ratio applies only to Rounded rectangle copper geometry".into());
        }
        if changed(&["shape", "width", "height", "rratio_ppm"]) {
            let aperture = match self.value("shape") {
                "circle" => json!({"circle":{"diameter_nm":self.length("width")?}}),
                "rect" => {
                    json!({"rect":{"width_nm":self.length("width")?,"height_nm":self.length("height")?}})
                }
                "round_rect" => {
                    let ratio = self
                        .value("rratio_ppm")
                        .parse::<u32>()
                        .ok()
                        .filter(|ratio| *ratio <= 500_000)
                        .ok_or("Enter a rounded corner ratio from 0 to 500000 ppm")?;
                    json!({"round_rect":{"width_nm":self.length("width")?,"height_nm":self.length("height")?,"rratio_ppm":ratio}})
                }
                _ => return Err("Choose Circle, Rect or Rounded rectangle copper geometry".into()),
            };
            fields.insert("aperture".into(), aperture);
        }
        for (key, target) in [
            ("drill", "drill_nm"),
            ("plated", "plated"),
            ("span", "layer_span"),
        ] {
            if self.dirty.contains(key) || self.create && !self.value(key).is_empty() {
                let v = match key {
                    "drill" => {
                        if self.value(key).is_empty() {
                            Value::Null
                        } else {
                            json!(self.length(key)?)
                        }
                    }
                    "plated" => match self.value(key) {
                        "true" => json!(true),
                        "false" => json!(false),
                        _ => return Err("Choose a plating value".into()),
                    },
                    _ => json!(self.value(key)),
                };
                fields.insert(target.into(), v);
            }
        }
        for (key, margin, target) in [
            ("mask", "mask_margin", "mask_policy"),
            ("paste", "paste_margin", "paste_policy"),
        ] {
            if self.dirty.contains(key)
                || self.dirty.contains(margin)
                || self.create && !self.value(key).is_empty()
            {
                let v = if self.value(key) == "expansion_nm" {
                    json!({"expansion_nm":self.length(margin)?})
                } else {
                    json!(self.value(key))
                };
                fields.insert(target.into(), v);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saved() -> LibraryDraft {
        let id = Uuid::new_v4();
        let entry = LibraryEntry {
            id,
            kind: "padstacks".into(),
            name: "Imported rounded land".into(),
            source_path: format!("pool/padstacks/{id}.json"),
            revision: Some(1),
            raw: json!({"uuid":id,"name":"Imported rounded land", "source_extension":{"keep":true},
                "aperture":{"round_rect":{"width_nm":800000,"height_nm":950000,"rratio_ppm":250000,"vendor":"retained by engine merge"}},
                "layer_span":"pad_layer", "mask_policy":"inherit", "paste_policy":"inherit"}),
            warning: None,
        };
        LibraryDraft::begin(
            &LibraryBrowser::default(),
            Some(&entry),
            "padstacks",
            "rev".into(),
            None,
        )
        .unwrap()
    }

    fn edit(draft: &mut LibraryDraft, key: &str, text: &str) {
        draft.focused = draft
            .fields
            .iter()
            .position(|field| field.key == key)
            .unwrap();
        draft.change(text.into());
    }

    #[test]
    fn imported_rounded_shape_hydrates_and_name_only_edits_leave_geometry_untouched() {
        let mut draft = saved();
        assert_eq!(draft.value("shape"), "round_rect");
        assert_eq!(draft.value("width"), "800000");
        assert_eq!(draft.value("height"), "950000");
        assert_eq!(draft.value("rratio_ppm"), "250000");
        assert!(draft.request().unwrap().fields.is_empty());
        edit(&mut draft, "name", "Renamed land");
        assert_eq!(
            draft.request().unwrap().fields,
            json!({"name":"Renamed land"}).as_object().unwrap().clone()
        );
    }

    #[test]
    fn rounded_dimension_edits_retain_ratio_and_explicit_ratio_edits_are_bounded() {
        let mut draft = saved();
        edit(&mut draft, "width", "900000");
        let request = draft.request().unwrap();
        assert_eq!(
            request.fields["aperture"],
            json!({"round_rect":{"width_nm":900000,"height_nm":950000,"rratio_ppm":250000}})
        );
        assert!(!request.fields.contains_key("mask_policy"));
        edit(&mut draft, "rratio_ppm", "500001");
        assert!(draft.request().unwrap_err().contains("0 to 500000"));
        edit(&mut draft, "rratio_ppm", "125000");
        assert_eq!(
            draft.request().unwrap().fields["aperture"]["round_rect"]["rratio_ppm"],
            125000
        );
        edit(&mut draft, "shape", "circle");
        assert!(
            draft
                .request()
                .unwrap_err()
                .contains("only to Rounded rectangle")
        );
    }
}
