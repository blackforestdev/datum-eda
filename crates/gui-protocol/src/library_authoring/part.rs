//! Disposable explicit Part metadata and compatible optional Footprint choices.
use super::*;
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PartDraft {
    footprints: Vec<(Uuid, Uuid, String)>,
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
    pub(super) fn part_fields(&mut self, browser: &LibraryBrowser) {
        for (key, label) in [
            ("manufacturer", "Manufacturer"),
            ("mpn", "MPN"),
            ("value", "Value"),
            ("description", "Description"),
            ("datasheet", "Datasheet reference"),
            ("tags", "Tags (comma separated)"),
        ] {
            self.field(key, label, None, vec![]);
        }
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
        self.field("part_footprint", "Optional default Footprint", None, vec![]);
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
    }
    pub(super) fn refresh_part_footprints(&mut self) {
        let package = self.value("part_package").parse::<Uuid>().ok();
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
                f.text.clear();
            }
        }
    }
    pub(super) fn part_request(&self, fields: &mut Map<String, Value>) -> Result<(), String> {
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
        let footprint = if self.value("part_footprint").is_empty() {
            None
        } else {
            let id = self.uuid("part_footprint")?;
            if !self
                .part
                .footprints
                .iter()
                .any(|(f, p, _)| *f == id && p.to_string() == self.value("part_package"))
            {
                return Err("Choose a compatible Footprint or no default".into());
            }
            Some(id)
        };
        fields.insert("default_footprint".into(), json!(footprint));
        let tags: BTreeSet<_> = self
            .value("tags")
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        fields.insert("tags".into(), json!(tags));
        Ok(())
    }
}
#[cfg(test)]
mod tests;
