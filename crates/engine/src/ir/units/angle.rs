use super::{QuantityKind, RefusalReason, parse_decimal, rational_to_i64};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanonicalAngleScale {
    units_per_degree: i64,
}

impl CanonicalAngleScale {
    pub const fn new(units_per_degree: i64) -> Option<Self> {
        if units_per_degree > 0 {
            Some(Self { units_per_degree })
        } else {
            None
        }
    }

    pub const fn units_per_degree(self) -> i64 {
        self.units_per_degree
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAngle {
    pub original_token: String,
    pub canonical_value: i64,
    pub scale: CanonicalAngleScale,
    pub explicit_degrees: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormattedAngle {
    pub canonical_value: i64,
    pub scale: CanonicalAngleScale,
    pub decimal_places: u8,
    pub rendered: String,
    pub rounded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AngleRefusal {
    pub quantity: QuantityKind,
    pub original_token: Option<String>,
    pub scale: Option<CanonicalAngleScale>,
    pub reason: RefusalReason,
}

pub fn parse_decimal_degrees(
    token: &str,
    scale: Option<CanonicalAngleScale>,
    allow_bare: bool,
) -> Result<ParsedAngle, AngleRefusal> {
    let original_token = token.to_owned();
    let refuse = |reason| AngleRefusal {
        quantity: QuantityKind::Angle,
        original_token: Some(original_token.clone()),
        scale,
        reason,
    };
    let scale = scale.ok_or_else(|| refuse(RefusalReason::MissingCanonicalAngleScale))?;
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return Err(refuse(RefusalReason::EmptyToken));
    }
    let (number, suffix) = split_angle_token(trimmed);
    let explicit_degrees = match suffix {
        "°" => true,
        degrees if degrees.eq_ignore_ascii_case("deg") => true,
        "" if allow_bare => false,
        "" => return Err(refuse(RefusalReason::MissingUnitContext)),
        other => return Err(refuse(RefusalReason::UnsupportedSuffix(other.to_owned()))),
    };
    let (numerator, denominator) = parse_decimal(number).map_err(&refuse)?;
    let canonical_value =
        rational_to_i64(numerator, denominator, i128::from(scale.units_per_degree()))
            .map_err(&refuse)?;
    Ok(ParsedAngle {
        original_token,
        canonical_value,
        scale,
        explicit_degrees,
    })
}

pub fn format_decimal_degrees(
    canonical_value: i64,
    scale: CanonicalAngleScale,
    decimal_places: u8,
) -> Result<FormattedAngle, AngleRefusal> {
    if decimal_places > 3 {
        return Err(AngleRefusal {
            quantity: QuantityKind::Angle,
            original_token: None,
            scale: Some(scale),
            reason: RefusalReason::UnsupportedPrecision(decimal_places),
        });
    }
    let factor = 10_i128
        .checked_pow(u32::from(decimal_places))
        .ok_or(AngleRefusal {
            quantity: QuantityKind::Angle,
            original_token: None,
            scale: Some(scale),
            reason: RefusalReason::Overflow,
        })?;
    let magnitude = i128::from(canonical_value).abs();
    let scaled = magnitude.checked_mul(factor).ok_or(AngleRefusal {
        quantity: QuantityKind::Angle,
        original_token: None,
        scale: Some(scale),
        reason: RefusalReason::Overflow,
    })?;
    let divisor = i128::from(scale.units_per_degree());
    let quotient = scaled / divisor;
    let remainder = scaled % divisor;
    let rounded = remainder != 0;
    let rounded_value = if remainder * 2 >= divisor {
        quotient + 1
    } else {
        quotient
    };
    let sign = if canonical_value < 0 { "-" } else { "" };
    let rendered = if decimal_places == 0 {
        format!("{sign}{rounded_value}deg")
    } else {
        let whole = rounded_value / factor;
        let fraction = rounded_value % factor;
        format!(
            "{sign}{whole}.{fraction:0width$}deg",
            width = usize::from(decimal_places)
        )
    };
    Ok(FormattedAngle {
        canonical_value,
        scale,
        decimal_places,
        rendered,
        rounded,
    })
}

fn split_angle_token(token: &str) -> (&str, &str) {
    if let Some(number) = token.strip_suffix('°') {
        return (number.trim_end(), "°");
    }
    let lower = token.to_ascii_lowercase();
    if lower.ends_with("deg") {
        let number_end = token.len() - 3;
        return (token[..number_end].trim_end(), &token[number_end..]);
    }
    (token, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    const TENTHS: CanonicalAngleScale = CanonicalAngleScale::new(10).unwrap();

    #[test]
    fn exact_decimal_and_scientific_degrees_share_canonical_truth() {
        for token in ["90deg", "90°", "9e1deg", "900e-1deg"] {
            assert_eq!(
                parse_decimal_degrees(token, Some(TENTHS), false)
                    .unwrap()
                    .canonical_value,
                900
            );
        }
    }

    #[test]
    fn bare_angle_requires_explicit_context_and_exact_scale() {
        assert_eq!(
            parse_decimal_degrees("90", Some(TENTHS), false)
                .unwrap_err()
                .reason,
            RefusalReason::MissingUnitContext
        );
        assert_eq!(
            parse_decimal_degrees("90", Some(TENTHS), true)
                .unwrap()
                .canonical_value,
            900
        );
        assert_eq!(
            parse_decimal_degrees("0.01deg", Some(TENTHS), false)
                .unwrap_err()
                .reason,
            RefusalReason::NonIntegralCanonicalValue
        );
    }

    #[test]
    fn formatting_reports_hidden_information() {
        let rounded = format_decimal_degrees(123, TENTHS, 0).unwrap();
        assert_eq!(rounded.rendered, "12deg");
        assert!(rounded.rounded);
        let exact = format_decimal_degrees(123, TENTHS, 1).unwrap();
        assert_eq!(exact.rendered, "12.3deg");
        assert!(!exact.rounded);
    }
}
