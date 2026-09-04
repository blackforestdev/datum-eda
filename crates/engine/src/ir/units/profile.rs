use super::{DisplayPrecision, LengthUnit, MeasurementSystem, OverrideState, RefusalReason};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LengthQuantity {
    BoardLayout,
    DrillHole,
    SchematicGeometry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LengthUnitChoice {
    FollowSystem,
    Explicit(LengthUnit),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LengthPrecisionChoice {
    Automatic,
    DecimalPlaces(u8),
    ExactNanometer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecimalDegreePrecision {
    Decimal0,
    Decimal1,
    Decimal2,
    Decimal3,
}

impl DecimalDegreePrecision {
    pub const fn places(self) -> u8 {
        match self {
            Self::Decimal0 => 0,
            Self::Decimal1 => 1,
            Self::Decimal2 => 2,
            Self::Decimal3 => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuantityUnits {
    pub unit: LengthUnitChoice,
    pub precision: LengthPrecisionChoice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnitsProfile {
    pub system: MeasurementSystem,
    pub board: QuantityUnits,
    pub drill: QuantityUnits,
    pub schematic: QuantityUnits,
    pub angle_precision: DecimalDegreePrecision,
}

impl Default for UnitsProfile {
    fn default() -> Self {
        let defaults = QuantityUnits {
            unit: LengthUnitChoice::FollowSystem,
            precision: LengthPrecisionChoice::Automatic,
        };
        Self {
            system: MeasurementSystem::Metric,
            board: defaults,
            drill: defaults,
            schematic: defaults,
            angle_precision: DecimalDegreePrecision::Decimal1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResolvedQuantityUnits {
    pub unit: LengthUnit,
    pub precision: DisplayPrecision,
    pub override_state: OverrideState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResolvedUnitsProfile {
    pub system: MeasurementSystem,
    pub board: ResolvedQuantityUnits,
    pub drill: ResolvedQuantityUnits,
    pub schematic: ResolvedQuantityUnits,
    pub angle_precision: DecimalDegreePrecision,
}

impl UnitsProfile {
    pub fn resolve(self) -> Result<ResolvedUnitsProfile, RefusalReason> {
        Ok(ResolvedUnitsProfile {
            system: self.system,
            board: resolve_quantity(LengthQuantity::BoardLayout, self.board, self.system)?,
            drill: resolve_quantity(LengthQuantity::DrillHole, self.drill, self.system)?,
            schematic: resolve_quantity(
                LengthQuantity::SchematicGeometry,
                self.schematic,
                self.system,
            )?,
            angle_precision: self.angle_precision,
        })
    }
}

fn resolve_quantity(
    quantity: LengthQuantity,
    setting: QuantityUnits,
    system: MeasurementSystem,
) -> Result<ResolvedQuantityUnits, RefusalReason> {
    let unit = match setting.unit {
        LengthUnitChoice::FollowSystem => follow_system_unit(quantity, system),
        LengthUnitChoice::Explicit(unit) if unit_allowed(quantity, unit) => unit,
        LengthUnitChoice::Explicit(unit) => {
            return Err(RefusalReason::UnitNotAllowedForQuantity { quantity, unit });
        }
    };
    let precision = match setting.precision {
        LengthPrecisionChoice::Automatic => automatic_precision(quantity, unit)?,
        LengthPrecisionChoice::DecimalPlaces(places @ 0..=6) => {
            DisplayPrecision::DecimalPlaces(places)
        }
        LengthPrecisionChoice::DecimalPlaces(places) => {
            return Err(RefusalReason::UnsupportedPrecision(places));
        }
        LengthPrecisionChoice::ExactNanometer => DisplayPrecision::ExactNanometer,
    };
    let override_state = match setting.unit {
        LengthUnitChoice::FollowSystem => OverrideState::FollowsMeasurementSystem,
        LengthUnitChoice::Explicit(_) if unit.measurement_system() == system => {
            OverrideState::ExplicitSameSystem
        }
        LengthUnitChoice::Explicit(_) => OverrideState::CrossSystemOverride,
    };
    Ok(ResolvedQuantityUnits {
        unit,
        precision,
        override_state,
    })
}

pub const fn follow_system_unit(quantity: LengthQuantity, system: MeasurementSystem) -> LengthUnit {
    match (quantity, system) {
        (
            LengthQuantity::BoardLayout | LengthQuantity::SchematicGeometry,
            MeasurementSystem::Metric,
        )
        | (LengthQuantity::DrillHole, MeasurementSystem::Metric) => LengthUnit::Millimeter,
        (
            LengthQuantity::BoardLayout | LengthQuantity::SchematicGeometry,
            MeasurementSystem::Imperial,
        ) => LengthUnit::Mil,
        (LengthQuantity::DrillHole, MeasurementSystem::Imperial) => LengthUnit::Inch,
    }
}

pub const fn unit_allowed(quantity: LengthQuantity, unit: LengthUnit) -> bool {
    match quantity {
        LengthQuantity::BoardLayout => matches!(
            unit,
            LengthUnit::Millimeter | LengthUnit::Micrometer | LengthUnit::Mil | LengthUnit::Inch
        ),
        LengthQuantity::DrillHole => {
            matches!(
                unit,
                LengthUnit::Millimeter | LengthUnit::Mil | LengthUnit::Inch
            )
        }
        LengthQuantity::SchematicGeometry => {
            matches!(unit, LengthUnit::Millimeter | LengthUnit::Mil)
        }
    }
}

pub const fn automatic_precision(
    quantity: LengthQuantity,
    unit: LengthUnit,
) -> Result<DisplayPrecision, RefusalReason> {
    let places = match (quantity, unit) {
        (LengthQuantity::BoardLayout, LengthUnit::Millimeter) => 3,
        (LengthQuantity::BoardLayout, LengthUnit::Micrometer | LengthUnit::Mil) => 1,
        (LengthQuantity::BoardLayout, LengthUnit::Inch) => 5,
        (LengthQuantity::DrillHole, LengthUnit::Millimeter) => 3,
        (LengthQuantity::DrillHole, LengthUnit::Mil) => 1,
        (LengthQuantity::DrillHole, LengthUnit::Inch) => 4,
        (LengthQuantity::SchematicGeometry, LengthUnit::Millimeter) => 2,
        (LengthQuantity::SchematicGeometry, LengthUnit::Mil) => 0,
        _ => return Err(RefusalReason::UnitNotAllowedForQuantity { quantity, unit }),
    };
    Ok(DisplayPrecision::DecimalPlaces(places))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_automatic_cell_matches_the_contract() {
        let cases = [
            (LengthQuantity::BoardLayout, LengthUnit::Millimeter, 3),
            (LengthQuantity::BoardLayout, LengthUnit::Micrometer, 1),
            (LengthQuantity::BoardLayout, LengthUnit::Mil, 1),
            (LengthQuantity::BoardLayout, LengthUnit::Inch, 5),
            (LengthQuantity::DrillHole, LengthUnit::Millimeter, 3),
            (LengthQuantity::DrillHole, LengthUnit::Mil, 1),
            (LengthQuantity::DrillHole, LengthUnit::Inch, 4),
            (LengthQuantity::SchematicGeometry, LengthUnit::Millimeter, 2),
            (LengthQuantity::SchematicGeometry, LengthUnit::Mil, 0),
        ];
        for (quantity, unit, places) in cases {
            assert_eq!(
                automatic_precision(quantity, unit).unwrap(),
                DisplayPrecision::DecimalPlaces(places)
            );
        }
    }

    #[test]
    fn follow_system_and_cross_system_override_are_distinct() {
        let mut profile = UnitsProfile::default();
        profile.board.unit = LengthUnitChoice::Explicit(LengthUnit::Mil);
        let resolved = profile.resolve().unwrap();
        assert_eq!(resolved.board.unit, LengthUnit::Mil);
        assert_eq!(resolved.board.precision, DisplayPrecision::DecimalPlaces(1));
        assert_eq!(
            resolved.board.override_state,
            OverrideState::CrossSystemOverride
        );
        assert_eq!(
            resolved.drill.override_state,
            OverrideState::FollowsMeasurementSystem
        );
    }

    #[test]
    fn invalid_quantity_unit_pairs_refuse() {
        let mut profile = UnitsProfile::default();
        profile.schematic.unit = LengthUnitChoice::Explicit(LengthUnit::Inch);
        assert_eq!(
            profile.resolve().unwrap_err(),
            RefusalReason::UnitNotAllowedForQuantity {
                quantity: LengthQuantity::SchematicGeometry,
                unit: LengthUnit::Inch,
            }
        );
    }

    #[test]
    fn explicit_precision_survives_unit_and_system_changes() {
        let explicit = LengthPrecisionChoice::DecimalPlaces(6);
        let mut profile = UnitsProfile::default();
        profile.board.precision = explicit;
        profile.board.unit = LengthUnitChoice::Explicit(LengthUnit::Inch);
        assert_eq!(
            profile.resolve().unwrap().board.precision,
            DisplayPrecision::DecimalPlaces(6)
        );
        profile.system = MeasurementSystem::Imperial;
        profile.board.unit = LengthUnitChoice::Explicit(LengthUnit::Millimeter);
        assert_eq!(profile.board.precision, explicit);
        assert_eq!(
            profile.resolve().unwrap().board.precision,
            DisplayPrecision::DecimalPlaces(6)
        );
    }
}
