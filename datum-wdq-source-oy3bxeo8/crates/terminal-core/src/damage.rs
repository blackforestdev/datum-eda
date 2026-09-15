use crate::{CellPoint, Column, LimitError, PaletteIndex, PendingDamageLimit, Row};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
    seen: HashSet<Damage>,
    limit: PendingDamageLimit,
}

impl DamageSet {
    pub fn new(limit: PendingDamageLimit) -> Self {
        Self {
            entries: Vec::new(),
            seen: HashSet::new(),
            limit,
        }
    }

    pub fn push(&mut self, damage: Damage) -> Result<(), LimitError> {
        self.limit.checked_total(self.entries.len(), 1)?;
        self.entries.push(damage);
        self.seen.insert(damage);
        Ok(())
    }

    pub(crate) fn push_coalesced(&mut self, damage: Damage) {
        if self.seen.contains(&damage)
            || (self.seen.contains(&Damage::Full) && visible_damage(damage))
        {
            return;
        }
        if damage == Damage::Full {
            self.entries.retain(|entry| !visible_damage(*entry));
            self.seen.retain(|entry| !visible_damage(*entry));
        }
        if self.push(damage).is_err() {
            self.entries.clear();
            self.entries.push(Damage::Full);
            self.seen.clear();
            self.seen.insert(Damage::Full);
        }
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = Damage> + '_ {
        self.entries.iter().copied()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.seen.clear();
    }

    pub(crate) fn contains_full(&self) -> bool {
        self.seen.contains(&Damage::Full)
    }
}

const fn visible_damage(damage: Damage) -> bool {
    matches!(
        damage,
        Damage::Cell(_)
            | Damage::Row(_)
            | Damage::Rows { .. }
            | Damage::Scroll { .. }
            | Damage::Cursor
            | Damage::Full
    )
}
