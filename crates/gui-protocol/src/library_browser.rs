//! Named, disposable library projection over the same resolved DesignModel.
use eda_engine::substrate::{DesignModel, SourceShardKind, SourceShardTaxon};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

pub const KINDS: [(&str, &str); 8] = [
    ("units", "Unit"),
    ("symbols", "Symbol"),
    ("entities", "Entity"),
    ("parts", "Part"),
    ("packages", "Package"),
    ("footprints", "Footprint"),
    ("padstacks", "Padstack"),
    ("pin_pad_maps", "PinPadMap"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryControl {
    Kind(usize),
    Search,
    Result(usize),
    Relationship(usize),
    PreviewPad(usize),
    Notice,
    Scroll {
        content: usize,
        height: usize,
        offset: usize,
    },
    ResultsPage(bool),
    New,
    Edit,
    Field(usize),
    Preview,
    Commit,
    Cancel,
    DetailPage(bool),
    Pad(usize),
    AddPad,
    RemovePad,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LibraryEntry {
    pub id: Uuid,
    pub kind: String,
    pub name: String,
    pub source_path: String,
    pub revision: Option<u64>,
    pub raw: Value,
    pub warning: Option<String>,
}
impl LibraryEntry {
    pub fn key(&self) -> String {
        format!("library:{}", self.source_path)
    }
    pub fn editable(&self) -> bool {
        self.warning.is_none()
            && self.source_path == format!("pool/{}/{}.json", self.kind, self.id)
            && matches!(self.kind.as_str(), "footprints" | "padstacks")
    }
    pub fn label(&self) -> &str {
        KINDS
            .iter()
            .find(|k| k.0 == self.kind)
            .map_or("Unavailable", |k| k.1)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LibraryLink {
    pub role: String,
    pub kind: String,
    pub id: Uuid,
    pub targets: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LibraryBrowser {
    pub entries: Vec<LibraryEntry>,
    pub basis: Option<Arc<DesignModel>>,
    pub kind: usize,
    pub search: String,
    pub search_focus: bool,
    pub result_offset: usize,
    pub detail_offset: usize,
    pub draft: Option<crate::library_authoring::LibraryDraft>,
    pub refusal: Option<String>,
    pub focus: Option<LibraryControl>,
    pub scroll_offset: usize,
    pub reveal_focus: bool,
    pub preview_pad: usize,
}

impl LibraryBrowser {
    pub fn process_inspection(
        &self,
        kind: &str,
        raw: &Value,
        index: usize,
    ) -> Result<eda_engine::pool::process::EffectivePadstackProcess, String> {
        let model = self.basis.as_ref().ok_or("Resolved source unavailable")?;
        let stack = if kind == "padstacks" {
            raw
        } else {
            let pad = raw
                .get("pads")
                .and_then(Value::as_object)
                .and_then(|p| p.values().nth(index))
                .ok_or("Named pad unavailable")?;
            let id = pad
                .get("padstack")
                .and_then(Value::as_str)
                .and_then(|s| Uuid::parse_str(s).ok())
                .ok_or("Padstack reference unavailable")?;
            let candidates = self
                .entries
                .iter()
                .filter(|e| e.kind == "padstacks" && e.id == id)
                .collect::<Vec<_>>();
            if candidates.len() != 1 || candidates[0].warning.is_some() {
                return Err("Padstack source missing or ambiguous".into());
            }
            &candidates[0].raw
        };
        eda_engine::api::native_write::process::inspect_materialized_process(
            model,
            if kind == "footprints" {
                raw
            } else {
                &Value::Null
            },
            stack,
        )
        .map_err(|e| e.to_string())
    }
    /// Leaf metadata remains inspectable without squeezing entire JSON maps
    /// onto one row. Named records keep their authored names in the path.
    pub fn detail_lines(&self, entry: &LibraryEntry) -> Vec<String> {
        fn flatten(path: &str, value: &Value, lines: &mut Vec<String>) {
            match value {
                Value::Object(fields) => {
                    for (key, v) in fields {
                        let name = v.get("name").and_then(Value::as_str).unwrap_or(key);
                        flatten(
                            &if path.is_empty() {
                                name.into()
                            } else {
                                format!("{path}.{name}")
                            },
                            v,
                            lines,
                        );
                    }
                }
                Value::Array(values) => {
                    if values.is_empty() {
                        lines.push(format!("{path}: empty"));
                    } else {
                        for (i, v) in values.iter().enumerate() {
                            flatten(&format!("{path}[{i}]"), v, lines);
                        }
                    }
                }
                _ => lines.push(format!(
                    "{path}: {}",
                    value
                        .as_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| value.to_string())
                )),
            }
        }
        let mut lines = Vec::new();
        flatten("", &entry.raw, &mut lines);
        lines
    }
    pub fn capture(model: Arc<DesignModel>) -> Self {
        let mut browser = Self::default();
        for shard in model
            .source_shards
            .iter()
            .filter(|s| s.kind == SourceShardKind::Pool)
        {
            let kind = match shard.taxon {
                Some(SourceShardTaxon::PoolUnit) => "units",
                Some(SourceShardTaxon::PoolSymbol) => "symbols",
                Some(SourceShardTaxon::PoolEntity) => "entities",
                Some(SourceShardTaxon::PoolPart) => "parts",
                Some(SourceShardTaxon::PoolPackage) => "packages",
                Some(SourceShardTaxon::PoolFootprint) => "footprints",
                Some(SourceShardTaxon::PoolPadstack) => "padstacks",
                Some(SourceShardTaxon::PoolPinPadMap) => "pin_pad_maps",
                _ => model
                    .objects
                    .values()
                    .find(|o| {
                        o.domain == "pool"
                            && o.source_shard_id == shard.shard_id
                            && KINDS.iter().any(|k| k.0 == o.kind)
                    })
                    .map(|o| o.kind.as_str())
                    .unwrap_or(""),
            };
            if kind.is_empty() {
                continue;
            }
            let (raw, warning) = match model
                .materialized_source_shard_value_by_relative_path(&shard.relative_path)
            {
                Ok(raw) => (raw, None),
                Err(e) => (Value::Null, Some(e.to_string())),
            };
            let id = raw
                .get("uuid")
                .and_then(Value::as_str)
                .and_then(|s| Uuid::parse_str(s).ok());
            let id = id.unwrap_or(shard.shard_id);
            let source = model.objects.get(&id).filter(|o| {
                o.domain == "pool" && o.kind == kind && o.source_shard_id == shard.shard_id
            });
            let warning = warning.or_else(|| {
                source
                    .is_none()
                    .then(|| "Source identity is unavailable or ambiguous; inspection only".into())
            });
            let name = raw
                .get("name")
                .or_else(|| raw.get("mpn"))
                .and_then(Value::as_str)
                .unwrap_or("Unnamed source")
                .to_owned();
            browser.entries.push(LibraryEntry {
                id,
                kind: kind.into(),
                name,
                source_path: shard.relative_path.clone(),
                revision: source.map(|o| o.object_revision.0),
                raw,
                warning,
            });
        }
        // Preserve the actual resolved source order; do not invent pool ranking.
        let ambiguous: Vec<_> = browser
            .entries
            .iter()
            .filter(|e| {
                browser
                    .entries
                    .iter()
                    .filter(|other| other.id == e.id)
                    .count()
                    > 1
            })
            .map(|e| e.id)
            .collect();
        for entry in &mut browser.entries {
            if ambiguous.contains(&entry.id) {
                entry.warning =
                    Some("Multiple source candidates share this identity; inspection only".into());
            }
        }
        browser.basis = Some(model);
        browser
    }
    pub fn results(&self) -> Vec<usize> {
        let query = self.search.to_lowercase();
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                e.kind == KINDS[self.kind].0
                    && (query.is_empty()
                        || e.name.to_lowercase().contains(&query)
                        || ["manufacturer", "mpn", "tags", "parametric"]
                            .iter()
                            .filter_map(|k| e.raw.get(*k))
                            .any(|v| v.to_string().to_lowercase().contains(&query)))
            })
            .map(|(i, _)| i)
            .collect()
    }
    pub fn selected<'a>(&'a self, selection: &crate::SelectionTarget) -> Option<&'a LibraryEntry> {
        let crate::SelectionTarget::AuthoredObject(key) = selection else {
            return None;
        };
        self.entries.iter().find(|e| e.key() == *key)
    }
    pub fn choices(&self, kind: &str) -> Vec<usize> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.kind == kind && e.warning.is_none())
            .map(|(i, _)| i)
            .collect()
    }
    pub fn links(&self, entry: &LibraryEntry) -> Vec<LibraryLink> {
        let mut links = Vec::new();
        let mut push = |role: String, kind: &str, value: &Value| {
            if let Some(id) = value.as_str().and_then(|s| Uuid::parse_str(s).ok()) {
                links.push(LibraryLink {
                    role,
                    kind: kind.into(),
                    id,
                    targets: self
                        .entries
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| e.kind == kind && e.id == id)
                        .map(|(i, _)| i)
                        .collect(),
                });
            }
        };
        let fields: &[(&str, &str)] = match entry.kind.as_str() {
            "symbols" => &[("unit", "units")],
            "parts" => &[
                ("base", "parts"),
                ("entity", "entities"),
                ("package", "packages"),
                ("default_footprint", "footprints"),
                ("default_pin_pad_map", "pin_pad_maps"),
            ],
            "footprints" => &[("package", "packages")],
            "pin_pad_maps" => &[("part", "parts"), ("footprint", "footprints")],
            _ => &[],
        };
        for (field, kind) in fields {
            if let Some(value) = entry.raw.get(*field) {
                push(field.to_string(), kind, value);
            }
        }
        if let Some(gates) = entry.raw.get("gates").and_then(Value::as_object) {
            for (id, gate) in gates {
                let name = gate.get("name").and_then(Value::as_str).unwrap_or(id);
                for (field, kind) in [("unit", "units"), ("symbol", "symbols")] {
                    if let Some(v) = gate.get(field) {
                        push(format!("Gate {name} {field}"), kind, v);
                    }
                }
            }
        }
        if matches!(entry.kind.as_str(), "footprints" | "packages")
            && let Some(pads) = entry.raw.get("pads").and_then(Value::as_object)
        {
            for (id, pad) in pads {
                if let Some(v) = pad.get("padstack") {
                    push(
                        format!(
                            "Pad {} Padstack",
                            pad.get("name").and_then(Value::as_str).unwrap_or(id)
                        ),
                        "padstacks",
                        v,
                    );
                }
            }
        }
        links
    }
}
