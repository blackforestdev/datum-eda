//! Disposable Part metadata and compatible optional default source choices.
use super::*;
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PartDraft {
    footprints: Vec<(Uuid, Uuid, String)>,
    maps: Vec<PartMapChoice>,
    entities: BTreeSet<Uuid>,
    packages: BTreeSet<Uuid>,
    pads: BTreeMap<Uuid, BTreeSet<Uuid>>,
    pub original: Option<Value>,
}
#[derive(Debug, Clone, PartialEq)]
struct PartMapChoice {
    id: Uuid,
    footprint: Option<Uuid>,
    package: Option<Uuid>,
    pads: BTreeSet<Uuid>,
    label: String,
    entities: BTreeSet<Uuid>,
}
fn map_entities(browser: &LibraryBrowser, raw: &Value) -> BTreeSet<Uuid> {
    let rows = match raw.get("mappings") {
        None => Vec::new(),
        Some(Value::Object(rows)) => {
            let Ok(rows) = rows
                .values()
                .cloned()
                .map(serde_json::from_value::<eda_engine::pool::PadMapEntry>)
                .collect::<Result<Vec<_>, _>>()
            else {
                return BTreeSet::new();
            };
            rows
        }
        _ => return BTreeSet::new(),
    };
    unique(browser, "entities")
        .into_iter()
        .filter_map(|(id, _)| {
            let id = id.parse::<Uuid>().ok()?;
            let source = browser
                .entries
                .iter()
                .find(|e| e.kind == "entities" && e.id == id)?;
            let entity: eda_engine::pool::Entity =
                serde_json::from_value(source.raw.clone()).ok()?;
            rows.iter()
                .all(|row| {
                    let Some(gate) = entity.gates.get(&row.gate) else {
                        return false;
                    };
                    let units: Vec<_> = browser
                        .entries
                        .iter()
                        .filter(|e| e.kind == "units" && e.id == gate.unit)
                        .collect();
                    let [unit] = units.as_slice() else {
                        return false;
                    };
                    unit.warning.is_none()
                        && serde_json::from_value::<eda_engine::pool::Unit>(unit.raw.clone())
                            .is_ok_and(|u| u.pins.contains_key(&row.pin))
                })
                .then_some(id)
        })
        .collect()
}
fn unique(browser: &LibraryBrowser, kind: &str) -> Vec<(String, String)> {
    named(browser, kind)
        .into_iter()
        .filter(|(id, _)| {
            browser
                .entries
                .iter()
                .filter(|e| e.kind == kind && e.id.to_string() == *id)
                .count()
                == 1
        })
        .collect()
}
impl LibraryDraft {
    pub(super) fn part_fields(
        &mut self,
        browser: &LibraryBrowser,
        raw: &Value,
    ) -> Result<(), String> {
        if !self.create {
            let _: eda_engine::pool::Part = serde_json::from_value(raw.clone())
                .map_err(|_| "Saved Part data is unavailable")?;
        }
        for (key, label) in [
            ("name", "Part name (required if procurement unspecified)"),
            ("manufacturer", "Manufacturer"),
            ("mpn", "MPN"),
            ("value", "Value"),
            ("description", "Description"),
            ("datasheet", "Datasheet reference"),
            ("tags", "Tags (comma separated)"),
        ] {
            let value = if key == "tags" {
                match raw.get(key) {
                    Some(value) => Some(json!(
                        serde_json::from_value::<Vec<String>>(value.clone())
                            .map_err(|_| "Saved Part tags are unavailable")?
                            .join(", ")
                    )),
                    None => None,
                }
            } else {
                raw.get(key).cloned()
            };
            self.field(key, label, value.as_ref(), vec![]);
        }
        if !self.create {
            self.part.original = Some(raw.clone());
            let choices = unique(browser, "entities");
            self.part.entities = choices
                .iter()
                .filter_map(|(id, _)| id.parse().ok())
                .collect();
            let declared = raw["entity"]
                .as_str()
                .and_then(|s| s.parse::<Uuid>().ok())
                .map(|id| json!(id));
            self.field("part_entity", "Named Entity", declared.as_ref(), choices);
            let field = self.fields.last_mut().unwrap();
            if !field.choices.iter().any(|(id, _)| *id == field.text) {
                field.choices.push((
                    field.text.clone(),
                    format!("Unavailable / ambiguous Entity: {}", field.text),
                ));
            }
        } else {
            self.field(
                "part_entity",
                "Named Entity",
                None,
                unique(browser, "entities"),
            );
            self.field(
                "part_package",
                "Named Package",
                None,
                unique(browser, "packages"),
            );
        }
        if !self.create {
            let choices = unique(browser, "packages");
            self.part.packages = choices
                .iter()
                .filter_map(|(id, _)| id.parse().ok())
                .collect();
            let declared = raw["package"]
                .as_str()
                .and_then(|s| s.parse::<Uuid>().ok())
                .map(|id| json!(id));
            self.field("part_package", "Named Package", declared.as_ref(), choices);
            let field = self.fields.last_mut().unwrap();
            if !field.choices.iter().any(|(id, _)| *id == field.text) {
                field.choices.push((
                    field.text.clone(),
                    format!("Unavailable / ambiguous Package: {}", field.text),
                ));
            }
        }
        self.part.pads = browser
            .entries
            .iter()
            .filter(|e| matches!(e.kind.as_str(), "packages" | "footprints") && e.warning.is_none())
            .map(|e| {
                (
                    e.id,
                    e.raw
                        .get("pads")
                        .and_then(Value::as_object)
                        .into_iter()
                        .flat_map(|p| p.keys())
                        .filter_map(|id| id.parse().ok())
                        .collect(),
                )
            })
            .collect();
        let footprint = raw
            .get("default_footprint")
            .and_then(Value::as_str)
            .and_then(|s| s.parse::<Uuid>().ok())
            .map(|id| json!(id));
        self.field(
            "part_footprint",
            "Optional default Footprint",
            footprint.as_ref(),
            vec![],
        );
        self.part.footprints = unique(browser, "footprints")
            .into_iter()
            .filter_map(|(id, label)| {
                let id: Uuid = id.parse().ok()?;
                let raw = &browser
                    .entries
                    .iter()
                    .find(|e| e.id == id && e.kind == "footprints")?
                    .raw;
                Some((id, raw.get("package")?.as_str()?.parse().ok()?, label))
            })
            .collect();
        self.refresh_part_footprints();
        if !self.create {
            let declared = raw
                .get("default_pin_pad_map")
                .and_then(Value::as_str)
                .and_then(|s| s.parse::<Uuid>().ok())
                .map(|id| json!(id));
            self.field(
                "part_map",
                "Optional default PinPadMap",
                declared.as_ref(),
                vec![],
            );
            self.part.maps = unique(browser, "pin_pad_maps")
                .into_iter()
                .filter_map(|(id, label)| {
                    let id: Uuid = id.parse().ok()?;
                    let raw = &browser
                        .entries
                        .iter()
                        .find(|e| e.kind == "pin_pad_maps" && e.id == id)?
                        .raw;
                    if raw.get("part")?.as_str()?.parse::<Uuid>().ok() != Some(self.id) {
                        return None;
                    }
                    let footprint = match raw.get("footprint") {
                        None | Some(Value::Null) => None,
                        Some(v) => Some(v.as_str()?.parse::<Uuid>().ok()?),
                    };
                    let package = if let Some(footprint) = footprint {
                        Some(
                            self.part
                                .footprints
                                .iter()
                                .find(|(id, _, _)| *id == footprint)?
                                .1,
                        )
                    } else {
                        None
                    };
                    Some(PartMapChoice {
                        id,
                        footprint,
                        package,
                        pads: raw
                            .get("mappings")
                            .and_then(Value::as_object)
                            .into_iter()
                            .flat_map(|rows| rows.keys())
                            .filter_map(|id| id.parse().ok())
                            .collect(),
                        label,
                        entities: map_entities(browser, raw),
                    })
                })
                .collect();
            self.refresh_part_maps();
        }
        Ok(())
    }
    fn part_package(&self) -> Option<Uuid> {
        self.value("part_package").parse::<Uuid>().ok()
    }
    fn map_package_compatible(&self, map: &PartMapChoice) -> bool {
        let package = self.part_package();
        if map.package.is_some() && map.package != package {
            return false;
        }
        if !self.dirty.contains("part_package") {
            return true;
        }
        let source = map
            .footprint
            .or_else(|| self.value("part_footprint").parse().ok())
            .or(package);
        map.pads.is_empty()
            || source
                .and_then(|id| self.part.pads.get(&id))
                .is_some_and(|pads| map.pads.is_subset(pads))
    }

    pub(super) fn refresh_part_footprints(&mut self) {
        let package = self.part_package();
        if let Some(f) = self.fields.iter_mut().find(|f| f.key == "part_footprint") {
            f.choices = vec![(String::new(), "No default Footprint".into())];
            f.choices.extend(
                self.part
                    .footprints
                    .iter()
                    .filter(|(_, p, _)| Some(*p) == package)
                    .map(|(id, _, label)| (id.to_string(), label.clone())),
            );
            if !f.choices.iter().any(|(id, _)| id == &f.text) {
                if self.create {
                    f.text.clear();
                } else {
                    f.choices.push((
                        f.text.clone(),
                        format!("Unavailable / ambiguous Footprint: {}", f.text),
                    ));
                }
            }
        }
    }
    pub(super) fn refresh_part_maps(&mut self) {
        let footprint = self.value("part_footprint").parse::<Uuid>().ok();
        let compatible: BTreeSet<_> = self
            .part
            .maps
            .iter()
            .filter(|m| self.map_package_compatible(m))
            .map(|m| m.id)
            .collect();
        let entity = self.value("part_entity").parse::<Uuid>().ok();
        let original_entity = self
            .part
            .original
            .as_ref()
            .and_then(|p| p["entity"].as_str()?.parse::<Uuid>().ok());
        if let Some(field) = self.fields.iter_mut().find(|f| f.key == "part_map") {
            field.choices = vec![(String::new(), "No default PinPadMap".into())];
            field.choices.extend(
                self.part
                    .maps
                    .iter()
                    .filter(|m| {
                        compatible.contains(&m.id)
                            && (entity.is_none()
                                || entity == original_entity
                                || entity.is_some_and(|e| m.entities.contains(&e)))
                            && (m.footprint.is_none()
                                || footprint.is_none()
                                || m.footprint == footprint)
                    })
                    .map(|m| (m.id.to_string(), m.label.clone())),
            );
            if !field.choices.iter().any(|(id, _)| id == &field.text) {
                field.choices.push((
                    field.text.clone(),
                    format!("Unavailable / incompatible PinPadMap: {}", field.text),
                ));
            }
        }
    }
    pub(super) fn part_request(&self, fields: &mut Map<String, Value>) -> Result<(), String> {
        if !self.create {
            if self.dirty.contains("part_entity") {
                let entity = self.uuid("part_entity")?;
                if !self.part.entities.contains(&entity) {
                    return Err("Choose an available, unique named Entity".into());
                }
                fields.insert("entity".into(), json!(entity));
            }
            if self.dirty.contains("part_package") {
                let package = self.uuid("part_package")?;
                if !self.part.packages.contains(&package) {
                    return Err("Choose an available, unique named Package".into());
                }
                self.part_footprint()?;
                fields.insert("package".into(), json!(package));
            }
            for key in [
                "name",
                "manufacturer",
                "mpn",
                "value",
                "description",
                "datasheet",
                "tags",
            ] {
                if !self.dirty.contains(key) {
                    continue;
                }
                let value = self.value(key);
                let value = if key == "tags" {
                    json!(
                        value
                            .split(',')
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .collect::<BTreeSet<_>>()
                    )
                } else {
                    json!(value)
                };
                fields.insert(key.into(), value);
            }
            if self.dirty.contains("part_footprint") {
                fields.insert("default_footprint".into(), json!(self.part_footprint()?));
            }
            if self.dirty.contains("part_map")
                || self.dirty.contains("part_footprint")
                || self.dirty.contains("part_entity")
                || self.dirty.contains("part_package")
            {
                let map = self.part_map()?;
                if self.dirty.contains("part_map") {
                    fields.insert("default_pin_pad_map".into(), json!(map));
                }
            }
            let mut identity = self.part.original.clone().unwrap_or_else(|| json!({}));
            for (key, value) in fields.iter() {
                identity[key] = value.clone();
            }
            eda_engine::api::native_write::library_definition::validate_part_identity(&identity)
                .map_err(|error| error.to_string())?;
            return Ok(());
        }
        for key in [
            "name",
            "manufacturer",
            "mpn",
            "value",
            "description",
            "datasheet",
        ] {
            fields.insert(key.into(), json!(self.value(key)));
        }
        eda_engine::api::native_write::library_definition::validate_part_identity(&Value::Object(
            fields.clone(),
        ))
        .map_err(|error| error.to_string())?;
        for (field, target, label) in [
            ("part_entity", "entity", "Entity"),
            ("part_package", "package", "Package"),
        ] {
            let id = self
                .uuid(field)
                .map_err(|_| format!("Choose an existing named {label}"))?;
            if !self
                .fields
                .iter()
                .find(|f| f.key == field)
                .unwrap()
                .choices
                .iter()
                .any(|(key, _)| *key == id.to_string())
            {
                return Err(format!("{label} is unavailable or ambiguous"));
            }
            fields.insert(target.into(), json!(id));
        }
        fields.insert("default_footprint".into(), json!(self.part_footprint()?));
        let tags: BTreeSet<_> = self
            .value("tags")
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        fields.insert("tags".into(), json!(tags));
        Ok(())
    }
    fn part_footprint(&self) -> Result<Option<Uuid>, String> {
        let footprint = if self.value("part_footprint").is_empty() {
            None
        } else {
            let id = self.uuid("part_footprint")?;
            let package = self.part_package();
            if !self
                .part
                .footprints
                .iter()
                .any(|(f, p, _)| *f == id && Some(*p) == package)
            {
                return Err("Choose a compatible Footprint or no default".into());
            }
            Some(id)
        };
        Ok(footprint)
    }
    fn part_map(&self) -> Result<Option<Uuid>, String> {
        if self.value("part_map").is_empty() {
            return Ok(None);
        }
        let id = self.uuid("part_map")?;
        let footprint = self.part_footprint()?;
        let entity = self.value("part_entity").parse::<Uuid>().ok();
        let original_entity = self
            .part
            .original
            .as_ref()
            .and_then(|p| p["entity"].as_str()?.parse::<Uuid>().ok());
        if !self.part.maps.iter().any(|m| {
            m.id == id
                && (entity == original_entity || entity.is_some_and(|e| m.entities.contains(&e)))
                && self.map_package_compatible(m)
                && (m.footprint.is_none() || footprint.is_none() || m.footprint == footprint)
        }) {
            return Err("Choose a compatible named PinPadMap for this Part or no default".into());
        }
        Ok(Some(id))
    }
}
#[cfg(test)]
mod tests;

#[cfg(test)]
mod edit_tests;
