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
            self.fields.retain(|f| {
                f.key != "unit" && f.key != "anchor_pin" && !f.key.starts_with("rect_")
            });
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
        if self.kind == "symbols" {
            self.hydrate_graphics();
            self.hydrate_symbol_fields(raw)?;
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
                "anchor_pin",
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
        let target = self.uuid("anchor_pin")?;
        let unit = self.electrical_unit().parse::<Uuid>().ok();
        if target != id
            && (target.is_nil()
                || !self
                    .electrical
                    .unit_pins
                    .iter()
                    .find(|(key, _)| key.parse::<Uuid>().ok() == unit)
                    .is_some_and(|(_, pins)| {
                        pins.iter().any(|(key, name)| {
                            key.parse::<Uuid>().ok() == Some(target) && !name.trim().is_empty()
                        })
                    }))
        {
            return Err("Choose an existing named pin of the retained Unit".into());
        }
        if (target != id && self.original_anchor(target))
            || anchors.iter().any(|(key, row)| {
                key.parse::<Uuid>().ok() != Some(id)
                    && row["pin"].as_str().and_then(|s| s.parse::<Uuid>().ok()) == Some(target)
            })
        {
            return Err("This Unit pin already has an anchor; choose an unanchored pin".into());
        }
        let original_pin = self
            .electrical
            .original
            .as_ref()
            .and_then(|raw| raw["pin_anchors"].as_array())
            .and_then(|rows| {
                rows.iter().find(|row| {
                    row["pin"].as_str().and_then(|s| s.parse::<Uuid>().ok()) == Some(id)
                })
            })
            .map(|row| row["pin"].clone())
            .ok_or("Saved anchor identity unavailable")?;
        let anchor = anchors
            .iter_mut()
            .find(|(key, _)| Uuid::parse_str(key).ok() == Some(id))
            .map(|(_, value)| value)
            .ok_or("Select an existing anchor")?;
        // Keep the captured map key as the disposable row identity. Reverting
        // the choice also restores its original raw UUID spelling.
        anchor["pin"] = if target == id {
            original_pin
        } else {
            json!(target)
        };
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
            self.pin_metadata_request(original, fields)?;
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
            self.graphic_request(fields)?;
            self.symbol_field_request(original, fields)?;
            let anchors = self.electrical_anchors()?;
            let mut ordered = original
                .get("pin_anchors")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let mut removals = Vec::new();
            let mut relinks = Map::new();
            for anchor in &mut ordered {
                let pin = anchor
                    .get("pin")
                    .and_then(Value::as_str)
                    .ok_or("Saved anchor identity unavailable")?;
                if let Some(current) = anchors.get(pin) {
                    let old: Uuid = pin.parse().map_err(|_| "Saved anchor identity invalid")?;
                    let next: Uuid = serde_json::from_value(current["pin"].clone())
                        .map_err(|_| "Choose a named Unit pin")?;
                    let mut posed = current.clone();
                    if next != old {
                        relinks.insert(old.to_string(), json!(next));
                    }
                    // Pose admission still receives original identities; the
                    // explicit sparse correction owns only the pin-reference leaf.
                    posed["pin"] = anchor["pin"].clone();
                    *anchor = posed;
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
            if !relinks.is_empty() {
                fields.insert("anchor_relinks".into(), relinks.into());
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
