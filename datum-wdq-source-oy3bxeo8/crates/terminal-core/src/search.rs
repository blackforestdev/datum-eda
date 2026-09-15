use crate::grid::GridRow;
use crate::{CellContent, LimitError, LimitKind, LogicalPoint, ScreenBuffer, TerminalCore};
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchCase {
    Sensitive,
    Insensitive,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SearchQuery {
    Literal { text: String, case: SearchCase },
    Regex { pattern: String, case: SearchCase },
}

impl SearchQuery {
    pub fn literal(text: impl Into<String>, case: SearchCase) -> Self {
        Self::Literal {
            text: text.into(),
            case,
        }
    }

    pub fn regex(pattern: impl Into<String>, case: SearchCase) -> Self {
        Self::Regex {
            pattern: pattern.into(),
            case,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SearchCursor {
    pub after: Option<LogicalPoint>,
    pub direction: SearchDirection,
}

impl SearchCursor {
    pub const fn forward(after: Option<LogicalPoint>) -> Self {
        Self {
            after,
            direction: SearchDirection::Forward,
        }
    }
    pub const fn backward(after: Option<LogicalPoint>) -> Self {
        Self {
            after,
            direction: SearchDirection::Backward,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SearchMatch {
    start: LogicalPoint,
    end: LogicalPoint,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchMatchState {
    Active,
    Trimmed,
    Unknown,
}

impl SearchMatch {
    pub const fn new(start: LogicalPoint, end: LogicalPoint) -> Self {
        Self { start, end }
    }

    pub const fn start(self) -> LogicalPoint {
        self.start
    }
    pub const fn end(self) -> LogicalPoint {
        self.end
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SearchResult {
    matched: Option<SearchMatch>,
    work: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchBatch {
    matches: Vec<SearchMatch>,
    work: usize,
}

impl SearchBatch {
    pub fn matches(&self) -> &[SearchMatch] {
        &self.matches
    }

    pub const fn work(&self) -> usize {
        self.work
    }
}

impl SearchResult {
    pub const fn matched(self) -> Option<SearchMatch> {
        self.matched
    }
    pub const fn work(self) -> usize {
        self.work
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SearchError {
    EmptyPattern,
    InvalidPattern,
    TrimmedCursor,
    UnknownCursor,
    Limit(LimitError),
}

impl fmt::Display for SearchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPattern => formatter.write_str("terminal search pattern is empty"),
            Self::InvalidPattern => formatter.write_str("terminal regex pattern is invalid"),
            Self::TrimmedCursor => formatter.write_str("terminal search cursor was trimmed"),
            Self::UnknownCursor => formatter.write_str("terminal search cursor is unknown"),
            Self::Limit(error) => error.fmt(formatter),
        }
    }
}

impl Error for SearchError {}

pub(crate) struct WorkMeter {
    used: usize,
    maximum: usize,
}

impl WorkMeter {
    fn new(maximum: usize) -> Self {
        Self { used: 0, maximum }
    }
    pub(crate) fn charge(&mut self, amount: usize) -> Result<(), SearchError> {
        let requested = self.used.checked_add(amount).ok_or(SearchError::Limit(
            LimitError::ArithmeticOverflow {
                kind: LimitKind::SearchWork,
            },
        ))?;
        if requested > self.maximum {
            return Err(SearchError::Limit(LimitError::Exceeded {
                kind: LimitKind::SearchWork,
                requested,
                maximum: self.maximum,
            }));
        }
        self.used = requested;
        Ok(())
    }
}

#[derive(Clone)]
struct Unit {
    text: String,
    point: Option<LogicalPoint>,
}

impl TerminalCore {
    pub fn search(
        &self,
        query: &SearchQuery,
        cursor: SearchCursor,
    ) -> Result<SearchResult, SearchError> {
        let mut work = WorkMeter::new(self.limits.search_work.get());
        let units = document_units(self, &mut work)?;
        let origin = cursor_index(self, &units, cursor)?;
        let found = match query {
            SearchQuery::Literal { text, case } => {
                search_literal(&units, text, *case, origin, cursor.direction, &mut work)?
            }
            SearchQuery::Regex { pattern, case } => {
                search_regex(&units, pattern, *case, origin, cursor.direction, &mut work)?
            }
        };
        Ok(SearchResult {
            matched: found,
            work: work.used,
        })
    }

    /// Return every match in document order under one shared search-work
    /// budget. This is the bounded authority used by terminal all-match
    /// highlighting; callers must not emulate it with an unbounded sequence of
    /// independent searches.
    pub fn search_all(&self, query: &SearchQuery) -> Result<SearchBatch, SearchError> {
        let mut work = WorkMeter::new(self.limits.search_work.get());
        let units = document_units(self, &mut work)?;
        let matches = match query {
            SearchQuery::Literal { text, case } => {
                search_all_literal(&units, text, *case, &mut work)?
            }
            SearchQuery::Regex { pattern, case } => {
                search_all_regex(&units, pattern, *case, &mut work)?
            }
        };
        Ok(SearchBatch {
            matches,
            work: work.used,
        })
    }

    pub fn search_match_state(&self, matched: SearchMatch) -> SearchMatchState {
        let start = self.state.resolve_logical_point(matched.start);
        let end = self.state.resolve_logical_point(matched.end);
        if matches!(start, crate::AnchorResolution::Trimmed)
            || matches!(end, crate::AnchorResolution::Trimmed)
        {
            SearchMatchState::Trimmed
        } else if matches!(start, crate::AnchorResolution::Unknown)
            || matches!(end, crate::AnchorResolution::Unknown)
        {
            SearchMatchState::Unknown
        } else {
            SearchMatchState::Active
        }
    }
}

fn document_units(core: &TerminalCore, work: &mut WorkMeter) -> Result<Vec<Unit>, SearchError> {
    let mut rows: Vec<&GridRow> = Vec::new();
    if core.state.active_buffer == ScreenBuffer::Primary {
        rows.extend(core.state.history.rows());
    }
    rows.extend(&core.state.active_grid().rows);
    let last = rows
        .iter()
        .rposition(|row| {
            row.cells
                .iter()
                .any(|cell| !matches!(cell.content, CellContent::Empty))
        })
        .map_or(0, |index| index + 1);
    let mut units = Vec::new();
    for (row_index, row) in rows[..last].iter().enumerate() {
        let mut cluster = row.cluster_start;
        for cell in &row.cells {
            work.charge(1)?;
            match &cell.content {
                CellContent::Cluster(value) => {
                    units.push(Unit {
                        text: value.text().to_owned(),
                        point: Some(LogicalPoint {
                            line: row.logical_line,
                            cluster,
                        }),
                    });
                    cluster = cluster.saturating_add(1);
                }
                CellContent::Empty => cluster = cluster.saturating_add(1),
                CellContent::Continuation { .. } => {}
            }
        }
        if !row.soft_wrapped && row_index + 1 < last {
            units.push(Unit {
                text: "\n".into(),
                point: None,
            });
        }
    }
    Ok(units)
}

fn cursor_index(
    core: &TerminalCore,
    units: &[Unit],
    cursor: SearchCursor,
) -> Result<usize, SearchError> {
    let Some(point) = cursor.after else {
        return Ok(if cursor.direction == SearchDirection::Forward {
            0
        } else {
            units.len()
        });
    };
    match core.state.resolve_logical_point(point) {
        crate::AnchorResolution::Trimmed => return Err(SearchError::TrimmedCursor),
        crate::AnchorResolution::Unknown => return Err(SearchError::UnknownCursor),
        _ => {}
    }
    let index = units
        .iter()
        .position(|unit| unit.point == Some(point))
        .ok_or(SearchError::UnknownCursor)?;
    Ok(match cursor.direction {
        SearchDirection::Forward => index.saturating_add(1),
        SearchDirection::Backward => index,
    })
}

fn pattern_units(text: &str) -> Result<Vec<String>, SearchError> {
    if text.is_empty() {
        return Err(SearchError::EmptyPattern);
    }
    Ok(crate::grapheme_indices(text)
        .map(|(_, cluster)| cluster.to_owned())
        .collect())
}

fn search_literal(
    units: &[Unit],
    text: &str,
    case: SearchCase,
    origin: usize,
    direction: SearchDirection,
    work: &mut WorkMeter,
) -> Result<Option<SearchMatch>, SearchError> {
    let pattern = pattern_units(text)?;
    work.charge(pattern.len())?;
    let starts: Box<dyn Iterator<Item = usize>> = match direction {
        SearchDirection::Forward => Box::new(origin..units.len()),
        SearchDirection::Backward => Box::new((0..origin.min(units.len())).rev()),
    };
    for start in starts {
        if start + pattern.len() > units.len() {
            continue;
        }
        let mut equal = true;
        for (unit, expected) in units[start..start + pattern.len()].iter().zip(&pattern) {
            work.charge(1)?;
            equal &= compare(&unit.text, expected, case);
            if !equal {
                break;
            }
        }
        if equal && let Some(found) = make_match(units, start, start + pattern.len()) {
            return Ok(Some(found));
        }
    }
    Ok(None)
}

fn search_all_literal(
    units: &[Unit],
    text: &str,
    case: SearchCase,
    work: &mut WorkMeter,
) -> Result<Vec<SearchMatch>, SearchError> {
    let pattern = pattern_units(text)?;
    work.charge(pattern.len())?;
    let mut matches = Vec::new();
    for start in 0..units.len() {
        if start + pattern.len() > units.len() {
            break;
        }
        let mut equal = true;
        for (unit, expected) in units[start..start + pattern.len()].iter().zip(&pattern) {
            work.charge(1)?;
            if !compare(&unit.text, expected, case) {
                equal = false;
                break;
            }
        }
        if equal && let Some(found) = make_match(units, start, start + pattern.len()) {
            work.charge(1)?;
            matches.push(found);
        }
    }
    Ok(matches)
}

fn search_regex(
    units: &[Unit],
    pattern: &str,
    case: SearchCase,
    origin: usize,
    direction: SearchDirection,
    work: &mut WorkMeter,
) -> Result<Option<SearchMatch>, SearchError> {
    if pattern.is_empty() {
        return Err(SearchError::EmptyPattern);
    }
    let program = crate::search_regex::compile(pattern, work)?;
    let texts = units
        .iter()
        .map(|unit| unit.text.clone())
        .collect::<Vec<_>>();
    let starts: Box<dyn Iterator<Item = usize>> = match direction {
        SearchDirection::Forward => Box::new(origin..=units.len()),
        SearchDirection::Backward => Box::new((0..origin.min(units.len()).saturating_add(1)).rev()),
    };
    for start in starts {
        if let Some(end) = program.match_at(&texts, start, case == SearchCase::Sensitive, work)?
            && let Some(found) = make_match(units, start, end)
        {
            return Ok(Some(found));
        }
    }
    Ok(None)
}

fn search_all_regex(
    units: &[Unit],
    pattern: &str,
    case: SearchCase,
    work: &mut WorkMeter,
) -> Result<Vec<SearchMatch>, SearchError> {
    if pattern.is_empty() {
        return Err(SearchError::EmptyPattern);
    }
    let program = crate::search_regex::compile(pattern, work)?;
    let texts = units
        .iter()
        .map(|unit| unit.text.clone())
        .collect::<Vec<_>>();
    let mut matches = Vec::new();
    for start in 0..=units.len() {
        if let Some(end) = program.match_at(&texts, start, case == SearchCase::Sensitive, work)?
            && let Some(found) = make_match(units, start, end)
            && matches.last() != Some(&found)
        {
            work.charge(1)?;
            matches.push(found);
        }
    }
    Ok(matches)
}

fn make_match(units: &[Unit], start: usize, end: usize) -> Option<SearchMatch> {
    if start == end {
        let point = units
            .get(start)
            .and_then(|unit| unit.point)
            .or_else(|| units.get(start.wrapping_sub(1)).and_then(|unit| unit.point))?;
        return Some(SearchMatch {
            start: point,
            end: point,
        });
    }
    let first = units.get(start..end)?.iter().find_map(|unit| unit.point)?;
    let last = units
        .get(start..end)?
        .iter()
        .rev()
        .find_map(|unit| unit.point)?;
    Some(SearchMatch {
        start: first,
        end: last,
    })
}

fn compare(left: &str, right: &str, case: SearchCase) -> bool {
    match case {
        SearchCase::Sensitive => left == right,
        SearchCase::Insensitive => left.to_lowercase() == right.to_lowercase(),
    }
}
