/// Unit conversion utilities.
/// Internal unit: nanometers (i64). See docs/CANONICAL_IR.md §2.
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
