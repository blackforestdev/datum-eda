#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CharacterSet {
    #[default]
    Ascii,
    DecSpecialGraphics,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CharacterSetSlot {
    #[default]
    G0,
    G1,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CharacterSetState {
    pub g0: CharacterSet,
    pub g1: CharacterSet,
    pub active: CharacterSetSlot,
}

impl CharacterSetState {
    pub const fn active_set(self) -> CharacterSet {
        match self.active {
            CharacterSetSlot::G0 => self.g0,
            CharacterSetSlot::G1 => self.g1,
        }
    }

    pub fn designate(&mut self, slot: CharacterSetSlot, set: CharacterSet) {
        match slot {
            CharacterSetSlot::G0 => self.g0 = set,
            CharacterSetSlot::G1 => self.g1 = set,
        }
    }

    pub fn map(self, character: char) -> char {
        if self.active_set() != CharacterSet::DecSpecialGraphics {
            return character;
        }
        match character {
            '`' => '◆',
            'a' => '▒',
            'b' => '␉',
            'c' => '␌',
            'd' => '␍',
            'e' => '␊',
            'f' => '°',
            'g' => '±',
            'h' => '␤',
            'i' => '␋',
            'j' => '┘',
            'k' => '┐',
            'l' => '┌',
            'm' => '└',
            'n' => '┼',
            'o' => '⎺',
            'p' => '⎻',
            'q' => '─',
            'r' => '⎼',
            's' => '⎽',
            't' => '├',
            'u' => '┤',
            'v' => '┴',
            'w' => '┬',
            'x' => '│',
            'y' => '≤',
            'z' => '≥',
            '{' => 'π',
            '|' => '≠',
            '}' => '£',
            '~' => '·',
            _ => character,
        }
    }
}
