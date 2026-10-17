//! Disposable form intent. Only explicitly changed fields enter a native write.
use crate::library_browser::{LibraryBrowser, LibraryEntry};
use eda_engine::{
    api::native_write::{
        WriteProvenance,
        library_gui::{LibraryDefinitionRequest, build_definition},
        registry::NativeWriteContext,
    },
    ir::units::{LengthUnit, parse_fixed_length},
    substrate::CommitSource,
};
use serde_json::{Map, Value, json};
use std::{collections::BTreeSet, path::Path};
use uuid::Uuid;

mod package;

#[derive(Debug, Clone, PartialEq)]
pub struct LibraryField {
    pub key: &'static str,
    pub label: &'static str,
    pub text: String,
    pub choices: Vec<(String, String)>,
    pub selector: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryDraft {
    pub create: bool,
    pub id: Uuid,
    pub kind: String,
    pub revision: String,
    pub tip: Option<Uuid>,
    pub fields: Vec<LibraryField>,
    pub dirty: BTreeSet<&'static str>,
    pub focused: usize,
    pub offset: usize,
    pub pad_id: Option<Uuid>,
    pub pads: Map<String, Value>,
    pub removed: BTreeSet<String>,
    pub original_pads: BTreeSet<String>,
    pub terminal_id: Option<Uuid>,
    pub terminals: Map<String, Value>,
    pub preview: Option<Value>,
    pub refusal: Option<String>,
}
impl LibraryDraft {
    pub fn begin(
        browser: &LibraryBrowser,
        entry: Option<&LibraryEntry>,
        kind: &str,
        revision: String,
        tip: Option<Uuid>,
    ) -> Result<Self, String> {
        if !matches!(kind, "footprints" | "padstacks" | "packages") {
            return Err(
                "Authoring is available for Footprint, Padstack and Package creation".into(),
            );
        }
        if kind == "packages" && entry.is_some() {
            return Err("Saved Package editing is unavailable in this increment".into());
        }
        if kind == "footprints" && entry.is_none() && browser.choices("packages").is_empty() {
            return Err(
                "Footprint creation unavailable: no existing named Package in the project pool"
                    .into(),
            );
        }
        if entry.is_some_and(|e| !e.editable()) {
            return Err("This source is available for inspection only".into());
        }
        let mut draft = Self {
            create: entry.is_none(),
            id: entry.map_or_else(Uuid::new_v4, |e| e.id),
            kind: kind.into(),
            revision,
            tip,
            fields: Vec::new(),
            dirty: BTreeSet::new(),
            focused: 0,
            offset: 0,
            pad_id: None,
            pads: Map::new(),
            removed: BTreeSet::new(),
            original_pads: BTreeSet::new(),
            terminal_id: None,
            terminals: Map::new(),
            preview: None,
            refusal: None,
        };
        let raw = entry.map(|e| &e.raw).unwrap_or(&Value::Null);
        draft.field("name", "Name", raw.get("name"), vec![]);
        if kind == "packages" {
            draft.package_fields();
        } else if kind == "padstacks" {
            let aperture = raw.get("aperture").unwrap_or(&Value::Null);
            let shape = if aperture.get("circle").is_some() {
                "circle"
            } else if aperture.get("rect").is_some() {
                "rect"
            } else {
                ""
            };
            draft.field(
                "shape",
                "Copper shape",
                Some(&json!(shape)),
                choices(&[("circle", "Circle"), ("rect", "Rect")]),
            );
            draft.field(
                "width",
                "Diameter / width (nm)",
                aperture
                    .pointer("/circle/diameter_nm")
                    .or_else(|| aperture.pointer("/rect/width_nm")),
                vec![],
            );
            draft.field(
                "height",
                "Rect height (nm)",
                aperture.pointer("/rect/height_nm"),
                vec![],
            );
            draft.field(
                "drill",
                "Drill diameter (nm; blank = absent)",
                raw.get("drill_nm"),
                vec![],
            );
            draft.field(
                "plated",
                "Plating",
                raw.get("plated"),
                choices(&[("true", "Plated"), ("false", "Unplated")]),
            );
            draft.field(
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
                draft.field(key, label, Some(&json!(policy)), options);
                draft.field(
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
        } else {
            draft.pads = raw
                .get("pads")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            draft.original_pads = draft.pads.keys().cloned().collect();
            draft.field(
                "package",
                "Named Package",
                raw.get("package"),
                named(browser, "packages"),
            );
            let pad = raw
                .get("pads")
                .and_then(Value::as_object)
                .and_then(|p| p.iter().next());
            draft.pad_id = pad.and_then(|(id, _)| Uuid::parse_str(id).ok());
            let pad = pad.map(|(_, v)| v).unwrap_or(&Value::Null);
            draft.field(
                "pad_name",
                "Pad name (blank = no new pad)",
                pad.get("name"),
                vec![],
            );
            draft.field(
                "padstack",
                "Named Padstack",
                pad.get("padstack"),
                named(browser, "padstacks"),
            );
            draft.field("x", "Pad X (nm)", pad.pointer("/position/x"), vec![]);
            draft.field("y", "Pad Y (nm)", pad.pointer("/position/y"), vec![]);
            draft.field("layer", "Pad layer", pad.get("layer"), vec![]);
        }
        Ok(draft)
    }
    fn field(
        &mut self,
        key: &'static str,
        label: &'static str,
        value: Option<&Value>,
        choices: Vec<(String, String)>,
    ) {
        let text = match value {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::Bool(b)) => b.to_string(),
            _ => String::new(),
        };
        self.fields.push(LibraryField {
            key,
            label,
            text,
            choices,
            selector: matches!(
                key,
                "shape" | "plated" | "span" | "mask" | "paste" | "package" | "padstack"
            ),
        });
    }
    pub fn value(&self, key: &str) -> &str {
        self.fields
            .iter()
            .find(|f| f.key == key)
            .map_or("", |f| f.text.as_str())
    }
    pub fn change(&mut self, text: String) {
        if self.fields[self.focused].key == "terminal_name"
            && self.terminal_id.is_none()
            && !text.trim().is_empty()
        {
            self.terminal_id = Some(Uuid::new_v4());
        }
        if self.fields[self.focused].key == "pad_name"
            && self.pad_id.is_none()
            && !text.trim().is_empty()
        {
            self.pad_id = Some(Uuid::new_v4());
        }
        let field = &mut self.fields[self.focused];
        field.text = text;
        self.dirty.insert(field.key);
        self.preview = None;
        self.refusal = None;
    }
    pub fn cycle(&mut self, forward: bool) {
        let field = &self.fields[self.focused];
        if field.choices.is_empty() {
            return;
        }
        let n = field.choices.len();
        let index = field.choices.iter().position(|c| c.0 == field.text);
        let next = index.map_or(0, |i| {
            if forward {
                (i + 1) % n
            } else {
                (i + n - 1) % n
            }
        });
        self.change(field.choices[next].0.clone());
    }
    pub fn request(&self) -> Result<LibraryDefinitionRequest, String> {
        let mut fields = Map::new();
        let changed = |keys: &[&str]| self.create || keys.iter().any(|k| self.dirty.contains(k));
        if changed(&["name"]) {
            fields.insert("name".into(), json!(self.value("name")));
        }
        if self.kind == "packages" {
            self.package_request(&mut fields)?;
        } else if self.kind == "padstacks" {
            if changed(&["shape", "width", "height"]) {
                let aperture = match self.value("shape") {
                    "circle" => json!({"circle":{"diameter_nm":self.length("width")?}}),
                    "rect" => {
                        json!({"rect":{"width_nm":self.length("width")?,"height_nm":self.length("height")?}})
                    }
                    _ => return Err("Choose Circle or Rect copper geometry".into()),
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
        } else {
            if changed(&["package"]) {
                fields.insert("package".into(), json!(self.uuid("package")?));
            }
            let mut pads = self.pads.clone();
            for id in &self.removed {
                pads.insert(id.clone(), Value::Null);
            }
            if changed(&["pad_name", "padstack", "x", "y", "layer"])
                && !self.value("pad_name").trim().is_empty()
            {
                // Identity is allocated at draft start, never during repeated preview.
                let id = self.pad_id.ok_or("Start a named pad before preview")?;
                let layer = self
                    .value("layer")
                    .parse::<i32>()
                    .map_err(|_| "Enter a positive layer number")?;
                if layer <= 0 {
                    return Err("Enter a positive layer number".into());
                }
                let pad = pads
                    .entry(id.to_string())
                    .or_insert_with(|| json!({"uuid":id}))
                    .as_object_mut()
                    .ok_or("Pad source is not an object")?;
                for (key, value) in [
                    ("name", json!(self.value("pad_name"))),
                    ("padstack", json!(self.uuid("padstack")?)),
                    (
                        "position",
                        json!({"x":self.length("x")?,"y":self.length("y")?}),
                    ),
                    ("layer", json!(layer)),
                ] {
                    if key == "position"
                        && let Some(p) = pad.get_mut(key).and_then(Value::as_object_mut)
                    {
                        p.extend(value.as_object().unwrap().clone());
                    } else {
                        pad.insert(key.into(), value);
                    }
                }
            }
            if self.create
                || changed(&["pad_name", "padstack", "x", "y", "layer", "pads"])
                || !self.removed.is_empty()
            {
                if !self.create {
                    for pad in pads.values_mut() {
                        if let Some(raw) = pad.as_object_mut() {
                            raw.retain(|k, _| {
                                ["uuid", "name", "position", "padstack", "layer"]
                                    .contains(&k.as_str())
                            });
                        }
                    }
                }
                fields.insert("pads".into(), Value::Object(pads));
            }
        }
        Ok(LibraryDefinitionRequest {
            kind: self.kind.clone(),
            object_id: self.id,
            pool_path: "pool".into(),
            fields,
        })
    }
    fn length(&self, key: &str) -> Result<i64, String> {
        parse_fixed_length(self.value(key), LengthUnit::Nanometer)
            .map(i64::from)
            .map_err(|e| {
                format!(
                    "{}: {e:?}",
                    self.fields.iter().find(|f| f.key == key).unwrap().label
                )
            })
    }
    fn uuid(&self, key: &str) -> Result<Uuid, String> {
        Uuid::parse_str(self.value(key)).map_err(|_| format!("Choose an existing named {key}"))
    }
    pub fn validate_preview(
        &mut self,
        browser: &LibraryBrowser,
        root: &Path,
    ) -> Result<(), String> {
        if self.kind == "footprints" && !self.value("pad_name").is_empty() && self.pad_id.is_none()
        {
            self.pad_id = Some(Uuid::new_v4());
        }
        let request = self.request()?;
        let model = browser
            .basis
            .as_ref()
            .ok_or("Resolved project source unavailable")?;
        let prepared = build_definition(
            &NativeWriteContext {
                model,
                project_root: root,
            },
            WriteProvenance::new("datum-gui", CommitSource::Manual, "Library preview"),
            request,
            self.create,
        )
        .map_err(|e| e.to_string())?;
        // The prepared batch is inspectable, build-only evidence; never a write.
        self.preview = prepared
            .batch
            .operations
            .into_iter()
            .find_map(|op| match op {
                eda_engine::substrate::Operation::CreatePoolLibraryObject { object, .. }
                | eda_engine::substrate::Operation::SetPoolLibraryObject { object, .. } => {
                    Some(object)
                }
                _ => None,
            });
        self.refusal = None;
        Ok(())
    }
    pub fn switch_pad(&mut self, index: Option<usize>, remove: bool) -> Result<(), String> {
        if self.pad_id.is_some() && !self.value("pad_name").is_empty() {
            let request = self.request()?;
            if let Some(pads) = request.fields.get("pads").and_then(Value::as_object) {
                for (id, pad) in pads {
                    if !pad.is_null() {
                        self.pads.insert(id.clone(), pad.clone());
                    }
                }
            }
        }
        if remove {
            if let Some(id) = self.pad_id.take() {
                self.pads.remove(&id.to_string());
                if self.original_pads.contains(&id.to_string()) {
                    self.removed.insert(id.to_string());
                }
            }
        }
        let selected = index
            .and_then(|i| self.pads.iter().nth(i))
            .map(|(id, p)| (id.clone(), p.clone()));
        self.pad_id = selected
            .as_ref()
            .and_then(|(id, _)| Uuid::parse_str(id).ok())
            .or_else(|| (!remove).then(Uuid::new_v4));
        let raw = selected.map(|(_, p)| p).unwrap_or(Value::Null);
        for (field, path) in [
            ("pad_name", "/name"),
            ("padstack", "/padstack"),
            ("x", "/position/x"),
            ("y", "/position/y"),
            ("layer", "/layer"),
        ] {
            let f = self.fields.iter_mut().find(|f| f.key == field).unwrap();
            f.text = raw
                .pointer(path)
                .filter(|v| !v.is_null())
                .map(|v| {
                    v.as_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| v.to_string())
                })
                .unwrap_or_default();
            self.dirty.remove(field);
        }
        self.dirty.insert("pads");
        self.preview = None;
        Ok(())
    }
}
fn choices(values: &[(&str, &str)]) -> Vec<(String, String)> {
    values
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}
fn named(browser: &LibraryBrowser, kind: &str) -> Vec<(String, String)> {
    browser
        .choices(kind)
        .into_iter()
        .map(|i| {
            let e = &browser.entries[i];
            (e.id.to_string(), format!("{} · {}", e.name, e.source_path))
        })
        .collect()
}
