//! Disposable explicit Part metadata and compatible optional Footprint choices.
use super::*;
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PartDraft {
    footprints: Vec<(Uuid, Uuid, String)>,
    pub original: Option<Value>,
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
        Ok(())
    }
    pub(super) fn refresh_part_footprints(&mut self) {
        let package = if self.create {
            self.value("part_package").parse::<Uuid>().ok()
        } else {
            self.part
                .original
                .as_ref()
                .and_then(|r| r["package"].as_str()?.parse::<Uuid>().ok())
        };
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
    pub(super) fn part_request(&self, fields: &mut Map<String, Value>) -> Result<(), String> {
        if !self.create {
            for key in [
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
                if matches!(key, "manufacturer" | "mpn") && value.trim().is_empty() {
                    return Err("Enter a manufacturer and MPN".into());
                }
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
            return Ok(());
        }
        if self.value("manufacturer").trim().is_empty() || self.value("mpn").trim().is_empty() {
            return Err("Enter a manufacturer and MPN".into());
        }
        for key in ["manufacturer", "mpn", "value", "description", "datasheet"] {
            fields.insert(key.into(), json!(self.value(key)));
        }
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
            let package = if self.create {
                self.value("part_package").parse::<Uuid>().ok()
            } else {
                self.part
                    .original
                    .as_ref()
                    .and_then(|r| r["package"].as_str()?.parse::<Uuid>().ok())
            };
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
}
#[cfg(test)]
mod tests;

#[cfg(test)]
mod edit_tests;
