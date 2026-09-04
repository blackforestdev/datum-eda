use super::{
    DisplayPrecision, FormatLengthRequest, LengthUnit, MeasurementSystem, Nanometers,
    ParseLengthRequest, QuantityKind, UnitsRefusal, format_length, parse_length,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditCommit {
    Unchanged,
    Changed(Nanometers),
}

/// Transient numeric editing state. Display text is deliberately not accepted
/// as input: focus always derives an exact buffer from canonical geometry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LengthEditSession {
    original: Nanometers,
    unit: LengthUnit,
    measurement_system: MeasurementSystem,
    buffer: String,
}

impl LengthEditSession {
    pub fn begin(
        canonical: Nanometers,
        unit: LengthUnit,
        measurement_system: MeasurementSystem,
    ) -> Self {
        let buffer = exact_edit_text(canonical, unit);
        Self {
            original: canonical,
            unit,
            measurement_system,
            buffer,
        }
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn replace_buffer(&mut self, value: impl Into<String>) {
        self.buffer = value.into();
    }

    pub fn cancel(self) -> Nanometers {
        self.original
    }

    pub fn commit(self) -> Result<EditCommit, UnitsRefusal> {
        let parsed = parse_length(ParseLengthRequest {
            quantity: QuantityKind::Length,
            token: &self.buffer,
            contextual_unit: Some(self.unit),
            measurement_system: self.measurement_system,
            precision: DisplayPrecision::ExactNanometer,
        })?;
        Ok(if parsed.canonical_value == self.original {
            EditCommit::Unchanged
        } else {
            EditCommit::Changed(parsed.canonical_value)
        })
    }
}

fn exact_edit_text(canonical: Nanometers, unit: LengthUnit) -> String {
    format_length(FormatLengthRequest {
        quantity: QuantityKind::Length,
        value: canonical,
        display_unit: unit,
        measurement_system: unit.measurement_system(),
        precision: DisplayPrecision::DecimalPlaces(6),
    })
    .ok()
    .filter(|formatted| !formatted.rounded)
    .map(|formatted| trim_exact_fraction(&formatted.rendered, unit.suffix()))
    .unwrap_or_else(|| format!("{}nm", canonical.get()))
}

fn trim_exact_fraction(rendered: &str, suffix: &str) -> String {
    let number = rendered.strip_suffix(suffix).unwrap_or(rendered);
    let trimmed = number.trim_end_matches('0').trim_end_matches('.');
    format!("{trimmed}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_uses_canonical_truth_and_unchanged_commit_is_noop() {
        let session = LengthEditSession::begin(
            Nanometers::new(5_080_001),
            LengthUnit::Millimeter,
            MeasurementSystem::Metric,
        );
        assert_eq!(session.buffer(), "5.080001mm");
        assert_eq!(session.commit().unwrap(), EditCommit::Unchanged);
    }

    #[test]
    fn non_finite_unit_representation_falls_back_to_exact_nm() {
        let session = LengthEditSession::begin(
            Nanometers::new(1),
            LengthUnit::Mil,
            MeasurementSystem::Imperial,
        );
        assert_eq!(session.buffer(), "1nm");
        assert_eq!(session.cancel(), Nanometers::new(1));
    }

    #[test]
    fn edited_commit_parses_once_and_reports_only_canonical_change() {
        let mut session = LengthEditSession::begin(
            Nanometers::new(5_080_000),
            LengthUnit::Mil,
            MeasurementSystem::Imperial,
        );
        assert_eq!(session.buffer(), "200mil");
        session.replace_buffer("0.2in");
        assert_eq!(session.commit().unwrap(), EditCommit::Unchanged);
        let mut session = LengthEditSession::begin(
            Nanometers::new(5_080_000),
            LengthUnit::Mil,
            MeasurementSystem::Imperial,
        );
        session.replace_buffer("201mil");
        assert_eq!(
            session.commit().unwrap(),
            EditCommit::Changed(Nanometers::new(5_105_400))
        );
    }
}
