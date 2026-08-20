use crate::{
    CellWidth, Cluster, unicode_grapheme_tables as grapheme_tables,
    unicode_width_tables as width_tables,
};

pub const UNICODE_VERSION: &str = "17.0.0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BidirectionalTextPolicy {
    LogicalOrder,
}

pub const BIDIRECTIONAL_TEXT_POLICY: BidirectionalTextPolicy =
    BidirectionalTextPolicy::LogicalOrder;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShapingCluster<'a> {
    text: &'a str,
    cell_width: CellWidth,
}

impl<'a> ShapingCluster<'a> {
    pub fn from_cluster(cluster: &'a Cluster) -> Self {
        Self {
            text: cluster.text(),
            cell_width: cluster.width(),
        }
    }

    pub const fn text(self) -> &'a str {
        self.text
    }

    pub const fn cell_width(self) -> CellWidth {
        self.cell_width
    }
}

pub(crate) type Range<T> = (u32, u32, T);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GraphemeBreak {
    Other,
    CR,
    LF,
    Control,
    Extend,
    Zwj,
    RegionalIndicator,
    Prepend,
    SpacingMark,
    L,
    V,
    T,
    LV,
    Lvt,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IndicConjunctBreak {
    None,
    Consonant,
    Extend,
    Linker,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EastAsianWidth {
    Neutral,
    A,
    W,
    F,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EmojiProperty {
    ExtendedPictographic,
    EmojiPresentation,
}

pub fn grapheme_break_before(cluster: &str, next: char) -> bool {
    let Some(previous) = cluster.chars().next_back() else {
        return true;
    };
    let prior = grapheme_property(previous);
    let following = grapheme_property(next);

    if prior == GraphemeBreak::CR && following == GraphemeBreak::LF {
        return false;
    }
    if matches!(
        prior,
        GraphemeBreak::CR | GraphemeBreak::LF | GraphemeBreak::Control
    ) || matches!(
        following,
        GraphemeBreak::CR | GraphemeBreak::LF | GraphemeBreak::Control
    ) {
        return true;
    }
    if prior == GraphemeBreak::L
        && matches!(
            following,
            GraphemeBreak::L | GraphemeBreak::V | GraphemeBreak::LV | GraphemeBreak::Lvt
        )
    {
        return false;
    }
    if matches!(prior, GraphemeBreak::LV | GraphemeBreak::V)
        && matches!(following, GraphemeBreak::V | GraphemeBreak::T)
    {
        return false;
    }
    if matches!(prior, GraphemeBreak::Lvt | GraphemeBreak::T) && following == GraphemeBreak::T {
        return false;
    }
    if matches!(
        following,
        GraphemeBreak::Extend | GraphemeBreak::Zwj | GraphemeBreak::SpacingMark
    ) || prior == GraphemeBreak::Prepend
    {
        return false;
    }
    if indic_conjunct_joins(cluster, next) || extended_pictographic_joins(cluster, next) {
        return false;
    }
    if prior == GraphemeBreak::RegionalIndicator
        && following == GraphemeBreak::RegionalIndicator
        && trailing_regional_indicators(cluster) % 2 == 1
    {
        return false;
    }
    true
}

pub fn grapheme_indices(text: &str) -> GraphemeIndices<'_> {
    GraphemeIndices {
        text,
        start: 0,
        scan: 0,
    }
}

pub struct GraphemeIndices<'a> {
    text: &'a str,
    start: usize,
    scan: usize,
}

impl<'a> Iterator for GraphemeIndices<'a> {
    type Item = (usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.text.len() {
            return None;
        }
        if self.scan == self.start {
            self.scan += self.text[self.scan..].chars().next()?.len_utf8();
        }
        while self.scan < self.text.len() {
            let next = self.text[self.scan..].chars().next()?;
            if grapheme_break_before(&self.text[self.start..self.scan], next) {
                let result = (self.start, &self.text[self.start..self.scan]);
                self.start = self.scan;
                return Some(result);
            }
            self.scan += next.len_utf8();
        }
        let result = (self.start, &self.text[self.start..]);
        self.start = self.text.len();
        Some(result)
    }
}

pub fn terminal_cluster_width(cluster: &str) -> CellWidth {
    let characters = cluster.chars().collect::<Vec<_>>();
    let emoji_text_override = characters.last() == Some(&'\u{fe0e}');
    let emoji_presentation = !emoji_text_override
        && (characters.contains(&'\u{fe0f}')
            || characters.contains(&'\u{20e3}')
            || characters
                .iter()
                .any(|character| is_emoji_presentation(*character))
            || is_regional_indicator_pair(&characters)
            || is_extended_pictographic_zwj_sequence(&characters));
    if emoji_presentation
        || characters.iter().any(|character| {
            matches!(
                east_asian_width(*character),
                EastAsianWidth::W | EastAsianWidth::F
            )
        })
    {
        CellWidth::Two
    } else {
        CellWidth::One
    }
}

fn indic_conjunct_joins(cluster: &str, next: char) -> bool {
    if indic_conjunct_property(next) != IndicConjunctBreak::Consonant {
        return false;
    }
    let mut saw_linker = false;
    for character in cluster.chars().rev() {
        match indic_conjunct_property(character) {
            IndicConjunctBreak::Extend => {}
            IndicConjunctBreak::Linker => saw_linker = true,
            IndicConjunctBreak::Consonant => return saw_linker,
            IndicConjunctBreak::None => return false,
        }
    }
    false
}

fn extended_pictographic_joins(cluster: &str, next: char) -> bool {
    if !is_extended_pictographic(next)
        || grapheme_property(cluster.chars().next_back().unwrap()) != GraphemeBreak::Zwj
    {
        return false;
    }
    let mut characters = cluster.chars().rev();
    characters.next();
    for character in characters {
        if grapheme_property(character) == GraphemeBreak::Extend {
            continue;
        }
        return is_extended_pictographic(character);
    }
    false
}

fn trailing_regional_indicators(cluster: &str) -> usize {
    cluster
        .chars()
        .rev()
        .take_while(|character| grapheme_property(*character) == GraphemeBreak::RegionalIndicator)
        .count()
}

fn is_regional_indicator_pair(characters: &[char]) -> bool {
    characters.len() == 2
        && characters
            .iter()
            .all(|character| grapheme_property(*character) == GraphemeBreak::RegionalIndicator)
}

fn is_extended_pictographic_zwj_sequence(characters: &[char]) -> bool {
    characters.contains(&'\u{200d}')
        && characters
            .iter()
            .filter(|character| is_extended_pictographic(**character))
            .count()
            >= 2
}

fn grapheme_property(character: char) -> GraphemeBreak {
    lookup(character, grapheme_tables::GRAPHEME_BREAK_RANGES).unwrap_or(GraphemeBreak::Other)
}

fn indic_conjunct_property(character: char) -> IndicConjunctBreak {
    lookup(character, grapheme_tables::INCB_RANGES).unwrap_or(IndicConjunctBreak::None)
}

fn east_asian_width(character: char) -> EastAsianWidth {
    lookup(character, width_tables::EAST_ASIAN_WIDTH_RANGES).unwrap_or(EastAsianWidth::Neutral)
}

fn is_extended_pictographic(character: char) -> bool {
    lookup(character, width_tables::EXTENDED_PICTOGRAPHIC_RANGES).is_some()
}

fn is_emoji_presentation(character: char) -> bool {
    lookup(character, width_tables::EMOJI_PRESENTATION_RANGES).is_some()
}

fn lookup<T: Copy>(character: char, ranges: &[Range<T>]) -> Option<T> {
    let codepoint = u32::from(character);
    let index = ranges.partition_point(|range| range.1 < codepoint);
    let range = ranges.get(index)?;
    (range.0 <= codepoint).then_some(range.2)
}
