//! Disposable Unit pin and Symbol anchor intent; creation through the shared writer.
use super::*;
use std::collections::BTreeMap;

mod saved;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ElectricalDraft {
    pub original: Option<Value>,
    pub unit_label: Option<String>,
    pub pin_id: Option<Uuid>,
    pub pins: Map<String, Value>,
    pub anchors: Map<String, Value>,
    pub anchor_id: Option<Uuid>,
    pub unit_pins: BTreeMap<String, Vec<(String, String)>>,
}

impl ElectricalDraft {
    pub fn selected_new_pin(&self) -> bool {
        self.pin_id.is_some_and(|id| {
            self.pins
                .keys()
                .any(|key| key.parse::<Uuid>().ok() == Some(id))
                && self
                    .original
                    .as_ref()
                    .and_then(|raw| raw["pins"].as_object())
                    .is_some_and(|pins| !pins.keys().any(|k| k.parse::<Uuid>().ok() == Some(id)))
        })
    }
}
fn pin_key(pins: &Map<String, Value>, id: Uuid) -> Result<String, String> {
    let matches: Vec<_> = pins
        .keys()
        .filter(|k| k.parse::<Uuid>().ok() == Some(id))
        .collect();
    if matches.len() != 1 {
        return Err("Logical pin identity is missing or ambiguous".into());
    }
    Ok(matches[0].clone())
}

impl LibraryDraft {
    pub(super) fn electrical_fields(&mut self, browser: &LibraryBrowser) {
        if self.kind == "units" {
            self.electrical.pin_id = Some(Uuid::new_v4());
            self.field("pin_name", "Pin name", None, vec![]);
            self.field(
                "pin_type",
                "Electrical type",
                None,
                [
                    "Input",
                    "Output",
                    "Bidirectional",
                    "Passive",
                    "PowerIn",
                    "PowerOut",
                    "OpenCollector",
                    "OpenEmitter",
                    "TriState",
                    "NoConnect",
                ]
                .into_iter()
                .map(|s| (s.into(), s.into()))
                .collect(),
            );
            return;
        }
        self.electrical.unit_pins = browser
            .entries
            .iter()
            .filter(|e| e.kind == "units" && e.warning.is_none())
            .map(|e| {
                let pins = e
                    .raw
                    .get("pins")
                    .and_then(Value::as_object)
                    .into_iter()
                    .flat_map(|m| m.iter())
                    .filter_map(|(id, pin)| Some((id.clone(), pin.get("name")?.as_str()?.into())))
                    .collect();
                (e.id.to_string(), pins)
            })
            .collect();
        self.field("unit", "Named Unit", None, named(browser, "units"));
        for (key, label) in [
            ("rect_min_x", "Body minimum X (nm)"),
            ("rect_min_y", "Body minimum Y (nm)"),
            ("rect_max_x", "Body maximum X (nm)"),
            ("rect_max_y", "Body maximum Y (nm)"),
            ("rect_width", "Body stroke width (nm)"),
        ] {
            self.field(key, label, None, vec![]);
        }
        self.field("anchor_pin", "Named Unit pin", None, vec![]);
        self.field("anchor_x", "Anchor X (nm)", None, vec![]);
        self.field("anchor_y", "Anchor Y (nm)", None, vec![]);
        self.field(
            "anchor_orientation",
            "Pin stub direction",
            None,
            choices(&[
                ("Left", "Left"),
                ("Right", "Right"),
                ("Up", "Up"),
                ("Down", "Down"),
            ]),
        );
        self.field("anchor_length", "Pin stub length (nm)", None, vec![]);
    }

    pub(super) fn refresh_unit_pin_choices(&mut self) {
        let choices = self
            .electrical
            .unit_pins
            .get(self.value("unit"))
            .cloned()
            .unwrap_or_default();
        if let Some(field) = self.fields.iter_mut().find(|f| f.key == "anchor_pin") {
            field.choices = choices;
            // Retain authored intent; a changed Unit must explicitly resolve old anchors.
        }
    }

    pub fn electrical_pins(&self) -> Result<Map<String, Value>, String> {
        if !self.create && !self.electrical.selected_new_pin() {
            return self.saved_pins();
        }
        let mut pins = self.electrical.pins.clone();
        let name = self.value("pin_name").trim();
        if !name.is_empty() {
            let id = self
                .electrical
                .pin_id
                .ok_or("Add a pin before entering its name")?;
            if self.value("pin_type").is_empty() {
                return Err("Choose the pin electrical type".into());
            }
            pins.insert(
                id.to_string(),
                json!({"name":name,"electrical_type":self.value("pin_type")}),
            );
        } else if !self.value("pin_type").is_empty()
            || self
                .electrical
                .pin_id
                .is_some_and(|id| pins.contains_key(&id.to_string()))
        {
            return Err("Enter a pin name, or remove the draft pin".into());
        }
        Ok(pins)
    }

    pub fn electrical_anchors(&self) -> Result<Map<String, Value>, String> {
        if !self.create {
            return self.saved_anchors();
        }
        let mut anchors = self.electrical.anchors.clone();
        if !self.value("anchor_pin").is_empty() {
            let pin = self.uuid("anchor_pin")?;
            if anchors.contains_key(&pin.to_string()) && self.electrical.anchor_id != Some(pin) {
                return Err(
                    "This Unit pin already has an anchor; select that anchor to edit it".into(),
                );
            }
            if let Some(previous) = self.electrical.anchor_id {
                anchors.remove(&previous.to_string());
            }
            if self.value("anchor_orientation").is_empty() {
                return Err("Choose the pin stub direction".into());
            }
            anchors.insert(
                pin.to_string(),
                json!({
                    "pin":pin,
                    "position":{"x":self.length("anchor_x")?,"y":self.length("anchor_y")?},
                    "style":{"orientation":self.value("anchor_orientation"),
                        "length_nm":self.length("anchor_length")?,"decoration":"none"}
                }),
            );
        } else if self.electrical.anchor_id.is_some()
            || [
                "anchor_x",
                "anchor_y",
                "anchor_orientation",
                "anchor_length",
            ]
            .into_iter()
            .any(|key| !self.value(key).is_empty())
        {
            return Err("Choose a named Unit pin for this anchor".into());
        }
        Ok(anchors)
    }

    pub(super) fn electrical_request(&self, fields: &mut Map<String, Value>) -> Result<(), String> {
        if !self.create {
            return self.saved_request(fields);
        }
        if self.kind == "units" {
            fields.insert("pins".into(), self.electrical_pins()?.into());
        } else {
            fields.insert("unit".into(), json!(self.uuid("unit")?));
            fields.insert(
                "drawings".into(),
                json!([{"Rect":{
                    "min":{"x":self.length("rect_min_x")?,"y":self.length("rect_min_y")?},
                    "max":{"x":self.length("rect_max_x")?,"y":self.length("rect_max_y")?},
                    "width":self.length("rect_width")?
                }}]),
            );
            fields.insert(
                "pin_anchors".into(),
                self.electrical_anchors()?
                    .into_iter()
                    .map(|(_, anchor)| anchor)
                    .collect::<Vec<_>>()
                    .into(),
            );
        }
        Ok(())
    }

    pub fn switch_pin(&mut self, index: Option<usize>, remove: bool) -> Result<(), String> {
        if self.kind != "units" {
            return Err("Logical pin authoring requires a Unit draft".into());
        }
        if !self.create && remove && !self.electrical.selected_new_pin() {
            return Err("Saved edits preserve existing logical pin identities".into());
        }
        let selected = if let Some(index) = index {
            let key = self
                .electrical
                .pins
                .keys()
                .nth(index)
                .ok_or("Choose an existing logical pin")?;
            let id = key.parse().map_err(|_| "Logical pin identity is invalid")?;
            Some(pin_key(&self.electrical.pins, id)?)
        } else {
            None
        };
        if remove {
            if let Some(id) = self.electrical.pin_id {
                self.electrical.pins.remove(&id.to_string());
            }
        } else {
            self.electrical.pins = self.electrical_pins()?;
        }
        let raw = selected
            .as_ref()
            .and_then(|id| self.electrical.pins.get(id))
            .cloned();
        self.electrical.pin_id = selected
            .and_then(|id| id.parse().ok())
            .or_else(|| (!remove).then(Uuid::new_v4));
        if !self.create && index.is_none() && !remove {
            // The explicit Add action owns this disposable row immediately.
            // Merely typing with no selection cannot create a saved pin.
            self.electrical.pins.insert(
                self.electrical.pin_id.unwrap().to_string(),
                json!({"name":"","electrical_type":""}),
            );
        }
        for (key, target) in [("pin_name", "name"), ("pin_type", "electrical_type")] {
            let value = raw
                .as_ref()
                .and_then(|r| {
                    r.get(target).and_then(Value::as_str).or_else(|| {
                        (target == "electrical_type")
                            .then(|| r.get("direction").and_then(Value::as_str))
                            .flatten()
                    })
                })
                .unwrap_or("");
            self.fields
                .iter_mut()
                .find(|f| f.key == key)
                .unwrap()
                .set_text(value.into());
        }
        self.preview = None;
        self.refusal = None;
        Ok(())
    }

    pub fn switch_anchor(&mut self, index: Option<usize>, remove: bool) -> Result<(), String> {
        if !self.create && (remove || index.is_none()) {
            return Err("Saved edits preserve anchor identities and pin links".into());
        }
        let selected = index
            .and_then(|i| self.electrical.anchors.keys().nth(i))
            .cloned();
        if remove {
            if let Some(id) = self.electrical.anchor_id {
                self.electrical.anchors.remove(&id.to_string());
            }
        } else {
            self.electrical.anchors = self.electrical_anchors()?;
        }
        let raw = selected
            .as_ref()
            .and_then(|id| self.electrical.anchors.get(id))
            .cloned();
        self.electrical.anchor_id = selected.and_then(|id| id.parse().ok());
        for (key, pointer) in [
            ("anchor_pin", "/pin"),
            ("anchor_x", "/position/x"),
            ("anchor_y", "/position/y"),
            ("anchor_orientation", "/style/orientation"),
            ("anchor_length", "/style/length_nm"),
        ] {
            let value = raw.as_ref().and_then(|r| {
                let flat = match key {
                    "anchor_orientation" => r.get("orientation"),
                    "anchor_length" => r.get("length_nm"),
                    _ => None,
                };
                flat.filter(|v| !v.is_null()).or_else(|| r.pointer(pointer))
            });
            let Some(field) = self.fields.iter_mut().find(|f| f.key == key) else {
                continue;
            };
            field.set_text(
                value
                    .filter(|v| !v.is_null())
                    .map_or_else(String::new, |v| {
                        v.as_str()
                            .map(str::to_owned)
                            .unwrap_or_else(|| v.to_string())
                    }),
            );
        }
        self.preview = None;
        self.refusal = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod add_pin_tests;
