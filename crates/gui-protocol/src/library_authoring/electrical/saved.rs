//! A saved form carries a disposable, revision-qualified raw basis. Only the
//! supported properties change; other raw graphics and binding identity stay put.
use super::*;

const RECTANGLE_FIELDS: &[(&str, &str)] = &[
    ("rect_min_x", "/Rect/min/x"),
    ("rect_min_y", "/Rect/min/y"),
    ("rect_max_x", "/Rect/max/x"),
    ("rect_max_y", "/Rect/max/y"),
    ("rect_width", "/Rect/width"),
];

impl LibraryDraft {
    pub(in crate::library_authoring) fn hydrate_saved(
        &mut self,
        raw: &Value,
    ) -> Result<(), String> {
        self.electrical.original = Some(raw.clone());
        self.electrical.pin_id = None;
        let mut identities = std::collections::HashSet::new();
        if self.kind == "units" {
            self.electrical.pins = raw
                .get("pins")
                .and_then(Value::as_object)
                .ok_or("Saved Unit pin data is unavailable")?
                .clone();
            for key in self.electrical.pins.keys() {
                let id =
                    Uuid::parse_str(key).map_err(|_| "Saved logical pin identity is invalid")?;
                if !identities.insert(id) {
                    return Err("Saved Unit has ambiguous logical pin identities".into());
                }
            }
        } else {
            self.electrical.unit_label = self
                .fields
                .iter()
                .find(|f| f.key == "unit")
                .and_then(|f| {
                    f.choices.iter().find(|(id, _)| {
                        raw.get("unit").and_then(Value::as_str) == Some(id.as_str())
                    })
                })
                .map(|(_, name)| name.clone());
            let rectangle = self.saved_rectangle_index();
            self.fields.retain(|f| {
                f.key != "unit"
                    && f.key != "anchor_pin"
                    && (!f.key.starts_with("rect_") || rectangle.is_some())
            });
            if let Some(index) = rectangle {
                for &(key, pointer) in RECTANGLE_FIELDS {
                    let text = raw["drawings"][index].pointer(pointer).unwrap().to_string();
                    self.fields
                        .iter_mut()
                        .find(|f| f.key == key)
                        .unwrap()
                        .set_text(text);
                }
            }
            let empty = Vec::new();
            let anchors = match raw.get("pin_anchors") {
                None => &empty,
                Some(value) => value
                    .as_array()
                    .ok_or("Saved Symbol anchor data is unavailable")?,
            };
            for anchor in anchors {
                let pin = anchor
                    .get("pin")
                    .and_then(Value::as_str)
                    .ok_or("Saved anchor pin identity is unavailable")?;
                let id =
                    Uuid::parse_str(pin).map_err(|_| "Saved anchor pin identity is invalid")?;
                if !identities.insert(id) {
                    return Err("Saved Symbol has ambiguous logical pin anchors".into());
                }
                if self
                    .electrical
                    .anchors
                    .insert(pin.into(), anchor.clone())
                    .is_some()
                {
                    return Err(
                        "Saved Symbol has ambiguous repeated pin anchors; inspect its source"
                            .into(),
                    );
                }
            }
        }
        Ok(())
    }

    /// The source index is disposable identity under this draft's captured revision.
    /// Zero/multiple rectangles and unavailable numeric data never select a body.
    pub fn saved_rectangle_index(&self) -> Option<usize> {
        let drawings = self
            .electrical
            .original
            .as_ref()?
            .get("drawings")?
            .as_array()?;
        let mut rectangles = drawings
            .iter()
            .enumerate()
            .filter(|(_, graphic)| graphic.get("Rect").is_some());
        let (index, graphic) = rectangles.next()?;
        if rectangles.next().is_some()
            || RECTANGLE_FIELDS
                .iter()
                .any(|(_, p)| graphic.pointer(p).and_then(Value::as_i64).is_none())
        {
            return None;
        }
        Some(index)
    }

    fn saved_drawings(&self) -> Result<Option<Value>, String> {
        if !RECTANGLE_FIELDS
            .iter()
            .any(|(key, _)| self.dirty.contains(key))
        {
            return Ok(None);
        }
        let index = self
            .saved_rectangle_index()
            .ok_or("Body correction requires exactly one existing rectangle")?;
        let original = &self.electrical.original.as_ref().unwrap()["drawings"];
        let mut drawings = original.clone();
        for &(key, pointer) in RECTANGLE_FIELDS {
            let next = self.length(key)?;
            if drawings[index].pointer(pointer).and_then(Value::as_i64) != Some(next) {
                *drawings[index].pointer_mut(pointer).unwrap() = json!(next);
            }
        }
        let eda_engine::pool::Primitive::Rect { min, max, width } =
            serde_json::from_value(drawings[index].clone())
                .map_err(|_| "Saved rectangle data is unavailable")?
        else {
            return Err("Saved rectangle data is unavailable".into());
        };
        if min.x >= max.x || min.y >= max.y || width <= 0 {
            return Err("Body corners must be ordered and stroke width positive".into());
        }
        Ok((drawings != *original).then_some(drawings))
    }

    /// Named pin labels share the captured source Unit; saved edits never relink it.
    pub fn electrical_unit(&self) -> &str {
        self.electrical
            .original
            .as_ref()
            .and_then(|v| v.get("unit"))
            .and_then(Value::as_str)
            .unwrap_or_else(|| self.value("unit"))
    }

    pub(super) fn saved_pins(&self) -> Result<Map<String, Value>, String> {
        let mut pins = self.electrical.pins.clone();
        if self.electrical.pin_id.is_none()
            && ["pin_name", "pin_type"]
                .iter()
                .any(|k| !self.value(k).is_empty())
        {
            return Err("Select an existing logical pin before editing its metadata".into());
        }
        if let Some(id) = self.electrical.pin_id {
            let key = pin_key(&pins, id)?;
            let pin = pins
                .get_mut(&key)
                .and_then(Value::as_object_mut)
                .ok_or("Select an existing logical pin")?;
            let name = self.value("pin_name");
            if pin.get("name").and_then(Value::as_str) != Some(name) {
                if name.trim().is_empty() {
                    return Err("Enter a pin name".into());
                }
                pin.insert("name".into(), json!(name));
            }
            let original_type = pin
                .get("electrical_type")
                .and_then(Value::as_str)
                .or_else(|| pin.get("direction").and_then(Value::as_str));
            if original_type != Some(self.value("pin_type")) {
                let kind = self.value("pin_type");
                if kind.is_empty() {
                    return Err("Choose the pin electrical type".into());
                }
                pin.insert("direction".into(), json!(kind));
                if pin.contains_key("electrical_type") {
                    pin.insert("electrical_type".into(), json!(kind));
                }
            }
        }
        Ok(pins)
    }

    pub(super) fn saved_anchors(&self) -> Result<Map<String, Value>, String> {
        let mut anchors = self.electrical.anchors.clone();
        let Some(id) = self.electrical.anchor_id else {
            if [
                "anchor_x",
                "anchor_y",
                "anchor_orientation",
                "anchor_length",
            ]
            .iter()
            .any(|k| !self.value(k).is_empty())
            {
                return Err("Select an existing pin anchor before editing its geometry".into());
            }
            return Ok(anchors);
        };
        let anchor = anchors
            .iter_mut()
            .find(|(key, _)| Uuid::parse_str(key).ok() == Some(id))
            .map(|(_, value)| value)
            .ok_or("Select an existing anchor")?;
        for (field, key) in [("anchor_x", "x"), ("anchor_y", "y")] {
            let next = self.length(field)?;
            if anchor
                .pointer(&format!("/position/{key}"))
                .and_then(Value::as_i64)
                != Some(next)
            {
                anchor["position"][key] = json!(next);
            }
        }
        for (field, key) in [
            ("anchor_orientation", "orientation"),
            ("anchor_length", "length_nm"),
        ] {
            let original = anchor
                .get(key)
                .filter(|v| !v.is_null())
                .or_else(|| anchor.get("style").and_then(|s| s.get(key)));
            let text = original
                .filter(|v| !v.is_null())
                .map_or_else(String::new, |v| {
                    v.as_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| v.to_string())
                });
            if self.value(field) == text {
                continue;
            }
            let value = if self.value(field).is_empty() {
                None
            } else if key == "length_nm" {
                Some(json!(self.length(field)?))
            } else {
                Some(json!(self.value(field)))
            };
            let flat = anchor.get(key).is_some();
            if flat {
                if let Some(value) = &value {
                    anchor[key] = value.clone();
                } else {
                    anchor.as_object_mut().unwrap().remove(key);
                }
            }
            if (!flat || anchor.get("style").is_some_and(|v| v.get(key).is_some()))
                && value.is_some()
                && anchor.get("style").is_none_or(Value::is_null)
            {
                anchor["style"] = json!({});
            }
            if let Some(style) = anchor.get_mut("style").and_then(Value::as_object_mut) {
                if !flat || style.contains_key(key) {
                    if let Some(value) = value {
                        style.insert(key.into(), value);
                    } else {
                        style.remove(key);
                    }
                }
            }
        }
        Ok(anchors)
    }

    pub(super) fn saved_request(&self, fields: &mut Map<String, Value>) -> Result<(), String> {
        let original = self
            .electrical
            .original
            .as_ref()
            .ok_or("Saved definition basis unavailable")?;
        if self.kind == "units" {
            let pins = self.electrical_pins()?;
            let old = original["pins"]
                .as_object()
                .ok_or("Saved Unit pins are unavailable")?;
            let mut removals = Vec::new();
            let mut retained = Map::new();
            for (key, original) in old {
                let value = if let Some(current) = pins.get(key) {
                    current.clone()
                } else {
                    let id: Uuid = key
                        .parse()
                        .map_err(|_| "Saved logical pin identity is invalid")?;
                    removals.push(json!(id));
                    // Removal owns only this row. Keep its raw preimage in the
                    // old-metadata payload so other buffered edits compose.
                    original.clone()
                };
                retained.insert(key.clone(), value);
            }
            if !removals.is_empty() {
                fields.insert("pin_removals".into(), removals.into());
            }
            if &retained != old {
                fields.insert("pins".into(), retained.into());
            }
            let additions: Map<_, _> = pins
                .into_iter()
                .filter(|(key, _)| !old.contains_key(key))
                .collect();
            if !additions.is_empty() {
                fields.insert("pin_additions".into(), additions.into());
            }
        } else {
            if let Some(drawings) = self.saved_drawings()? {
                fields.insert("drawings".into(), drawings);
            }
            let anchors = self.electrical_anchors()?;
            let mut ordered = original
                .get("pin_anchors")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let mut removals = Vec::new();
            for anchor in &mut ordered {
                let pin = anchor
                    .get("pin")
                    .and_then(Value::as_str)
                    .ok_or("Saved anchor identity unavailable")?;
                if let Some(current) = anchors.get(pin) {
                    *anchor = current.clone();
                } else {
                    // Keep the original in the old-pose payload: removal discards
                    // this row's pending pose without affecting another row.
                    let id: Uuid = pin.parse().map_err(|_| "Saved anchor identity invalid")?;
                    removals.push(json!(id));
                }
            }
            if !removals.is_empty() {
                fields.insert("anchor_removals".into(), removals.into());
            }
            let additions: Vec<_> = anchors
                .iter()
                .filter(|(key, _)| {
                    key.parse::<Uuid>()
                        .ok()
                        .is_some_and(|id| !self.original_anchor(id))
                })
                .map(|(_, raw)| raw.clone())
                .collect();
            if !additions.is_empty() {
                fields.insert("anchor_additions".into(), additions.into());
            }
            let anchors = Value::Array(ordered);
            if !(original.get("pin_anchors").is_none()
                && anchors.as_array().is_some_and(Vec::is_empty))
                && original.get("pin_anchors") != Some(&anchors)
            {
                fields.insert("pin_anchors".into(), anchors);
            }
        }
        Ok(())
    }
}
