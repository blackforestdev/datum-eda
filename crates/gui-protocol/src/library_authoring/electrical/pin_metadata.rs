//! Per-pin alternate names and swap group buffered by captured row index.
use super::*;
use crate::library_browser::LibraryControl as C;

const GROUP: &str = "pin_swap_group";
const NAME: &str = "pin_alternate_name";
const KIND: &str = "pin_alternate_kind";

#[derive(Debug, Clone, PartialEq)]
pub struct PinMetadataDraft {
    pub rows: BTreeMap<usize, Value>,
    pub selected: Option<usize>,
    group: u32,
    next: usize,
}

impl PinMetadataDraft {
    fn capture(raw: &Value, id: Uuid) -> Option<Self> {
        let pin: eda_engine::pool::Pin = serde_json::from_value(raw.clone()).ok()?;
        if pin.uuid != id || id.is_nil() {
            return None;
        }
        let rows = raw["alternates"].as_array()?;
        Some(Self {
            group: pin.swap_group,
            next: rows.len(),
            rows: rows.iter().cloned().enumerate().collect(),
            selected: None,
        })
    }

    pub fn controls(&self) -> impl Iterator<Item = C> + '_ {
        self.rows
            .keys()
            .copied()
            .map(C::PinAlternate)
            .chain([C::AddPinAlternate])
            .chain(self.selected.map(|_| C::RemovePinAlternate))
    }

    fn request(&self, original: &Value) -> Value {
        let mut request = json!({});
        let previous = original["alternates"].as_array().unwrap();
        let mut remaining = self.rows.clone();
        let mut edits = Vec::new();
        let mut removals = Vec::new();
        for (index, row) in previous.iter().enumerate() {
            match remaining.remove(&index) {
                Some(current) if &current != row => edits.push(json!({
                    "index": index, "previous": row, "name": current["name"], "kind": current["kind"],
                })),
                None => removals.push(json!({"index":index,"previous":row})),
                _ => (),
            }
        }
        if !edits.is_empty() {
            request["alternate_edits"] = edits.into();
        }
        if !removals.is_empty() {
            request["alternate_removals"] = removals.into();
        }
        if !remaining.is_empty() {
            request["alternate_additions"] = Value::Array(remaining.into_values().collect());
        }
        if original["swap_group"].as_u64() != Some(self.group.into()) {
            request["swap_group"] = json!(self.group);
        }
        if request != json!({}) {
            request["previous"] = original.clone();
        }
        request
    }
}

impl LibraryDraft {
    pub fn selected_pin_metadata(&self) -> Option<&PinMetadataDraft> {
        self.electrical.pin_metadata.get(&self.electrical.pin_id?)
    }

    fn pending_pin_metadata(
        &self,
        mut draft: PinMetadataDraft,
    ) -> Result<PinMetadataDraft, String> {
        if self.dirty.contains(GROUP) {
            draft.group = self
                .value(GROUP)
                .parse()
                .map_err(|_| "Enter a swap group from 0 to 4294967295")?;
        }
        if let Some(index) = draft.selected {
            let row = draft.rows.get_mut(&index).unwrap();
            row["name"] = json!(self.value(NAME));
            row["kind"] = json!(self.value(KIND));
        }
        Ok(draft)
    }

    pub(super) fn leave_pin_metadata(&mut self, remove: bool) -> Result<(), String> {
        if let Some(id) = self.electrical.pin_id {
            if remove {
                self.electrical.pin_metadata.remove(&id);
            } else if let Some(draft) = self.selected_pin_metadata() {
                let pending = self.pending_pin_metadata(draft.clone())?;
                self.electrical.pin_metadata.insert(id, pending);
            }
        }
        Ok(())
    }

    pub(super) fn bind_pin_metadata(&mut self) {
        if let Some(id) = self.electrical.pin_id
            && let std::collections::btree_map::Entry::Vacant(entry) =
                self.electrical.pin_metadata.entry(id)
            && let Some(raw) = self
                .electrical
                .original
                .as_ref()
                .and_then(|value| value["pins"].as_object())
            && let Ok(key) = pin_key(raw, id)
            && let Some(draft) = PinMetadataDraft::capture(&raw[&key], id)
        {
            entry.insert(draft);
        }
        self.fields.retain(|field| field.key != GROUP);
        self.dirty.remove(GROUP);
        if let Some(draft) = self.selected_pin_metadata() {
            self.field(
                GROUP,
                "Swap group (0 means none)",
                Some(&json!(draft.group)),
                vec![],
            );
        }
        self.bind_alternate_fields();
    }

    fn bind_alternate_fields(&mut self) {
        self.fields
            .retain(|field| ![NAME, KIND].contains(&field.key));
        for key in [NAME, KIND] {
            self.dirty.remove(key);
        }
        if let Some(draft) = self.selected_pin_metadata()
            && let Some(index) = draft.selected
        {
            let row = draft.rows[&index].clone();
            self.field(NAME, "Alternate name", row.get("name"), vec![]);
            self.field(KIND, "Alternate kind", row.get("kind"), vec![]);
        }
    }

    pub fn switch_pin_alternate_control(
        &mut self,
        control: &C,
    ) -> Option<Result<&'static str, String>> {
        let selected = match *control {
            C::PinAlternate(index) => Some(index),
            C::AddPinAlternate | C::RemovePinAlternate => None,
            _ => return None,
        };
        Some(self.switch_pin_alternate(selected, *control == C::RemovePinAlternate))
    }

    fn switch_pin_alternate(
        &mut self,
        selected: Option<usize>,
        remove: bool,
    ) -> Result<&'static str, String> {
        let id = self.electrical.pin_id.ok_or("Select a saved Unit pin")?;
        let draft = self
            .selected_pin_metadata()
            .ok_or("Select a saved Unit pin")?;
        if selected.is_some_and(|i| !draft.rows.contains_key(&i)) {
            return Err("Choose an existing alternate name".into());
        }
        let mut pending = draft.clone();
        if remove {
            let index = draft.selected.ok_or("Select an alternate name to remove")?;
            pending.rows.remove(&index);
            pending.selected = None;
        } else {
            // The swap group is independent of alternate-row selection.
            if let Some(index) = pending.selected {
                pending.rows.get_mut(&index).unwrap()["name"] = json!(self.value(NAME));
                pending.rows.get_mut(&index).unwrap()["kind"] = json!(self.value(KIND));
            }
            pending.selected = selected.or_else(|| {
                let index = pending.next;
                pending.next += 1;
                pending.rows.insert(index, json!({"name":"","kind":""}));
                Some(index)
            });
        }
        self.electrical.pin_metadata.insert(id, pending);
        self.bind_alternate_fields();
        self.preview = None;
        self.refusal = None;
        Ok(if remove { GROUP } else { NAME })
    }

    pub(super) fn pin_metadata_request(
        &self,
        original: &Value,
        fields: &mut Map<String, Value>,
    ) -> Result<(), String> {
        let old = original["pins"]
            .as_object()
            .ok_or("Saved Unit pins are unavailable")?;
        let mut edits = Map::new();
        for (id, draft) in &self.electrical.pin_metadata {
            let key = pin_key(old, *id)?;
            let pending = if self.electrical.pin_id == Some(*id) {
                self.pending_pin_metadata(draft.clone())?
            } else {
                draft.clone()
            };
            let request = pending.request(&old[&key]);
            if request != json!({}) {
                edits.insert(key, request);
            }
        }
        if !edits.is_empty() {
            fields.insert("pin_metadata".into(), edits.into());
        }
        Ok(())
    }
}
