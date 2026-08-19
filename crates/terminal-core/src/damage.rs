use crate::{CellPoint, Column, LimitError, PaletteIndex, PendingDamageLimit, Row};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Damage {
    Cell(CellPoint),
    Row(Row),
    Rows {
        first: Row,
        last: Row,
    },
    Scroll {
        first_row: Row,
        last_row: Row,
        first_column: Column,
        last_column: Column,
        direction: ScrollDirection,
        count: u16,
    },
    Cursor,
    Palette(PaletteIndex),
    Title,
    History,
    Graphics,
    Full,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DamageSet {
    entries: Vec<Damage>,
    limit: PendingDamageLimit,
}

impl DamageSet {
    pub fn new(limit: PendingDamageLimit) -> Self {
        Self {
            entries: Vec::new(),
            limit,
        }
    }

    pub fn push(&mut self, damage: Damage) -> Result<(), LimitError> {
        self.limit.checked_total(self.entries.len(), 1)?;
        self.entries.push(damage);
        Ok(())
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = Damage> + '_ {
        self.entries.iter().copied()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
