use crate::search::{SearchError, WorkMeter};

const UNSET: usize = usize::MAX;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Matcher {
    Any,
    Literal(String),
    Class {
        negated: bool,
        ranges: Vec<(char, char)>,
    },
}

impl Matcher {
    fn accepts(&self, text: &str, case_sensitive: bool) -> bool {
        match self {
            Self::Any => text != "\n",
            Self::Literal(expected) => equal(expected, text, case_sensitive),
            Self::Class { negated, ranges } => {
                let Some(character) = text.chars().next() else {
                    return false;
                };
                let character = fold_char(character, case_sensitive);
                let contained = ranges.iter().any(|(start, end)| {
                    let start = fold_char(*start, case_sensitive);
                    let end = fold_char(*end, case_sensitive);
                    start <= character && character <= end
                });
                contained != *negated
            }
        }
    }
}

fn fold_char(character: char, case_sensitive: bool) -> char {
    if case_sensitive {
        character
    } else {
        character.to_lowercase().next().unwrap_or(character)
    }
}

fn equal(left: &str, right: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        left == right
    } else {
        left.to_lowercase() == right.to_lowercase()
    }
}

#[derive(Clone, Debug)]
enum Postfix {
    Atom(Matcher),
    Start,
    End,
    Concat,
    Alternate,
    Star,
    Plus,
    Question,
}

#[derive(Clone, Debug)]
enum State {
    Consume(Matcher, usize),
    Split(usize, usize),
    Start(usize),
    End(usize),
    Match,
}

#[derive(Clone, Debug)]
pub(crate) struct RegexProgram {
    states: Vec<State>,
    start: usize,
}

#[derive(Clone, Debug)]
struct Fragment {
    start: usize,
    outs: Vec<(usize, bool)>,
}

pub(crate) fn compile(pattern: &str, work: &mut WorkMeter) -> Result<RegexProgram, SearchError> {
    work.charge(pattern.len())?;
    let postfix = postfix(pattern, work)?;
    let mut states = Vec::new();
    let mut stack: Vec<Fragment> = Vec::new();
    for token in postfix {
        work.charge(1)?;
        match token {
            Postfix::Atom(matcher) => {
                let index = states.len();
                states.push(State::Consume(matcher, UNSET));
                stack.push(Fragment {
                    start: index,
                    outs: vec![(index, false)],
                });
            }
            Postfix::Start | Postfix::End => {
                let index = states.len();
                states.push(if matches!(token, Postfix::Start) {
                    State::Start(UNSET)
                } else {
                    State::End(UNSET)
                });
                stack.push(Fragment {
                    start: index,
                    outs: vec![(index, false)],
                });
            }
            Postfix::Concat => {
                let right = stack.pop().ok_or(SearchError::InvalidPattern)?;
                let left = stack.pop().ok_or(SearchError::InvalidPattern)?;
                patch(&mut states, &left.outs, right.start)?;
                stack.push(Fragment {
                    start: left.start,
                    outs: right.outs,
                });
            }
            Postfix::Alternate => {
                let right = stack.pop().ok_or(SearchError::InvalidPattern)?;
                let left = stack.pop().ok_or(SearchError::InvalidPattern)?;
                let index = states.len();
                states.push(State::Split(left.start, right.start));
                let mut outs = left.outs;
                outs.extend(right.outs);
                stack.push(Fragment { start: index, outs });
            }
            Postfix::Star | Postfix::Plus | Postfix::Question => {
                let fragment = stack.pop().ok_or(SearchError::InvalidPattern)?;
                let index = states.len();
                states.push(State::Split(fragment.start, UNSET));
                let result = match token {
                    Postfix::Star => {
                        patch(&mut states, &fragment.outs, index)?;
                        Fragment {
                            start: index,
                            outs: vec![(index, true)],
                        }
                    }
                    Postfix::Plus => {
                        patch(&mut states, &fragment.outs, index)?;
                        Fragment {
                            start: fragment.start,
                            outs: vec![(index, true)],
                        }
                    }
                    Postfix::Question => {
                        let mut outs = fragment.outs;
                        outs.push((index, true));
                        Fragment { start: index, outs }
                    }
                    _ => unreachable!(),
                };
                stack.push(result);
            }
        }
    }
    if stack.len() != 1 {
        return Err(SearchError::InvalidPattern);
    }
    let fragment = stack.pop().unwrap();
    let matched = states.len();
    states.push(State::Match);
    patch(&mut states, &fragment.outs, matched)?;
    Ok(RegexProgram {
        states,
        start: fragment.start,
    })
}

fn patch(states: &mut [State], outs: &[(usize, bool)], target: usize) -> Result<(), SearchError> {
    for &(index, second) in outs {
        match (&mut states[index], second) {
            (State::Consume(_, next) | State::Start(next) | State::End(next), false) => {
                *next = target
            }
            (State::Split(_, next), true) => *next = target,
            _ => return Err(SearchError::InvalidPattern),
        }
    }
    Ok(())
}

fn postfix(pattern: &str, work: &mut WorkMeter) -> Result<Vec<Postfix>, SearchError> {
    let mut output = Vec::new();
    let mut groups: Vec<(usize, usize)> = Vec::new();
    let mut alternatives = 0usize;
    let mut atoms = 0usize;
    let mut offset = 0usize;
    while offset < pattern.len() {
        let cluster = crate::grapheme_indices(&pattern[offset..])
            .next()
            .map(|(_, cluster)| cluster)
            .ok_or(SearchError::InvalidPattern)?;
        offset = offset.saturating_add(cluster.len());
        work.charge(1)?;
        match cluster {
            "(" => {
                emit_concat(&mut output, &mut atoms);
                groups.push((alternatives, atoms));
                alternatives = 0;
                atoms = 0;
            }
            ")" => {
                if atoms == 0 {
                    return Err(SearchError::InvalidPattern);
                }
                emit_concat(&mut output, &mut atoms);
                emit_alternates(&mut output, &mut alternatives);
                let (parent_alternatives, parent_atoms) =
                    groups.pop().ok_or(SearchError::InvalidPattern)?;
                alternatives = parent_alternatives;
                atoms = parent_atoms.saturating_add(1);
            }
            "|" => {
                if atoms == 0 {
                    return Err(SearchError::InvalidPattern);
                }
                emit_concat(&mut output, &mut atoms);
                alternatives = alternatives.saturating_add(1);
                atoms = 0;
            }
            "*" | "+" | "?" => {
                if atoms == 0 {
                    return Err(SearchError::InvalidPattern);
                }
                output.push(match cluster {
                    "*" => Postfix::Star,
                    "+" => Postfix::Plus,
                    _ => Postfix::Question,
                });
            }
            "^" | "$" | "." => push_atom(
                &mut output,
                &mut atoms,
                match cluster {
                    "^" => Postfix::Start,
                    "$" => Postfix::End,
                    _ => Postfix::Atom(Matcher::Any),
                },
            ),
            "[" => push_atom(
                &mut output,
                &mut atoms,
                Postfix::Atom(parse_class_at(pattern, &mut offset)?),
            ),
            "\\" => {
                let escaped = crate::grapheme_indices(&pattern[offset..])
                    .next()
                    .map(|(_, cluster)| cluster)
                    .ok_or(SearchError::InvalidPattern)?;
                offset = offset.saturating_add(escaped.len());
                push_atom(
                    &mut output,
                    &mut atoms,
                    Postfix::Atom(Matcher::Literal(escaped.to_owned())),
                );
            }
            literal => push_atom(
                &mut output,
                &mut atoms,
                Postfix::Atom(Matcher::Literal(literal.to_owned())),
            ),
        }
    }
    if !groups.is_empty() || atoms == 0 {
        return Err(SearchError::InvalidPattern);
    }
    emit_concat(&mut output, &mut atoms);
    emit_alternates(&mut output, &mut alternatives);
    Ok(output)
}

fn parse_class_at(pattern: &str, offset: &mut usize) -> Result<Matcher, SearchError> {
    let remaining = &pattern[*offset..];
    let mut escaped = false;
    let mut closing = None;
    for (index, character) in remaining.char_indices() {
        if escaped {
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == ']' {
            closing = Some(index);
            break;
        }
    }
    let closing = closing.ok_or(SearchError::InvalidPattern)?;
    let body = &remaining[..closing];
    *offset = offset.saturating_add(closing + 1);
    let mut chars = body.chars().peekable();
    parse_class(&mut chars)
}

fn push_atom(output: &mut Vec<Postfix>, atoms: &mut usize, atom: Postfix) {
    if *atoms > 1 {
        output.push(Postfix::Concat);
        *atoms -= 1;
    }
    output.push(atom);
    *atoms = atoms.saturating_add(1);
}

fn emit_concat(output: &mut Vec<Postfix>, atoms: &mut usize) {
    while *atoms > 1 {
        output.push(Postfix::Concat);
        *atoms -= 1;
    }
}

fn emit_alternates(output: &mut Vec<Postfix>, alternatives: &mut usize) {
    while *alternatives > 0 {
        output.push(Postfix::Alternate);
        *alternatives -= 1;
    }
}

fn parse_class(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> Result<Matcher, SearchError> {
    let negated = chars.next_if_eq(&'^').is_some();
    let mut tokens = Vec::new();
    while let Some(character) = chars.next() {
        if character == '\\' {
            tokens.push((chars.next().ok_or(SearchError::InvalidPattern)?, true));
        } else {
            tokens.push((character, false));
        }
    }
    let mut ranges = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        let start = tokens[index].0;
        if tokens.get(index + 1) == Some(&('-', false)) && index + 2 < tokens.len() {
            let end = tokens[index + 2].0;
            if start > end {
                return Err(SearchError::InvalidPattern);
            }
            ranges.push((start, end));
            index += 3;
        } else {
            ranges.push((start, start));
            index += 1;
        }
    }
    if ranges.is_empty() {
        Err(SearchError::InvalidPattern)
    } else {
        Ok(Matcher::Class { negated, ranges })
    }
}

impl RegexProgram {
    pub(crate) fn match_at(
        &self,
        units: &[String],
        start: usize,
        case_sensitive: bool,
        work: &mut WorkMeter,
    ) -> Result<Option<usize>, SearchError> {
        let mut current = Vec::new();
        self.add_state(&mut current, self.start, start, units, work)?;
        let mut matched = current
            .iter()
            .any(|&state| matches!(self.states[state], State::Match))
            .then_some(start);
        for (offset, unit) in units[start..].iter().enumerate() {
            let mut next = Vec::new();
            for &state in &current {
                work.charge(1)?;
                if let State::Consume(matcher, target) = &self.states[state]
                    && matcher.accepts(unit, case_sensitive)
                {
                    self.add_state(&mut next, *target, start + offset + 1, units, work)?;
                }
            }
            current = next;
            if current
                .iter()
                .any(|&state| matches!(self.states[state], State::Match))
            {
                matched = Some(start + offset + 1);
            }
            if current.is_empty() {
                break;
            }
        }
        Ok(matched)
    }

    fn add_state(
        &self,
        output: &mut Vec<usize>,
        state: usize,
        position: usize,
        units: &[String],
        work: &mut WorkMeter,
    ) -> Result<(), SearchError> {
        let mut pending = vec![state];
        let mut seen = vec![false; self.states.len()];
        while let Some(index) = pending.pop() {
            work.charge(1)?;
            if index == UNSET || seen[index] {
                continue;
            }
            seen[index] = true;
            match self.states[index] {
                State::Split(left, right) => {
                    pending.push(right);
                    pending.push(left);
                }
                State::Start(next)
                    if position == 0
                        || units
                            .get(position.wrapping_sub(1))
                            .is_some_and(|unit| unit == "\n") =>
                {
                    pending.push(next)
                }
                State::End(next)
                    if position == units.len()
                        || units.get(position).is_some_and(|unit| unit == "\n") =>
                {
                    pending.push(next)
                }
                State::Start(_) | State::End(_) => {}
                _ => output.push(index),
            }
        }
        Ok(())
    }
}
