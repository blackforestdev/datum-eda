use crate::error::EngineError;

pub(super) fn checked_mm_to_nm(mm: f64) -> Result<i64, EngineError> {
    crate::ir::units::checked_f64_length(mm, crate::ir::units::LengthUnit::Millimeter)
        .map(|value| value.get())
        .map_err(|refusal| {
            EngineError::Import(format!(
                "KiCad millimeter value {mm:?} is not exactly representable: {:?}",
                refusal.reason
            ))
        })
}

pub(super) fn parse_kicad_rotation(token: &str) -> Option<i32> {
    crate::ir::units::parse_decimal_degrees(
        token,
        crate::ir::units::CanonicalAngleScale::new(1),
        true,
    )
    .ok()
    .and_then(|parsed| i32::try_from(parsed.canonical_value).ok())
}
