//! Exact authored-length authority.
//!
//! Canonical design truth remains signed `i64` nanometers. Parsing and
//! formatting are edge projections and never mutate stored geometry. Legacy
//! floating-point adapters at the end of this module remain temporarily for
//! existing callers; UNIT-I03 owns their migration to this checked service.

/// Canonical authored length in nanometers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Nanometers(i64);

impl Nanometers {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> i64 {
        self.0
    }

    pub const fn to_le_bytes(self) -> [u8; 8] {
        self.0.to_le_bytes()
    }
}

impl From<i64> for Nanometers {
    fn from(value: i64) -> Self {
        Self::new(value)
    }
}

impl From<Nanometers> for i64 {
    fn from(value: Nanometers) -> Self {
        value.get()
    }
}

/// Physical quantity identity. Length syntax is refused for every non-length
/// kind rather than leaking into unrelated fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuantityKind {
    Length,
    Angle,
    Ratio,
    Percentage,
    Frequency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeasurementSystem {
    Metric,
    Imperial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LengthUnit {
    Nanometer,
    Micrometer,
    Millimeter,
    Mil,
    Inch,
}

impl LengthUnit {
    const fn nanometers_per_unit(self) -> i128 {
        match self {
            Self::Nanometer => 1,
            Self::Micrometer => 1_000,
            Self::Millimeter => 1_000_000,
            Self::Mil => 25_400,
            Self::Inch => 25_400_000,
        }
    }

    pub const fn measurement_system(self) -> MeasurementSystem {
        match self {
            Self::Nanometer | Self::Micrometer | Self::Millimeter => MeasurementSystem::Metric,
            Self::Mil | Self::Inch => MeasurementSystem::Imperial,
        }
    }

    pub const fn suffix(self) -> &'static str {
        match self {
            Self::Nanometer => "nm",
            Self::Micrometer => "µm",
            Self::Millimeter => "mm",
            Self::Mil => "mil",
            Self::Inch => "in",
        }
    }
}

/// Display precision affects projection only. Exact-nanometer mode always
/// renders the canonical integer with an `nm` suffix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayPrecision {
    ExactNanometer,
    DecimalPlaces(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitSource {
    ExplicitSuffix,
    Context,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverrideState {
    FollowsMeasurementSystem,
    CrossSystemOverride,
}

impl OverrideState {
    fn resolve(unit: LengthUnit, system: MeasurementSystem) -> Self {
        if unit.measurement_system() == system {
            Self::FollowsMeasurementSystem
        } else {
            Self::CrossSystemOverride
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefusalReason {
    EmptyToken,
    WrongQuantityKind,
    MissingUnitContext,
    MalformedNumber,
    AmbiguousSuffix(String),
    UnsupportedSuffix(String),
    NonIntegralNanometer,
    Overflow,
    UnsupportedPrecision(u8),
}

/// Typed provenance retained when a units request is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitsRefusal {
    pub quantity: QuantityKind,
    pub original_token: Option<String>,
    pub explicit_unit: Option<LengthUnit>,
    pub contextual_unit: Option<LengthUnit>,
    pub resolved_unit: Option<LengthUnit>,
    pub precision: DisplayPrecision,
    pub reason: RefusalReason,
}

#[derive(Debug, Clone, Copy)]
pub struct ParseLengthRequest<'a> {
    pub token: &'a str,
    pub quantity: QuantityKind,
    pub contextual_unit: Option<LengthUnit>,
    pub measurement_system: MeasurementSystem,
    pub precision: DisplayPrecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLength {
    pub quantity: QuantityKind,
    pub original_token: String,
    pub unit_source: UnitSource,
    pub explicit_unit: Option<LengthUnit>,
    pub contextual_unit: Option<LengthUnit>,
    pub resolved_unit: LengthUnit,
    pub canonical_value: Nanometers,
    pub precision: DisplayPrecision,
    pub override_state: OverrideState,
}

#[derive(Debug, Clone, Copy)]
pub struct FormatLengthRequest {
    pub value: Nanometers,
    pub quantity: QuantityKind,
    pub display_unit: LengthUnit,
    pub measurement_system: MeasurementSystem,
    pub precision: DisplayPrecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormattedLength {
    pub quantity: QuantityKind,
    pub canonical_value: Nanometers,
    pub requested_unit: LengthUnit,
    pub resolved_unit: LengthUnit,
    pub precision: DisplayPrecision,
    pub override_state: OverrideState,
    pub rendered: String,
    pub rounded: bool,
}

/// Parse a locale-independent decimal length with exact rational arithmetic.
pub fn parse_length(request: ParseLengthRequest<'_>) -> Result<ParsedLength, UnitsRefusal> {
    let original_token = request.token.to_owned();
    let refusal = |reason, explicit_unit, resolved_unit| UnitsRefusal {
        quantity: request.quantity,
        original_token: Some(original_token.clone()),
        explicit_unit,
        contextual_unit: request.contextual_unit,
        resolved_unit,
        precision: request.precision,
        reason,
    };

    if request.quantity != QuantityKind::Length {
        return Err(refusal(RefusalReason::WrongQuantityKind, None, None));
    }
    match request.precision {
        DisplayPrecision::DecimalPlaces(places) if places > 18 => {
            return Err(refusal(
                RefusalReason::UnsupportedPrecision(places),
                None,
                None,
            ));
        }
        DisplayPrecision::ExactNanometer | DisplayPrecision::DecimalPlaces(_) => {}
    }

    let token = request.token.trim();
    if token.is_empty() {
        return Err(refusal(RefusalReason::EmptyToken, None, None));
    }

    let (number, suffix) = split_number_and_suffix(token);
    if !suffix.is_empty() && !suffix.chars().all(char::is_alphabetic) {
        return Err(refusal(RefusalReason::MalformedNumber, None, None));
    }
    let explicit_unit = if suffix.is_empty() {
        None
    } else {
        Some(parse_length_suffix(suffix).map_err(|reason| refusal(reason, None, None))?)
    };
    let (resolved_unit, unit_source) = match (explicit_unit, request.contextual_unit) {
        (Some(unit), _) => (unit, UnitSource::ExplicitSuffix),
        (None, Some(unit)) => (unit, UnitSource::Context),
        (None, None) => {
            return Err(refusal(RefusalReason::MissingUnitContext, None, None));
        }
    };

    let (numerator, denominator) = parse_decimal(number)
        .map_err(|reason| refusal(reason, explicit_unit, Some(resolved_unit)))?;
    let scaled = numerator
        .checked_mul(resolved_unit.nanometers_per_unit())
        .ok_or_else(|| refusal(RefusalReason::Overflow, explicit_unit, Some(resolved_unit)))?;
    if scaled % denominator != 0 {
        return Err(refusal(
            RefusalReason::NonIntegralNanometer,
            explicit_unit,
            Some(resolved_unit),
        ));
    }
    let exact = scaled / denominator;
    let canonical = i64::try_from(exact)
        .map(Nanometers::new)
        .map_err(|_| refusal(RefusalReason::Overflow, explicit_unit, Some(resolved_unit)))?;

    Ok(ParsedLength {
        quantity: request.quantity,
        original_token,
        unit_source,
        explicit_unit,
        contextual_unit: request.contextual_unit,
        resolved_unit,
        canonical_value: canonical,
        precision: request.precision,
        override_state: OverrideState::resolve(resolved_unit, request.measurement_system),
    })
}

/// Format a canonical length without changing the canonical value. Decimal
/// formatting rounds half away from zero; the result reports whether any
/// canonical remainder was rounded away.
pub fn format_length(request: FormatLengthRequest) -> Result<FormattedLength, UnitsRefusal> {
    let refusal = |reason, resolved_unit| UnitsRefusal {
        quantity: request.quantity,
        original_token: None,
        explicit_unit: None,
        contextual_unit: Some(request.display_unit),
        resolved_unit,
        precision: request.precision,
        reason,
    };
    if request.quantity != QuantityKind::Length {
        return Err(refusal(RefusalReason::WrongQuantityKind, None));
    }

    let (resolved_unit, rendered, rounded) = match request.precision {
        DisplayPrecision::ExactNanometer => (
            LengthUnit::Nanometer,
            format!("{}nm", request.value.get()),
            false,
        ),
        DisplayPrecision::DecimalPlaces(places) => {
            if places > 18 {
                return Err(refusal(
                    RefusalReason::UnsupportedPrecision(places),
                    Some(request.display_unit),
                ));
            }
            let (number, rounded) = format_decimal(
                request.value,
                request.display_unit.nanometers_per_unit(),
                places,
            )
            .ok_or_else(|| refusal(RefusalReason::Overflow, Some(request.display_unit)))?;
            (
                request.display_unit,
                format!("{number}{}", request.display_unit.suffix()),
                rounded,
            )
        }
    };

    Ok(FormattedLength {
        quantity: request.quantity,
        canonical_value: request.value,
        requested_unit: request.display_unit,
        resolved_unit,
        precision: request.precision,
        override_state: OverrideState::resolve(resolved_unit, request.measurement_system),
        rendered,
        rounded,
    })
}

fn split_number_and_suffix(token: &str) -> (&str, &str) {
    let mut end = 0;
    for (index, character) in token.char_indices() {
        if character.is_ascii_digit() || matches!(character, '+' | '-' | '.') {
            end = index + character.len_utf8();
        } else {
            break;
        }
    }
    (token[..end].trim(), token[end..].trim())
}

fn parse_length_suffix(suffix: &str) -> Result<LengthUnit, RefusalReason> {
    let normalized = suffix.to_lowercase();
    match normalized.as_str() {
        "nm" => Ok(LengthUnit::Nanometer),
        "µm" | "um" => Ok(LengthUnit::Micrometer),
        "mm" => Ok(LengthUnit::Millimeter),
        "mil" => Ok(LengthUnit::Mil),
        "in" => Ok(LengthUnit::Inch),
        "m" => Err(RefusalReason::AmbiguousSuffix(suffix.to_owned())),
        _ => Err(RefusalReason::UnsupportedSuffix(suffix.to_owned())),
    }
}

fn parse_decimal(number: &str) -> Result<(i128, i128), RefusalReason> {
    if number.is_empty() {
        return Err(RefusalReason::MalformedNumber);
    }
    let (negative, unsigned) = match number.as_bytes()[0] {
        b'-' => (true, &number[1..]),
        b'+' => (false, &number[1..]),
        _ => (false, number),
    };
    if unsigned.is_empty() {
        return Err(RefusalReason::MalformedNumber);
    }

    let mut numerator = 0_i128;
    let mut denominator = 1_i128;
    let mut saw_digit = false;
    let mut saw_dot = false;
    for byte in unsigned.bytes() {
        match byte {
            b'0'..=b'9' => {
                saw_digit = true;
                numerator = numerator
                    .checked_mul(10)
                    .and_then(|value| value.checked_add(i128::from(byte - b'0')))
                    .ok_or(RefusalReason::Overflow)?;
                if saw_dot {
                    denominator = denominator.checked_mul(10).ok_or(RefusalReason::Overflow)?;
                }
            }
            b'.' if !saw_dot => saw_dot = true,
            _ => return Err(RefusalReason::MalformedNumber),
        }
    }
    if !saw_digit {
        return Err(RefusalReason::MalformedNumber);
    }
    if negative {
        numerator = numerator.checked_neg().ok_or(RefusalReason::Overflow)?;
    }
    Ok((numerator, denominator))
}

fn format_decimal(value: Nanometers, unit_nm: i128, places: u8) -> Option<(String, bool)> {
    let scale = 10_i128.checked_pow(u32::from(places))?;
    let magnitude = i128::from(value.get()).abs();
    let scaled = magnitude.checked_mul(scale)?;
    let quotient = scaled / unit_nm;
    let remainder = scaled % unit_nm;
    let rounded = remainder != 0;
    let magnitude_at_precision = if remainder.checked_mul(2)? >= unit_nm {
        quotient.checked_add(1)?
    } else {
        quotient
    };
    let sign = if value.get() < 0 { "-" } else { "" };
    if places == 0 {
        return Some((format!("{sign}{magnitude_at_precision}"), rounded));
    }
    let whole = magnitude_at_precision / scale;
    let fraction = magnitude_at_precision % scale;
    Some((
        format!(
            "{sign}{whole}.{fraction:0width$}",
            width = usize::from(places)
        ),
        rounded,
    ))
}

/// Legacy conversion adapter. New authored-input paths must use
/// [`parse_length`] so overflow and non-integral nanometers are refused.
/// Convert millimeters to nanometers.
pub fn mm_to_nm(mm: f64) -> i64 {
    (mm * 1_000_000.0).round() as i64
}

/// Convert nanometers to millimeters.
pub fn nm_to_mm(nm: i64) -> f64 {
    nm as f64 / 1_000_000.0
}

/// Convert mils (thousandths of inch) to nanometers.
pub fn mil_to_nm(mil: f64) -> i64 {
    (mil * 25_400.0).round() as i64
}

/// Convert nanometers to mils.
pub fn nm_to_mil(nm: i64) -> f64 {
    nm as f64 / 25_400.0
}

/// Convert inches to nanometers.
pub fn inch_to_nm(inch: f64) -> i64 {
    (inch * 25_400_000.0).round() as i64
}

/// Angle: tenths of degree. 0 = right, 900 = up, 1800 = left, 2700 = down.
pub type AngleTenths = i32;

/// Normalize angle to 0..3599 range.
pub fn normalize_angle(a: AngleTenths) -> AngleTenths {
    a.rem_euclid(3600)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn parse(token: &str) -> Result<ParsedLength, UnitsRefusal> {
        parse_length(ParseLengthRequest {
            token,
            quantity: QuantityKind::Length,
            contextual_unit: None,
            measurement_system: MeasurementSystem::Metric,
            precision: DisplayPrecision::DecimalPlaces(6),
        })
    }

    #[test]
    fn exact_metric_and_imperial_inputs_share_one_value() {
        for token in ["5.08mm", "200mil", "0.2in", "5080000nm"] {
            assert_eq!(parse(token).unwrap().canonical_value.get(), 5_080_000);
        }
    }

    #[test]
    fn parses_signs_whitespace_case_and_micrometer_aliases() {
        assert_eq!(
            parse(" -2.5 MM ").unwrap().canonical_value.get(),
            -2_500_000
        );
        assert_eq!(parse("10µm").unwrap().canonical_value.get(), 10_000);
        assert_eq!(parse("10UM").unwrap().canonical_value.get(), 10_000);
        assert_eq!(parse(".001in").unwrap().canonical_value.get(), 25_400);
    }

    #[test]
    fn contextual_bare_number_names_its_source_and_override() {
        let parsed = parse_length(ParseLengthRequest {
            token: "200",
            quantity: QuantityKind::Length,
            contextual_unit: Some(LengthUnit::Mil),
            measurement_system: MeasurementSystem::Metric,
            precision: DisplayPrecision::DecimalPlaces(3),
        })
        .unwrap();
        assert_eq!(parsed.unit_source, UnitSource::Context);
        assert_eq!(parsed.explicit_unit, None);
        assert_eq!(parsed.contextual_unit, Some(LengthUnit::Mil));
        assert_eq!(parsed.resolved_unit, LengthUnit::Mil);
        assert_eq!(parsed.canonical_value.get(), 5_080_000);
        assert_eq!(parsed.override_state, OverrideState::CrossSystemOverride);
    }

    #[test]
    fn explicit_suffix_wins_without_hiding_cross_system_override() {
        let parsed = parse_length(ParseLengthRequest {
            token: "5mm",
            quantity: QuantityKind::Length,
            contextual_unit: Some(LengthUnit::Inch),
            measurement_system: MeasurementSystem::Imperial,
            precision: DisplayPrecision::DecimalPlaces(3),
        })
        .unwrap();
        assert_eq!(parsed.unit_source, UnitSource::ExplicitSuffix);
        assert_eq!(parsed.explicit_unit, Some(LengthUnit::Millimeter));
        assert_eq!(parsed.contextual_unit, Some(LengthUnit::Inch));
        assert_eq!(parsed.override_state, OverrideState::CrossSystemOverride);
    }

    #[test]
    fn refuses_non_integral_overflow_and_signed_boundary_values_exactly() {
        assert_eq!(
            parse("0.1nm").unwrap_err().reason,
            RefusalReason::NonIntegralNanometer
        );
        assert_eq!(
            parse("9223372036854775808nm").unwrap_err().reason,
            RefusalReason::Overflow
        );
        assert_eq!(
            parse("-9223372036854775808nm")
                .unwrap()
                .canonical_value
                .get(),
            i64::MIN
        );
        assert_eq!(
            parse("9223372036854775807nm")
                .unwrap()
                .canonical_value
                .get(),
            i64::MAX
        );
    }

    #[test]
    fn refusal_provenance_distinguishes_missing_ambiguous_and_malformed_input() {
        let missing = parse("5").unwrap_err();
        assert_eq!(missing.reason, RefusalReason::MissingUnitContext);
        assert_eq!(missing.original_token.as_deref(), Some("5"));

        assert_eq!(
            parse("5m").unwrap_err().reason,
            RefusalReason::AmbiguousSuffix("m".to_owned())
        );
        for token in ["1,5mm", "1e3mm", "--1mm", "mm", "1..0mm"] {
            assert_eq!(
                parse(token).unwrap_err().reason,
                RefusalReason::MalformedNumber,
                "token {token}"
            );
        }
        assert_eq!(
            parse("1px").unwrap_err().reason,
            RefusalReason::UnsupportedSuffix("px".to_owned())
        );
    }

    #[test]
    fn refuses_length_syntax_for_other_quantity_kinds() {
        let refusal = parse_length(ParseLengthRequest {
            token: "90mm",
            quantity: QuantityKind::Angle,
            contextual_unit: None,
            measurement_system: MeasurementSystem::Metric,
            precision: DisplayPrecision::DecimalPlaces(1),
        })
        .unwrap_err();
        assert_eq!(refusal.reason, RefusalReason::WrongQuantityKind);
        assert_eq!(refusal.quantity, QuantityKind::Angle);
    }

    #[test]
    fn exact_format_and_parse_round_trips_are_deterministic() {
        let cases = [
            (LengthUnit::Millimeter, 6, "5.080000mm"),
            (LengthUnit::Mil, 3, "200.000mil"),
            (LengthUnit::Inch, 8, "0.20000000in"),
        ];
        for (unit, places, expected) in cases {
            let formatted = format_length(FormatLengthRequest {
                value: Nanometers::new(5_080_000),
                quantity: QuantityKind::Length,
                display_unit: unit,
                measurement_system: unit.measurement_system(),
                precision: DisplayPrecision::DecimalPlaces(places),
            })
            .unwrap();
            assert_eq!(formatted.rendered, expected);
            assert!(!formatted.rounded);
            assert_eq!(
                parse(&formatted.rendered).unwrap().canonical_value,
                formatted.canonical_value
            );
        }
    }

    #[test]
    fn display_rounding_and_system_changes_never_rewrite_stored_bytes() {
        let stored = Nanometers::new(-1_234_567);
        let before = stored.to_le_bytes();
        let rounded = format_length(FormatLengthRequest {
            value: stored,
            quantity: QuantityKind::Length,
            display_unit: LengthUnit::Millimeter,
            measurement_system: MeasurementSystem::Imperial,
            precision: DisplayPrecision::DecimalPlaces(2),
        })
        .unwrap();
        assert_eq!(rounded.rendered, "-1.23mm");
        assert!(rounded.rounded);
        assert_eq!(rounded.override_state, OverrideState::CrossSystemOverride);
        assert_eq!(rounded.canonical_value.to_le_bytes(), before);

        let exact = format_length(FormatLengthRequest {
            value: stored,
            quantity: QuantityKind::Length,
            display_unit: LengthUnit::Inch,
            measurement_system: MeasurementSystem::Imperial,
            precision: DisplayPrecision::ExactNanometer,
        })
        .unwrap();
        assert_eq!(exact.rendered, "-1234567nm");
        assert_eq!(exact.resolved_unit, LengthUnit::Nanometer);
        assert_eq!(exact.canonical_value.to_le_bytes(), before);
    }

    #[test]
    fn unsupported_precision_is_a_typed_refusal() {
        let refusal = format_length(FormatLengthRequest {
            value: Nanometers::new(1),
            quantity: QuantityKind::Length,
            display_unit: LengthUnit::Inch,
            measurement_system: MeasurementSystem::Imperial,
            precision: DisplayPrecision::DecimalPlaces(19),
        })
        .unwrap_err();
        assert_eq!(refusal.reason, RefusalReason::UnsupportedPrecision(19));

        let refusal = parse_length(ParseLengthRequest {
            token: "1mm",
            quantity: QuantityKind::Length,
            contextual_unit: None,
            measurement_system: MeasurementSystem::Metric,
            precision: DisplayPrecision::DecimalPlaces(19),
        })
        .unwrap_err();
        assert_eq!(refusal.reason, RefusalReason::UnsupportedPrecision(19));
    }

    proptest! {
        #[test]
        fn every_canonical_i64_round_trips_through_exact_nanometer_text(value in any::<i64>()) {
            let formatted = format_length(FormatLengthRequest {
                value: Nanometers::new(value),
                quantity: QuantityKind::Length,
                display_unit: LengthUnit::Inch,
                measurement_system: MeasurementSystem::Imperial,
                precision: DisplayPrecision::ExactNanometer,
            })
            .unwrap();
            let reparsed = parse(&formatted.rendered).unwrap();
            prop_assert_eq!(reparsed.canonical_value, Nanometers::new(value));
            prop_assert_eq!(formatted.canonical_value.to_le_bytes(), value.to_le_bytes());
        }
    }

    #[test]
    fn mm_round_trip() {
        assert_eq!(mm_to_nm(1.0), 1_000_000);
        assert_eq!(mm_to_nm(0.1), 100_000);
        assert_eq!(mm_to_nm(0.001), 1_000);
        assert!((nm_to_mm(mm_to_nm(2.54)) - 2.54).abs() < 1e-10);
    }

    #[test]
    fn mil_round_trip() {
        assert_eq!(mil_to_nm(1.0), 25_400);
        assert_eq!(mil_to_nm(10.0), 254_000);
        assert!((nm_to_mil(mil_to_nm(100.0)) - 100.0).abs() < 1e-10);
    }

    #[test]
    fn angle_normalize() {
        assert_eq!(normalize_angle(0), 0);
        assert_eq!(normalize_angle(900), 900);
        assert_eq!(normalize_angle(3600), 0);
        assert_eq!(normalize_angle(-900), 2700);
        assert_eq!(normalize_angle(-3600), 0);
    }
}
