//! Transient Location suggestions. The typed path remains the only candidate.
use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct FolderSuggestions {
    pub input: String,
    pub parent: String,
    pub fragment: String,
}
impl FolderBrowserState {
    pub fn suggestions_ready(&self) -> bool {
        !self.loading
            && self.error.is_none()
            && self
                .suggestions
                .as_ref()
                .is_some_and(|q| q.input == self.path_input)
    }
    pub fn listing_ready(&self) -> bool {
        self.ready() || self.suggestions_ready()
    }
    pub fn suggestion_header(&self) -> Option<String> {
        let q = self.suggestions.as_ref()?;
        Some(format!(
            "Folders in {} matching “{}” · {}",
            q.parent,
            q.fragment,
            match self.entries.len() {
                0 => "no matching folders".into(),
                1 => "1 match · Tab completes".into(),
                n => format!("{n} matches"),
            }
        ))
    }
    pub fn suggestion_empty_message(&self) -> Option<String> {
        let q = self.suggestions.as_ref()?;
        Some(format!(
            "No folders in {} match “{}”. Keep typing, go Up, or choose Home.",
            q.parent, q.fragment
        ))
    }
    pub fn suggestion_path(&self, index: usize) -> Option<String> {
        if !self.suggestions_ready() {
            return None;
        }
        let entry = self.entries.get(index).filter(|e| e.navigable())?;
        Some(self.completed_leaf(&entry.name, true))
    }
    fn completed_leaf(&self, leaf: &str, separator: bool) -> String {
        let input = self.path_input.trim_end_matches('/');
        let parent = input.rfind('/').map(|i| &input[..=i]).unwrap_or("");
        format!("{parent}{leaf}{}", if separator { "/" } else { "" })
    }
    /// Exact spelling only. An ambiguous extension keeps its unsuffixed query;
    /// appending the separator explicitly accepts a directory.
    pub fn tab_suggestion(&self) -> Option<String> {
        if self.path_editor.caret != self.path_input.len()
            || !self.path_editor.selection(&self.path_input).is_empty()
        {
            return None;
        }
        if self.ready() {
            return (!self.path_input.ends_with('/')).then(|| format!("{}/", self.path_input));
        }
        if !self.suggestions_ready() {
            return None;
        }
        if let Some(index) = self.selected {
            return self.suggestion_path(index);
        }
        let q = self.suggestions.as_ref()?;
        let folded = q.fragment.to_lowercase();
        let mut group: Vec<_> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                entry.navigable() && entry.name.to_lowercase().starts_with(&folded)
            })
            .collect();
        if group.is_empty() {
            group = self
                .entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| entry.navigable())
                .collect();
        }
        if group.len() == 1 {
            return self.suggestion_path(group[0].0);
        }
        let first = group.first()?.1.name.as_str();
        if !group
            .iter()
            .all(|(_, entry)| entry.name.starts_with(&q.fragment))
        {
            return None;
        }
        let mut length = first.len();
        for (_, entry) in &group[1..] {
            length = length.min(
                first
                    .bytes()
                    .zip(entry.name.bytes())
                    .take_while(|(a, b)| a == b)
                    .count(),
            );
        }
        while !first.is_char_boundary(length) {
            length -= 1;
        }
        let shared = &first[..length];
        if length <= q.fragment.len() {
            return None;
        }
        Some(self.completed_leaf(shared, false))
    }
}

/// Map the folded literal match back to complete original UTF-8 characters.
pub fn folder_match_span(name: &str, query: &str) -> Option<std::ops::Range<usize>> {
    if query.is_empty() {
        return None;
    }
    let mut folded = String::new();
    let mut positions = vec![];
    for (offset, c) in name.char_indices() {
        let start = folded.len();
        folded.extend(c.to_lowercase());
        positions.push((start..folded.len(), offset..offset + c.len_utf8()));
    }
    let needle = query.to_lowercase();
    let start = folded.find(&needle)?;
    let end = start + needle.len();
    let first = positions.iter().find(|(r, _)| r.end > start)?.1.start;
    let last = positions.iter().rev().find(|(r, _)| r.start < end)?.1.end;
    Some(first..last)
}

#[cfg(test)]
#[path = "suggestions_tests.rs"]
mod tests;
