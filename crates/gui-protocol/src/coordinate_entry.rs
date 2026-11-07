//! Exact coordinate form data; input, paint and canonical writes retain their owners.
use crate::{PointNm, single_line_editor::SingleLineEditor};
use eda_engine::ir::units::{
    GuiProjectLengthInput, LengthEditSession, LengthQuantity, Nanometers, ResolvedUnitsProfile,
    resolve_gui_project_length,
};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinateEntryMode {
    Absolute,
    Delta,
}

/// Unit context and reference are captured once. Editing public field buffers
/// uses the existing shared SingleLineEditor, never another numeric controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoordinateEntry {
    project: Uuid,
    quantity: LengthQuantity,
    units: ResolvedUnitsProfile,
    reference: PointNm,
    current: PointNm,
    mode: CoordinateEntryMode,
    pub x: String,
    pub y: String,
    pub x_editor: SingleLineEditor,
    pub y_editor: SingleLineEditor,
}
impl CoordinateEntry {
    pub fn new(
        project: Uuid,
        quantity: LengthQuantity,
        units: ResolvedUnitsProfile,
        reference: PointNm,
        current: PointNm,
        mode: CoordinateEntryMode,
    ) -> Result<Self, String> {
        let mut entry = Self {
            project,
            quantity,
            units,
            reference,
            current,
            mode,
            x: String::new(),
            y: String::new(),
            x_editor: SingleLineEditor::default(),
            y_editor: SingleLineEditor::default(),
        };
        entry.set_destination(current)?;
        Ok(entry)
    }
    pub fn mode(&self) -> CoordinateEntryMode {
        self.mode
    }
    pub fn reference(&self) -> PointNm {
        self.reference
    }
    /// Last destination installed by the caller or a successful mode conversion.
    /// Partially edited buffers are parsed only through `destination`.
    pub fn current(&self) -> PointNm {
        self.current
    }
    pub fn project(&self) -> Uuid {
        self.project
    }
    pub fn quantity(&self) -> LengthQuantity {
        self.quantity
    }
    pub fn units(&self) -> ResolvedUnitsProfile {
        self.units
    }

    pub fn destination(&self) -> Result<PointNm, String> {
        let parse = |token: &str, field_id: &str, axis: &str| {
            resolve_gui_project_length(GuiProjectLengthInput {
                token,
                project_id: &self.project.to_string(),
                field_id,
                quantity: self.quantity,
                units: self.units,
            })
            .map(|result| result.canonical_nm)
            .map_err(|error| format!("Invalid {axis} coordinate: {error}"))
        };
        let x = parse(&self.x, "coordinate.x", "X")?;
        let y = parse(&self.y, "coordinate.y", "Y")?;
        match self.mode {
            CoordinateEntryMode::Absolute => Ok(PointNm { x, y }),
            CoordinateEntryMode::Delta => Ok(PointNm {
                x: checked(i128::from(self.reference.x) + i128::from(x))?,
                y: checked(i128::from(self.reference.y) + i128::from(y))?,
            }),
        }
    }

    /// Validate both axes and conversions before replacing any field or caret.
    pub fn set_mode(&mut self, mode: CoordinateEntryMode) -> Result<(), String> {
        let current = self.destination()?;
        if self.mode == mode {
            return Ok(());
        }
        let (x, y) = self.fields(current, mode)?;
        self.install(current, mode, x, y);
        Ok(())
    }
    pub fn set_destination(&mut self, current: PointNm) -> Result<(), String> {
        let (x, y) = self.fields(current, self.mode)?;
        self.install(current, self.mode, x, y);
        Ok(())
    }
    fn fields(
        &self,
        current: PointNm,
        mode: CoordinateEntryMode,
    ) -> Result<(String, String), String> {
        let point = match mode {
            CoordinateEntryMode::Absolute => current,
            CoordinateEntryMode::Delta => PointNm {
                x: checked(i128::from(current.x) - i128::from(self.reference.x))?,
                y: checked(i128::from(current.y) - i128::from(self.reference.y))?,
            },
        };
        let unit = match self.quantity {
            LengthQuantity::BoardLayout => self.units.board.unit,
            LengthQuantity::DrillHole => self.units.drill.unit,
            LengthQuantity::SchematicGeometry => self.units.schematic.unit,
        };
        let text = |value| {
            LengthEditSession::begin(Nanometers::new(value), unit, self.units.system)
                .buffer()
                .to_owned()
        };
        Ok((text(point.x), text(point.y)))
    }
    fn install(&mut self, current: PointNm, mode: CoordinateEntryMode, x: String, y: String) {
        self.x_editor = SingleLineEditor::at_end(&x);
        self.y_editor = SingleLineEditor::at_end(&y);
        self.x = x;
        self.y = y;
        self.current = current;
        self.mode = mode;
    }
}
fn checked(value: i128) -> Result<i64, String> {
    i64::try_from(value).map_err(|_| "Coordinate exceeds integer nanometers".into())
}

#[cfg(test)]
#[path = "coordinate_entry_tests.rs"]
mod tests;
