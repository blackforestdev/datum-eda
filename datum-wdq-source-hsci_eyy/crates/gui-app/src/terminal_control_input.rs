//! Native terminal control-byte ownership.
//!
//! Datum never translates keyboard control chords into process signals. These
//! bytes enter the PTY unchanged so the kernel line discipline targets the
//! current foreground process group, while raw-mode programs receive them.

pub(super) fn control_character_sequence(text: &str) -> Option<Vec<u8>> {
    let byte = text.as_bytes().first().copied()?;
    let control = match byte {
        b'a'..=b'z' => byte - b'a' + 1,
        b'A'..=b'Z' => byte - b'A' + 1,
        b'[' => 0x1b,
        b'\\' => 0x1c,
        b']' => 0x1d,
        b'^' => 0x1e,
        b'_' => 0x1f,
        b'?' => 0x7f,
        _ => return None,
    };
    Some(vec![control])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conventional_control_chords_are_exact_pty_bytes() {
        for (text, byte) in [("c", 3), ("d", 4), ("z", 26), ("\\", 28), ("?", 127)] {
            assert_eq!(control_character_sequence(text), Some(vec![byte]));
        }
    }
}
