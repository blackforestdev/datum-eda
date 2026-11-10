//! One disposable selection and numeric buffer for captured Line and Rect rows.
use super::*;

const LINE: &[(&str, &str, &str)] = &[
    ("line_from_x", "Line start X (nm)", "/Line/from/x"),
    ("line_from_y", "Line start Y (nm)", "/Line/from/y"),
    ("line_to_x", "Line end X (nm)", "/Line/to/x"),
    ("line_to_y", "Line end Y (nm)", "/Line/to/y"),
    ("line_width", "Line stroke width (nm)", "/Line/width"),
];
const RECT: &[(&str, &str, &str)] = &[
    ("rect_min_x", "Rectangle minimum X (nm)", "/Rect/min/x"),
    ("rect_min_y", "Rectangle minimum Y (nm)", "/Rect/min/y"),
    ("rect_max_x", "Rectangle maximum X (nm)", "/Rect/max/x"),
    ("rect_max_y", "Rectangle maximum Y (nm)", "/Rect/max/y"),
    ("rect_width", "Rectangle stroke width (nm)", "/Rect/width"),
];
fn fields(raw: &Value) -> &'static [(&'static str, &'static str, &'static str)] {
    if raw.get("Rect").is_some() {
        RECT
    } else {
        LINE
    }
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GraphicDraft {
    pub rows: BTreeMap<usize, Value>,
    pub selected: Option<usize>,
    next: usize,
}
impl GraphicDraft {
    /// The painter and keyboard traversal expose the same ordered actions.
    pub fn controls(&self) -> Vec<crate::library_browser::LibraryControl> {
        use crate::library_browser::LibraryControl as C;
        let mut controls: Vec<_> = self
            .rows
            .iter()
            .map(|(i, raw)| {
                if raw.get("Rect").is_some() {
                    C::Rect(*i)
                } else {
                    C::Line(*i)
                }
            })
            .collect();
        controls.extend([C::AddLine, C::AddRect]);
        if let Some(raw) = self.selected.and_then(|i| self.rows.get(&i)) {
            controls.push(if raw.get("Rect").is_some() {
                C::RemoveRect
            } else {
                C::RemoveLine
            });
        }
        controls
    }
}
impl LibraryDraft {
    pub(super) fn hydrate_graphics(&mut self) {
        let rows = self
            .electrical
            .original
            .as_ref()
            .and_then(|r| r["drawings"].as_array());
        self.electrical.graphics = GraphicDraft {
            rows: rows
                .into_iter()
                .flatten()
                .enumerate()
                .filter(|(_, raw)| raw.get("Line").is_some() || raw.get("Rect").is_some())
                .map(|(i, raw)| (i, raw.clone()))
                .collect(),
            selected: self.saved_rectangle_index(),
            next: rows.map_or(0, Vec::len),
        };
        self.bind_graphic_fields();
    }
    fn original_graphic_count(&self) -> usize {
        self.electrical
            .original
            .as_ref()
            .and_then(|r| r["drawings"].as_array())
            .map_or(0, Vec::len)
    }
    fn buffered_graphics(&self) -> Result<BTreeMap<usize, Value>, String> {
        let mut rows = self.electrical.graphics.rows.clone();
        if let Some(index) = self.electrical.graphics.selected {
            let row = rows
                .get_mut(&index)
                .ok_or("Selected graphic is unavailable")?;
            let previous = self
                .electrical
                .original
                .as_ref()
                .and_then(|v| v["drawings"].get(index))
                .cloned()
                .unwrap_or_else(|| row.clone());
            for &(key, _, pointer) in fields(row) {
                let value = self.length(key)?;
                let leaf = row
                    .pointer_mut(pointer)
                    .ok_or("Selected graphic coordinates are unavailable")?;
                if leaf.as_i64() != Some(value) {
                    *leaf = json!(value);
                }
            }
            let new = index >= self.original_graphic_count();
            match serde_json::from_value::<eda_engine::pool::Primitive>(row.clone())
                .map_err(|_| "Selected graphic data is unavailable")?
            {
                eda_engine::pool::Primitive::Line { from, to, width }
                    if new && (from == to || width <= 0) =>
                {
                    return Err(
                        "New Line requires distinct endpoints and positive stroke width".into(),
                    );
                }
                eda_engine::pool::Primitive::Rect { min, max, width }
                    if (new || *row != previous)
                        && (min.x >= max.x || min.y >= max.y || width <= 0) =>
                {
                    return Err(
                        "Rectangle requires ordered corners and positive stroke width".into(),
                    );
                }
                _ => {}
            }
        }
        Ok(rows)
    }
    fn bind_graphic_fields(&mut self) {
        self.fields
            .retain(|f| !f.key.starts_with("line_") && !f.key.starts_with("rect_"));
        self.dirty
            .retain(|key| !key.starts_with("line_") && !key.starts_with("rect_"));
        if let Some(raw) = self
            .electrical
            .graphics
            .selected
            .and_then(|i| self.electrical.graphics.rows.get(&i))
            .cloned()
        {
            for &(key, label, pointer) in fields(&raw) {
                self.field(key, label, raw.pointer(pointer), vec![]);
            }
        }
        self.focused = self.focused.min(self.fields.len().saturating_sub(1));
        self.offset = self.offset.min(self.focused);
    }
    pub fn switch_line(&mut self, index: Option<usize>, remove: bool) -> Result<(), String> {
        self.switch_graphic("Line", index, remove)
    }
    pub fn switch_rect(&mut self, index: Option<usize>, remove: bool) -> Result<(), String> {
        self.switch_graphic("Rect", index, remove)
    }
    fn switch_graphic(
        &mut self,
        kind: &str,
        index: Option<usize>,
        remove: bool,
    ) -> Result<(), String> {
        if self.kind != "symbols" || self.create {
            return Err("Graphic authoring requires a saved Symbol draft".into());
        }
        if let Some(index) = index
            && !self
                .electrical
                .graphics
                .rows
                .get(&index)
                .is_some_and(|r| r.get(kind).is_some())
        {
            return Err(format!("Choose an existing {kind} row"));
        }
        if remove {
            let selected = self
                .electrical
                .graphics
                .selected
                .filter(|i| {
                    self.electrical
                        .graphics
                        .rows
                        .get(i)
                        .is_some_and(|r| r.get(kind).is_some())
                })
                .ok_or_else(|| format!("Select a {kind} to remove"))?;
            self.electrical.graphics.rows.remove(&selected);
            self.electrical.graphics.selected = None;
        } else {
            let mut rows = self.buffered_graphics()?;
            let index = if let Some(index) = index {
                index
            } else {
                let index = self.electrical.graphics.next;
                let next = index.checked_add(1).ok_or("Too many draft graphics")?;
                let raw = if kind == "Rect" {
                    json!({"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}})
                } else {
                    json!({"Line":{"from":{"x":0,"y":0},"to":{"x":1000000,"y":0},"width":100000}})
                };
                rows.insert(index, raw);
                self.electrical.graphics.next = next;
                index
            };
            self.electrical.graphics.rows = rows;
            self.electrical.graphics.selected = Some(index);
        }
        self.bind_graphic_fields();
        self.preview = None;
        self.refusal = None;
        Ok(())
    }
    pub(super) fn graphic_request(&self, fields: &mut Map<String, Value>) -> Result<(), String> {
        let rows = self.buffered_graphics()?;
        let original = self
            .electrical
            .original
            .as_ref()
            .and_then(|r| r["drawings"].as_array());
        for (kind, edit_key, remove_key, payload) in [
            ("Line", "line_edits", "line_removals", "line"),
            ("Rect", "rect_edits", "rect_removals", "rect"),
        ] {
            let mut edits = Vec::new();
            let mut removals = Vec::new();
            for (index, previous) in original
                .into_iter()
                .flatten()
                .enumerate()
                .filter(|(_, r)| r.get(kind).is_some())
            {
                if let Some(row) = rows.get(&index) {
                    if row != previous {
                        if kind == "Rect" && self.saved_rectangle_index() == Some(index) {
                            let mut drawings = original.unwrap().clone();
                            drawings[index] = row.clone();
                            fields.insert("drawings".into(), drawings.into());
                        } else {
                            let mut edit = json!({"index":index,"previous":previous});
                            edit[payload] = row.clone();
                            edits.push(edit);
                        }
                    }
                } else {
                    removals.push(json!({"index":index,"previous":previous}));
                }
            }
            for (key, values) in [(edit_key, edits), (remove_key, removals)] {
                if !values.is_empty() {
                    fields.insert(key.into(), values.into());
                }
            }
        }
        let additions: Vec<_> = rows
            .into_iter()
            .filter(|(i, _)| *i >= self.original_graphic_count())
            .map(|(_, raw)| raw)
            .collect();
        if !additions.is_empty() {
            let key = if additions.iter().any(|r| r.get("Rect").is_some()) {
                "graphic_additions"
            } else {
                "line_additions"
            };
            fields.insert(key.into(), additions.into());
        }
        Ok(())
    }
}
