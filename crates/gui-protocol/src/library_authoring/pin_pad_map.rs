//! Disposable named pad -> gate-qualified pin intent, never a second model.
use super::*;
#[derive(Debug, Clone, PartialEq)]
pub struct MappingName {
    pub id: Option<Uuid>,
    pub label: String,
}
#[derive(Debug, Clone, PartialEq)]
pub struct MappingGate {
    pub id: Uuid,
    pub label: String,
    pub pins: Vec<MappingName>,
}
#[derive(Debug, Clone, PartialEq)]
struct MappingPart {
    id: Uuid,
    label: String,
    package: Uuid,
    gates: Vec<MappingGate>,
    sources_valid: bool,
    default_footprint: Option<Uuid>,
    default_map: Value,
}
#[derive(Debug, Clone, PartialEq)]
struct MappingFootprint {
    id: Uuid,
    label: String,
    package: Uuid,
    pads: Vec<MappingName>,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PinPadMapDraft {
    pub original: Option<Value>,
    pub mappings: Map<String, Value>,
    pub pads: Vec<MappingName>,
    pub gates: Vec<MappingGate>,
    parts: Vec<MappingPart>,
    footprints: Vec<MappingFootprint>,
}
fn entries<'a>(b: &'a LibraryBrowser, kind: &str) -> Vec<&'a LibraryEntry> {
    b.entries
        .iter()
        .filter(|e| {
            e.kind == kind
                && e.warning.is_none()
                && b.entries
                    .iter()
                    .filter(|other| other.kind == kind && other.id == e.id)
                    .count()
                    == 1
        })
        .collect()
}
fn linked<'a>(b: &'a LibraryBrowser, kind: &str, id: Uuid) -> Option<&'a LibraryEntry> {
    entries(b, kind).into_iter().find(|e| e.id == id)
}
fn uuid(raw: &Value, key: &str) -> Option<Uuid> {
    raw.get(key)?.as_str()?.parse().ok()
}
fn names(raw: &Value, field: &str) -> Vec<MappingName> {
    let Some(rows) = raw.get(field).and_then(Value::as_object) else {
        return vec![];
    };
    rows.iter()
        .map(|(key, row)| {
            let id = key.parse::<Uuid>().ok();
            let valid = id.is_some_and(|id| {
                uuid(row, "uuid") == Some(id)
                    && (!matches!(field, "gates" | "pins") || *key == id.to_string())
                    && rows
                        .keys()
                        .filter(|key| key.parse::<Uuid>().ok() == Some(id))
                        .count()
                        == 1
            });
            let name = row
                .get("name")
                .and_then(Value::as_str)
                .filter(|n| !n.is_empty())
                .unwrap_or("Name unavailable");
            let duplicate = rows
                .values()
                .filter(|r| {
                    r.get("name")
                        .and_then(Value::as_str)
                        .filter(|n| !n.is_empty())
                        .unwrap_or("Name unavailable")
                        == name
                })
                .count()
                > 1;
            MappingName {
                id: id.filter(|_| valid),
                label: if duplicate {
                    format!("{name} · {key}")
                } else {
                    name.into()
                },
            }
        })
        .collect()
}
impl PinPadMapDraft {
    pub fn part_label(&self, id: &str) -> &str {
        self.parts
            .iter()
            .find(|p| p.id.to_string() == id)
            .map_or("Unavailable Part", |p| p.label.as_str())
    }
    pub fn footprint_label(&self, id: &str) -> &str {
        self.footprints
            .iter()
            .find(|p| p.id.to_string() == id)
            .map_or("Unavailable Footprint", |p| p.label.as_str())
    }
    pub fn gate_label(&self, id: &str) -> &str {
        self.gates
            .iter()
            .find(|g| g.id.to_string() == id)
            .map_or("Unavailable gate", |g| g.label.as_str())
    }
    pub fn pin_label(&self, gate: &str, pin: &str) -> &str {
        self.gates
            .iter()
            .find(|g| g.id.to_string() == gate)
            .and_then(|g| {
                g.pins
                    .iter()
                    .find(|p| p.id.is_some_and(|id| id.to_string() == pin))
            })
            .map_or("Unavailable pin", |p| p.label.as_str())
    }
    pub fn pad_label(&self, id: &str) -> &str {
        self.pads
            .iter()
            .find(|p| p.id.is_some_and(|v| v.to_string() == id))
            .map_or("Unavailable pad", |p| p.label.as_str())
    }
    pub fn previous_default(&self, part: &str) -> String {
        self.parts
            .iter()
            .find(|p| p.id.to_string() == part)
            .map_or("Unavailable".into(), |p| {
                if p.default_map.is_null() {
                    "None".into()
                } else {
                    p.default_map.to_string()
                }
            })
    }
    pub fn matches_explicit_default(&self, part: &str, footprint: &str) -> bool {
        self.parts
            .iter()
            .find(|p| p.id.to_string() == part)
            .is_some_and(|p| {
                p.default_footprint
                    .is_some_and(|id| id.to_string() == footprint)
            })
    }
}
impl LibraryDraft {
    pub(super) fn pin_pad_map_fields(&mut self, b: &LibraryBrowser) {
        self.pin_pad_map.parts = entries(b, "parts")
            .into_iter()
            .filter_map(|p| {
                let package = uuid(&p.raw, "package")?;
                linked(b, "packages", package)?;
                let entity = linked(b, "entities", uuid(&p.raw, "entity")?)?;
                let gates = entity.raw.get("gates")?.as_object()?;
                let expected_gates = gates.len();
                let gates: Vec<_> = names(&entity.raw, "gates")
                    .into_iter()
                    .filter_map(|g| {
                        let id = g.id?;
                        let raw = gates
                            .iter()
                            .find(|(key, _)| key.parse::<Uuid>().ok() == Some(id))?
                            .1;
                        let unit = linked(b, "units", uuid(raw, "unit")?)?;
                        // Designator captions are optional projection, never admission.
                        let anchors = uuid(raw, "symbol")
                            .and_then(|id| linked(b, "symbols", id))
                            .filter(|s| uuid(&s.raw, "unit") == Some(unit.id))
                            .and_then(|s| s.raw.get("pin_anchors"))
                            .and_then(Value::as_array)
                            .map(Vec::as_slice)
                            .unwrap_or(&[]);
                        let pins = names(&unit.raw, "pins")
                            .into_iter()
                            .map(|mut pin| {
                                let numbers: Vec<_> = anchors
                                    .iter()
                                    .filter(|a| uuid(a, "pin") == pin.id)
                                    .filter_map(|a| a.get("number").and_then(Value::as_str))
                                    .filter(|number| !number.trim().is_empty())
                                    .collect();
                                if let [number] = numbers.as_slice() {
                                    if anchors
                                        .iter()
                                        .filter(|a| {
                                            a.get("number").and_then(Value::as_str) == Some(*number)
                                        })
                                        .count()
                                        == 1
                                    {
                                        pin.label = format!("Pin {number}");
                                    }
                                }
                                pin
                            })
                            .collect();
                        Some(MappingGate {
                            id,
                            label: format!("{} · {}", g.label, unit.name),
                            pins,
                        })
                    })
                    .collect();
                Some(MappingPart {
                    id: p.id,
                    label: format!("{} · {}", p.name, p.source_path),
                    package,
                    sources_valid: gates.len() == expected_gates,
                    gates,
                    default_footprint: uuid(&p.raw, "default_footprint"),
                    default_map: p
                        .raw
                        .get("default_pin_pad_map")
                        .cloned()
                        .unwrap_or(Value::Null),
                })
            })
            .collect();
        self.pin_pad_map.footprints = entries(b, "footprints")
            .into_iter()
            .filter_map(|f| {
                Some(MappingFootprint {
                    id: f.id,
                    label: format!("{} · {}", f.name, f.source_path),
                    package: uuid(&f.raw, "package")?,
                    pads: names(&f.raw, "pads"),
                })
            })
            .collect();
        let parts = self
            .pin_pad_map
            .parts
            .iter()
            .map(|p| (p.id.to_string(), p.label.clone()))
            .collect();
        if self.create {
            self.field("map_part", "Named Part", None, parts);
            self.field("map_footprint", "Compatible named Footprint", None, vec![]);
        }
        self.field("map_pad", "Named pad", None, vec![]);
        self.field("map_gate", "Gate / Unit", None, vec![]);
        self.field("map_pin", "Logical pin", None, vec![]);
        if self.create {
            self.field(
                "map_default",
                "Default assignment",
                Some(&json!("false")),
                choices(&[
                    ("false", "Create map only · do not change Part default"),
                    (
                        "true",
                        "Explicitly set this map as the eligible Part default",
                    ),
                ]),
            );
        }
    }
    pub fn map_source(&self, key: &str) -> &str {
        if let Some(raw) = &self.pin_pad_map.original {
            let name = match key {
                "map_part" => "part",
                "map_footprint" => "footprint",
                _ => return "",
            };
            return raw.get(name).and_then(Value::as_str).unwrap_or("");
        }
        self.value(key)
    }
    pub(super) fn hydrate_saved_map(&mut self, raw: &Value) -> Result<(), String> {
        let _: eda_engine::pool::PinPadMap = serde_json::from_value(raw.clone())
            .map_err(|_| "Saved PinPadMap data is unavailable")?;
        let part = uuid(raw, "part").ok_or("Saved Part source is unavailable")?;
        let footprint = uuid(raw, "footprint")
            .ok_or("Saved mapping has no declared Footprint; available for inspection only")?;
        let p = self
            .pin_pad_map
            .parts
            .iter()
            .find(|p| p.id == part)
            .ok_or("Saved Part source is unavailable or ambiguous")?;
        if !self
            .pin_pad_map
            .footprints
            .iter()
            .any(|f| f.id == footprint && f.package == p.package)
        {
            return Err("Saved Footprint source is unavailable or incompatible".into());
        }
        if !p.sources_valid
            || p.gates
                .iter()
                .any(|g| g.pins.iter().any(|pin| pin.id.is_none()))
        {
            return Err("Saved gate pin source is unavailable or ambiguous".into());
        }
        self.pin_pad_map.original = Some(raw.clone());
        self.refresh_map_sources();
        if self.pin_pad_map.pads.iter().any(|p| p.id.is_none()) {
            return Err("Saved pad source is unavailable or ambiguous".into());
        }
        let rows = raw
            .get("mappings")
            .and_then(Value::as_object)
            .ok_or("Saved mapping rows are unavailable")?;
        let mut mappings = Map::new();
        for (key, cell) in rows {
            let pad = key
                .parse::<Uuid>()
                .map_err(|_| "Saved pad identity is invalid")?;
            let gate = uuid(cell, "gate").ok_or("Saved gate identity is invalid")?;
            let pin = uuid(cell, "pin").ok_or("Saved pin identity is invalid")?;
            if !self.pin_pad_map.pads.iter().any(|p| p.id == Some(pad))
                || !self
                    .pin_pad_map
                    .gates
                    .iter()
                    .any(|g| g.id == gate && g.pins.iter().any(|p| p.id == Some(pin)))
                || mappings
                    .insert(pad.to_string(), json!({"gate":gate,"pin":pin}))
                    .is_some()
            {
                return Err("Saved mapping source is unavailable or ambiguous".into());
            }
        }
        self.pin_pad_map.mappings = mappings;
        self.load_map_pad();
        self.pin_pad_mappings()?;
        Ok(())
    }
    fn map_text(&mut self, key: &str, text: String) {
        if let Some(field) = self.fields.iter_mut().find(|f| f.key == key) {
            field.set_text(text);
        }
    }
    fn map_choices(&mut self, key: &str, options: Vec<(String, String)>) {
        let Some(f) = self.fields.iter_mut().find(|f| f.key == key) else {
            return;
        };
        f.choices = options;
        if !f.choices.iter().any(|(key, _)| key == &f.text) {
            f.text.clear();
        }
    }
    fn refresh_map_pins(&mut self) {
        let pins = self
            .pin_pad_map
            .gates
            .iter()
            .find(|g| g.id.to_string() == self.value("map_gate"))
            .map_or(vec![], |g| g.pins.clone());
        self.map_choices(
            "map_pin",
            pins.into_iter()
                .filter_map(|p| Some((p.id?.to_string(), p.label)))
                .collect(),
        );
    }
    pub(super) fn refresh_map_sources(&mut self) {
        let part = self
            .pin_pad_map
            .parts
            .iter()
            .find(|p| p.id.to_string() == self.map_source("map_part"))
            .cloned();
        self.pin_pad_map.gates = part.as_ref().map_or(vec![], |p| p.gates.clone());
        let options = self
            .pin_pad_map
            .footprints
            .iter()
            .filter(|f| Some(f.package) == part.as_ref().map(|p| p.package))
            .map(|f| (f.id.to_string(), f.label.clone()))
            .collect();
        self.map_choices("map_footprint", options);
        self.pin_pad_map.pads = self
            .pin_pad_map
            .footprints
            .iter()
            .find(|f| f.id.to_string() == self.map_source("map_footprint"))
            .map_or(vec![], |f| f.pads.clone());
        self.map_choices(
            "map_pad",
            self.pin_pad_map
                .pads
                .iter()
                .filter_map(|p| Some((p.id?.to_string(), p.label.clone())))
                .collect(),
        );
        self.map_choices(
            "map_gate",
            self.pin_pad_map
                .gates
                .iter()
                .map(|g| (g.id.to_string(), g.label.clone()))
                .collect(),
        );
        let pads = &self.pin_pad_map.pads;
        let gates = &self.pin_pad_map.gates;
        self.pin_pad_map.mappings.retain(|pad, entry| {
            pads.iter()
                .any(|p| p.id.is_some_and(|id| id.to_string() == *pad))
                && gates.iter().any(|g| {
                    Some(g.id) == uuid(entry, "gate")
                        && uuid(entry, "pin")
                            .is_some_and(|pin| g.pins.iter().any(|p| p.id == Some(pin)))
                })
        });
        self.load_map_pad();
        self.map_text("map_default", "false".into());
    }
    fn load_map_pad(&mut self) {
        let raw = self
            .pin_pad_map
            .mappings
            .get(self.value("map_pad"))
            .cloned()
            .unwrap_or(Value::Null);
        self.map_text(
            "map_gate",
            raw.get("gate").and_then(Value::as_str).unwrap_or("").into(),
        );
        self.refresh_map_pins();
        self.map_text(
            "map_pin",
            raw.get("pin").and_then(Value::as_str).unwrap_or("").into(),
        );
    }
    pub fn pin_pad_mappings(&self) -> Result<Map<String, Value>, String> {
        let mut rows = self.pin_pad_map.mappings.clone();
        let pad = self.value("map_pad");
        let gate = self.value("map_gate");
        let pin = self.value("map_pin");
        if !gate.is_empty() || !pin.is_empty() {
            let pad = self.uuid("map_pad").map_err(|_| "Choose a named pad")?;
            let gate = self.uuid("map_gate").map_err(|_| "Choose a named gate")?;
            let pin = self
                .uuid("map_pin")
                .map_err(|_| "Choose a logical pin or unmap the selected pad")?;
            if !self.pin_pad_map.pads.iter().any(|p| p.id == Some(pad))
                || !self
                    .pin_pad_map
                    .gates
                    .iter()
                    .any(|g| g.id == gate && g.pins.iter().any(|p| p.id == Some(pin)))
            {
                return Err("Mapping source is unavailable or incompatible".into());
            }
            rows.insert(pad.to_string(), json!({"gate":gate,"pin":pin}));
        } else if !pad.is_empty() {
            rows.remove(pad);
        }
        let mut used = BTreeSet::new();
        for value in rows.values() {
            if !used.insert((uuid(value, "gate"), uuid(value, "pin"))) {
                return Err("A gate-qualified pin is already mapped to another pad".into());
            }
        }
        Ok(rows)
    }
    pub(super) fn before_map_change(&mut self, key: &str, text: &str) -> Result<bool, String> {
        if self.value(key) == text {
            return Ok(true);
        }
        if key == "map_pad" {
            if !self.value("map_pad").is_empty() {
                self.pin_pad_map.mappings = self.pin_pad_mappings()?;
            }
            if !self
                .pin_pad_map
                .pads
                .iter()
                .any(|p| p.id.is_some_and(|id| id.to_string() == text))
            {
                return Err("Pad source is unavailable".into());
            }
            self.map_text("map_pad", text.into());
            self.load_map_pad();
            self.preview = None;
            self.refusal = None;
            return Ok(true);
        }
        if matches!(key, "map_part" | "map_footprint") {
            if let Ok(rows) = self.pin_pad_mappings() {
                self.pin_pad_map.mappings = rows;
            } else {
                let pad = self.value("map_pad").to_string();
                self.pin_pad_map.mappings.remove(&pad);
            }
        }
        Ok(false)
    }
    pub fn select_mapping_pad(&mut self, index: Option<usize>, remove: bool) -> Result<(), String> {
        if self.kind != "pin_pad_maps" {
            return Err("Mapping rows require a PinPadMap draft".into());
        }
        if remove {
            let pad = self.value("map_pad").to_string();
            if pad.is_empty() {
                return Err("Choose a pad to unmap".into());
            }
            self.pin_pad_map.mappings.remove(&pad);
            self.map_text("map_gate", String::new());
            self.map_text("map_pin", String::new());
        } else {
            let id = index
                .and_then(|i| self.pin_pad_map.pads.get(i))
                .and_then(|p| p.id)
                .ok_or("Pad source is unavailable")?;
            self.before_map_change("map_pad", &id.to_string())?;
        }
        self.preview = None;
        self.refusal = None;
        Ok(())
    }
    pub(super) fn after_map_change(&mut self, key: &str) {
        match key {
            "map_part" | "map_footprint" => self.refresh_map_sources(),
            "map_gate" => self.refresh_map_pins(),
            _ => {}
        }
    }
    pub(super) fn pin_pad_map_request(
        &self,
        fields: &mut Map<String, Value>,
    ) -> Result<(), String> {
        let part = self
            .map_source("map_part")
            .parse::<Uuid>()
            .map_err(|_| "Choose an existing named Part")?;
        let footprint = self
            .map_source("map_footprint")
            .parse::<Uuid>()
            .map_err(|_| "Choose a compatible named Footprint")?;
        if !self.pin_pad_map.parts.iter().any(|p| {
            p.id == part
                && self
                    .pin_pad_map
                    .footprints
                    .iter()
                    .any(|f| f.id == footprint && f.package == p.package)
        }) {
            return Err("Part or Footprint source is unavailable or incompatible".into());
        }
        let rows = self.pin_pad_mappings()?;
        if rows.is_empty() {
            return Err("Map one or more named pads to gate-qualified pins".into());
        }
        if !self.create {
            fields.insert("mappings".into(), json!(rows));
            return Ok(());
        }
        let assign = match self.value("map_default") {
            "false" => false,
            "true" => true,
            _ => return Err("Choose whether to assign the default".into()),
        };
        fields.extend([
            (String::from("part"), json!(part)),
            ("footprint".into(), json!(footprint)),
            ("mappings".into(), json!(rows)),
            ("assign_default".into(), json!(assign)),
        ]);
        Ok(())
    }
}
#[cfg(test)]
mod tests;
