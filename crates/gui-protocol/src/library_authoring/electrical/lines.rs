//! Disposable Line rows use original source indices, never geometry or persistent IDs.
use super::*;

const FIELDS: &[(&str, &str, &str)] = &[
    ("line_from_x", "Line start X (nm)", "/Line/from/x"),
    ("line_from_y", "Line start Y (nm)", "/Line/from/y"),
    ("line_to_x", "Line end X (nm)", "/Line/to/x"),
    ("line_to_y", "Line end Y (nm)", "/Line/to/y"),
    ("line_width", "Line stroke width (nm)", "/Line/width"),
];
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LineDraft {
    pub rows: BTreeMap<usize, Value>,
    pub selected: Option<usize>,
    next: usize,
}
impl LibraryDraft {
    pub(super) fn hydrate_lines(&mut self) {
        let rows = self
            .electrical
            .original
            .as_ref()
            .and_then(|r| r["drawings"].as_array());
        self.electrical.lines = LineDraft {
            rows: rows
                .into_iter()
                .flatten()
                .enumerate()
                .filter(|(_, raw)| raw.get("Line").is_some())
                .map(|(i, raw)| (i, raw.clone()))
                .collect(),
            selected: None,
            next: rows.map_or(0, Vec::len),
        };
    }
    fn original_graphic_count(&self) -> usize {
        self.electrical
            .original
            .as_ref()
            .and_then(|r| r["drawings"].as_array())
            .map_or(0, Vec::len)
    }
    fn buffered_lines(&self) -> Result<BTreeMap<usize, Value>, String> {
        let mut rows = self.electrical.lines.rows.clone();
        if let Some(index) = self.electrical.lines.selected {
            let row = rows.get_mut(&index).ok_or("Selected Line is unavailable")?;
            for &(key, _, pointer) in FIELDS {
                let value = self.length(key)?;
                let leaf = row
                    .pointer_mut(pointer)
                    .ok_or("Selected Line coordinates are unavailable")?;
                if leaf.as_i64() != Some(value) {
                    *leaf = json!(value);
                }
            }
            let eda_engine::pool::Primitive::Line { from, to, width } =
                serde_json::from_value(row.clone())
                    .map_err(|_| "Selected Line data is unavailable")?
            else {
                return Err("Select a native Line".into());
            };
            if index >= self.original_graphic_count() && (from == to || width <= 0) {
                return Err(
                    "New Line requires distinct endpoints and positive stroke width".into(),
                );
            }
        }
        Ok(rows)
    }
    fn bind_line_fields(&mut self) {
        self.fields.retain(|f| !f.key.starts_with("line_"));
        self.dirty.retain(|key| !key.starts_with("line_"));
        if let Some(raw) = self
            .electrical
            .lines
            .selected
            .and_then(|i| self.electrical.lines.rows.get(&i))
            .cloned()
        {
            for &(key, label, pointer) in FIELDS {
                self.field(key, label, raw.pointer(pointer), vec![]);
            }
        }
        self.focused = self.focused.min(self.fields.len().saturating_sub(1));
        self.offset = self.offset.min(self.focused);
    }
    pub fn switch_line(&mut self, index: Option<usize>, remove: bool) -> Result<(), String> {
        if self.kind != "symbols" || self.create {
            return Err("Line authoring requires a saved Symbol draft".into());
        }
        if let Some(index) = index
            && !self.electrical.lines.rows.contains_key(&index)
        {
            return Err("Choose an existing Line row".into());
        }
        if remove {
            let selected = self
                .electrical
                .lines
                .selected
                .ok_or("Select a Line to remove")?;
            self.electrical.lines.rows.remove(&selected);
            self.electrical.lines.selected = None;
        } else {
            let mut rows = self.buffered_lines()?;
            let index = if let Some(index) = index {
                index
            } else {
                let index = self.electrical.lines.next;
                self.electrical.lines.next =
                    index.checked_add(1).ok_or("Too many draft graphics")?;
                rows.insert(
                    index,
                    json!({"Line":{"from":{"x":0,"y":0},"to":{"x":1000000,"y":0},"width":100000}}),
                );
                index
            };
            self.electrical.lines.rows = rows;
            self.electrical.lines.selected = Some(index);
        }
        self.bind_line_fields();
        self.preview = None;
        self.refusal = None;
        Ok(())
    }
    pub(super) fn line_request(&self, fields: &mut Map<String, Value>) -> Result<(), String> {
        let rows = self.buffered_lines()?;
        let original = self
            .electrical
            .original
            .as_ref()
            .and_then(|r| r["drawings"].as_array());
        let mut edits = Vec::new();
        let mut removals = Vec::new();
        for (index, previous) in original
            .into_iter()
            .flatten()
            .enumerate()
            .filter(|(_, r)| r.get("Line").is_some())
        {
            if let Some(line) = rows.get(&index) {
                if line != previous {
                    edits.push(json!({"index":index,"previous":previous,"line":line}));
                }
            } else {
                removals.push(json!({"index":index,"previous":previous}));
            }
        }
        let additions: Vec<_> = rows
            .into_iter()
            .filter(|(i, _)| *i >= self.original_graphic_count())
            .map(|(_, raw)| raw)
            .collect();
        for (key, values) in [
            ("line_edits", edits),
            ("line_removals", removals),
            ("line_additions", additions),
        ] {
            if !values.is_empty() {
                fields.insert(key.into(), values.into());
            }
        }
        Ok(())
    }
}
