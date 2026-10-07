//! Bounded direct-child extraction for graphic records, including compact input.
use crate::error::EngineError;
use crate::ir::{
    geometry::Point,
    units::{LengthUnit, parse_fixed_length},
};

pub(super) fn refusal(reason: impl Into<String>) -> EngineError {
    EngineError::Import(reason.into())
}
pub(super) fn head(block: &str) -> &str {
    block
        .trim_start_matches('(')
        .split(|c: char| c.is_whitespace() || c == '(' || c == ')')
        .next()
        .unwrap_or("")
}
pub(super) fn children(block: &str) -> Result<Vec<&str>, EngineError> {
    let mut result = Vec::new();
    let (mut depth, mut quoted, mut escape, mut start) = (0usize, false, false, 0usize);
    for (i, byte) in block.bytes().enumerate() {
        if quoted {
            if escape {
                escape = false;
            } else if byte == b'\\' {
                escape = true;
            } else if byte == b'"' {
                quoted = false;
            }
            continue;
        }
        match byte {
            b'"' => quoted = true,
            b'(' => {
                depth += 1;
                if depth == 2 {
                    start = i;
                }
            }
            b')' => {
                if depth == 0 {
                    return Err(refusal("unbalanced graphic source"));
                }
                if depth == 2 {
                    result.push(&block[start..=i]);
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    if depth != 0 || quoted {
        return Err(refusal("incomplete graphic source"));
    }
    Ok(result)
}
pub(super) fn field<'a>(block: &'a str, name: &str) -> Result<Option<&'a str>, EngineError> {
    let values: Vec<_> = children(block)?
        .into_iter()
        .filter(|b| head(b) == name)
        .collect();
    if values.len() > 1 {
        return Err(refusal(format!("ambiguous graphic {name}")));
    }
    Ok(values.first().copied())
}
pub(super) fn scalar<'a>(block: &'a str, name: &str) -> Result<&'a str, EngineError> {
    let value = field(block, name)?.ok_or_else(|| refusal(format!("missing graphic {name}")))?;
    let tokens: Vec<_> = value.trim_matches(['(', ')']).split_whitespace().collect();
    if tokens.len() != 2 {
        return Err(refusal(format!("invalid graphic {name}")));
    }
    Ok(tokens[1].trim_matches('"'))
}
fn nm(token: &str) -> Result<i64, EngineError> {
    parse_fixed_length(token, LengthUnit::Millimeter)
        .map(|n| n.get())
        .map_err(|e| refusal(format!("graphic length {token:?} is not exact: {e:?}")))
}
pub(super) fn point(block: &str, name: &str) -> Result<Point, EngineError> {
    let value = field(block, name)?.ok_or_else(|| refusal(format!("missing graphic {name}")))?;
    let tokens: Vec<_> = value.trim_matches(['(', ')']).split_whitespace().collect();
    if tokens.len() != 3 {
        return Err(refusal(format!("invalid graphic {name}")));
    }
    Ok(Point::new(nm(tokens[1])?, nm(tokens[2])?))
}
pub(super) fn width(block: &str) -> Result<i64, EngineError> {
    match field(block, "stroke")? {
        Some(stroke) => {
            for child in children(stroke)? {
                if !matches!(head(child), "width" | "type") {
                    return Err(refusal(format!(
                        "unsupported graphic stroke field {}",
                        head(child)
                    )));
                }
            }
            if field(block, "width")?.is_some() {
                return Err(refusal("ambiguous graphic stroke width"));
            }
            if field(stroke, "type")?.is_some() && scalar(stroke, "type")? != "default" {
                return Err(refusal("unsupported non-solid graphic stroke"));
            }
            nm(scalar(stroke, "width")?)
        }
        None => nm(scalar(block, "width")?),
    }
}
pub(super) fn filled(block: &str) -> Result<bool, EngineError> {
    match field(block, "fill")? {
        None => Ok(false),
        Some(_) => match scalar(block, "fill")? {
            "solid" | "yes" => Ok(true),
            "none" | "no" => Ok(false),
            _ => Err(refusal("unsupported graphic fill style")),
        },
    }
}
pub(super) fn polygon(block: &str) -> Result<Vec<Point>, EngineError> {
    let pts = field(block, "pts")?.ok_or_else(|| refusal("missing graphic points"))?;
    children(pts)?
        .into_iter()
        .map(|v| {
            if head(v) != "xy" {
                return Err(refusal("unsupported curved graphic polygon edge"));
            }
            point(&format!("(point {v})"), "xy")
        })
        .collect()
}
