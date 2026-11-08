//! A saved form carries a disposable, revision-qualified raw basis. Only the
//! supported properties change; source graphics and binding identity stay put.
use super::*;

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
        Ok(())
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
            let pin = pins
                .iter_mut()
                .find(|(key, _)| Uuid::parse_str(key).ok() == Some(id))
                .and_then(|(_, value)| value.as_object_mut())
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
            let pins = Value::Object(self.saved_pins()?);
            if original.get("pins") != Some(&pins) {
                fields.insert("pins".into(), pins);
            }
        } else {
            let anchors = self.saved_anchors()?;
            let mut ordered = original
                .get("pin_anchors")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for anchor in &mut ordered {
                let pin = anchor
                    .get("pin")
                    .and_then(Value::as_str)
                    .ok_or("Saved anchor identity unavailable")?;
                *anchor = anchors
                    .get(pin)
                    .ok_or("Saved anchor identity changed")?
                    .clone();
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
