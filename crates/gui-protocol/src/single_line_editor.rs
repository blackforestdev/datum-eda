//! Shared cursor/selection operations over a caller-owned single-line value.
//! Text remains with its form; this model never owns a second authoritative copy.
use std::ops::Range;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SingleLineEditor {
    pub caret: usize,
    pub anchor: usize,
}

impl SingleLineEditor {
    pub fn at_end(text: &str) -> Self {
        Self {
            caret: text.len(),
            anchor: text.len(),
        }
    }

    pub fn selection(&self, text: &str) -> Range<usize> {
        let a = boundary(text, self.anchor);
        let b = boundary(text, self.caret);
        a.min(b)..a.max(b)
    }

    pub fn selected_text<'a>(&self, text: &'a str) -> &'a str {
        &text[self.selection(text)]
    }

    pub fn position(&mut self, text: &str, byte: usize, extend: bool) {
        self.caret = boundary(text, byte);
        self.anchor = if extend {
            boundary(text, self.anchor)
        } else {
            self.caret
        };
    }

    pub fn select_all(&mut self, text: &str) {
        self.anchor = 0;
        self.caret = text.len();
    }

    pub fn home(&mut self, text: &str, extend: bool) {
        self.position(text, 0, extend);
    }
    pub fn end(&mut self, text: &str, extend: bool) {
        self.position(text, text.len(), extend);
    }

    pub fn left(&mut self, text: &str, extend: bool) {
        let selection = self.selection(text);
        let caret = boundary(text, self.caret);
        let next = if !extend && !selection.is_empty() {
            selection.start
        } else {
            text[..caret]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0)
        };
        self.position(text, next, extend);
    }

    pub fn right(&mut self, text: &str, extend: bool) {
        let selection = self.selection(text);
        let caret = boundary(text, self.caret);
        let next = if !extend && !selection.is_empty() {
            selection.end
        } else {
            caret
                + text[caret..]
                    .chars()
                    .next()
                    .map(char::len_utf8)
                    .unwrap_or(0)
        };
        self.position(text, next, extend);
    }

    /// A clipboard newline/control is refused as a whole, never silently trimmed.
    pub fn replace(&mut self, text: &mut String, value: &str) -> Result<bool, &'static str> {
        if value.chars().any(char::is_control) {
            return Err("Enter a single line without control characters.");
        }
        let selection = self.selection(text);
        if selection.is_empty() && value.is_empty() {
            return Ok(false);
        }
        let next = selection.start + value.len();
        text.replace_range(selection, value);
        self.caret = next;
        self.anchor = next;
        Ok(true)
    }

    pub fn backspace(&mut self, text: &mut String) -> bool {
        if self.selection(text).is_empty() {
            self.left(text, true);
        }
        self.replace(text, "").unwrap_or(false)
    }

    pub fn delete(&mut self, text: &mut String) -> bool {
        if self.selection(text).is_empty() {
            self.right(text, true);
        }
        self.replace(text, "").unwrap_or(false)
    }
}

fn boundary(text: &str, byte: usize) -> usize {
    let mut byte = byte.min(text.len());
    while !text.is_char_boundary(byte) {
        byte -= 1;
    }
    byte
}

#[cfg(test)]
#[path = "single_line_editor_tests.rs"]
mod tests;
