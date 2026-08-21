//! Unicode-scalar range calculations for the AT-SPI Text interface.

type RangeResult<T> = Result<T, (&'static str, &'static str)>;

pub(super) fn text_at_offset(
    chars: &[char],
    offset: i32,
    granularity: u32,
) -> RangeResult<(String, i32, i32)> {
    let offset = usize::try_from(offset).map_err(|_| invalid_args())?;
    if offset >= chars.len() {
        return Ok((
            String::new(),
            clamp_i32(chars.len()),
            clamp_i32(chars.len()),
        ));
    }
    let (start, end) = match granularity {
        0 => (offset, offset + 1),
        1 => bounded_run(chars, offset, |ch| ch.is_alphanumeric() || ch == '_'),
        2 => bounded_run(chars, offset, |ch| !matches!(ch, '.' | '!' | '?' | '\n')),
        3 | 4 => bounded_run(chars, offset, |ch| ch != '\n'),
        _ => return Err(invalid_args()),
    };
    Ok((
        chars[start..end].iter().collect(),
        clamp_i32(start),
        clamp_i32(end),
    ))
}

pub(super) fn char_range(chars: &[char], start: i32, end: i32) -> RangeResult<String> {
    let start = usize::try_from(start)
        .map_err(|_| invalid_args())?
        .min(chars.len());
    let end = if end < 0 {
        chars.len()
    } else {
        usize::try_from(end)
            .map_err(|_| invalid_args())?
            .min(chars.len())
    };
    if start > end {
        return Err(invalid_args());
    }
    Ok(chars[start..end].iter().collect())
}

fn bounded_run(chars: &[char], offset: usize, matches: impl Fn(char) -> bool) -> (usize, usize) {
    let target = matches(chars[offset]);
    let start = (0..offset)
        .rev()
        .find(|index| matches(chars[*index]) != target)
        .map_or(0, |index| index + 1);
    let end = (offset + 1..chars.len())
        .find(|index| matches(chars[*index]) != target)
        .unwrap_or(chars.len());
    (start, end)
}

fn clamp_i32(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

fn invalid_args() -> (&'static str, &'static str) {
    (
        "org.freedesktop.DBus.Error.InvalidArgs",
        "invalid arguments",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_ranges_do_not_split_utf8() {
        let chars = "aβc\nline".chars().collect::<Vec<_>>();
        assert_eq!(char_range(&chars, 1, 2).unwrap(), "β");
        assert_eq!(text_at_offset(&chars, 5, 3).unwrap(), ("line".into(), 4, 8));
    }
}
