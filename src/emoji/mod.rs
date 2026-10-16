//! Emoji data, search, and picker for the built-in emoji surface.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Emoji {
    pub emoji: &'static str,
    pub name: &'static str,
    pub keywords: &'static [&'static str],
    pub category: EmojiCategory,
    /// Light-to-dark skin-tone variants, when the emoji takes a skin tone.
    pub tones: Option<[&'static str; 5]>,
    /// Emoji version that introduced it, in tenths (`15.1` -> 151).
    pub version: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter)]
pub enum EmojiCategory {
    SmileysEmotion,
    PeopleBody,
    AnimalsNature,
    FoodDrink,
    TravelPlaces,
    Activities,
    Objects,
    Symbols,
    Flags,
}

impl EmojiCategory {
    pub const fn display_name(self) -> &'static str {
        match self {
            EmojiCategory::SmileysEmotion => "Smileys & Emotion",
            EmojiCategory::PeopleBody => "People & Body",
            EmojiCategory::AnimalsNature => "Animals & Nature",
            EmojiCategory::FoodDrink => "Food & Drink",
            EmojiCategory::TravelPlaces => "Travel & Places",
            EmojiCategory::Activities => "Activities",
            EmojiCategory::Objects => "Objects",
            EmojiCategory::Symbols => "Symbols",
            EmojiCategory::Flags => "Flags",
        }
    }
}

use strum::IntoEnumIterator;

/// Returns an iterator over all emoji categories in declaration order.
pub fn all_categories() -> impl Iterator<Item = EmojiCategory> {
    EmojiCategory::iter()
}

/// Skin tone applied to emoji that support one (hands, people, gestures).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    serde::Serialize,
    serde::Deserialize,
    strum::EnumIter,
)]
#[serde(rename_all = "camelCase")]
pub enum SkinTone {
    #[default]
    None,
    Light,
    MediumLight,
    Medium,
    MediumDark,
    Dark,
}

impl SkinTone {
    pub const fn title(self) -> &'static str {
        match self {
            Self::None => "No Skin Tone",
            Self::Light => "Light",
            Self::MediumLight => "Medium-Light",
            Self::Medium => "Medium",
            Self::MediumDark => "Medium-Dark",
            Self::Dark => "Dark",
        }
    }

    pub const fn id(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Light => "light",
            Self::MediumLight => "medium_light",
            Self::Medium => "medium",
            Self::MediumDark => "medium_dark",
            Self::Dark => "dark",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::iter().find(|tone| tone.id() == id)
    }

    /// Index into an emoji's light-to-dark variants, or `None` for no tone.
    const fn variant_index(self) -> Option<usize> {
        match self {
            Self::None => None,
            Self::Light => Some(0),
            Self::MediumLight => Some(1),
            Self::Medium => Some(2),
            Self::MediumDark => Some(3),
            Self::Dark => Some(4),
        }
    }
}

/// The full Unicode emoji set, generated from Unicode's `emoji-test.txt` and
/// CLDR keywords by `scripts/emoji/generate_emoji_data.py`.
const EMOJI_DATA: &str = include_str!("emoji_data.tsv");

/// Every emoji this Mac can draw, in Unicode order. Emoji newer than the
/// running macOS supports are left out so the grid never shows empty boxes.
pub static EMOJIS: std::sync::LazyLock<Vec<Emoji>> = std::sync::LazyLock::new(|| {
    let supported = supported_emoji_version();
    parse_emoji_data(EMOJI_DATA)
        .into_iter()
        .filter(|emoji| emoji.version <= supported)
        .collect()
});

fn parse_category(group: &str) -> Option<EmojiCategory> {
    Some(match group {
        "SmileysEmotion" => EmojiCategory::SmileysEmotion,
        "PeopleBody" => EmojiCategory::PeopleBody,
        "AnimalsNature" => EmojiCategory::AnimalsNature,
        "FoodDrink" => EmojiCategory::FoodDrink,
        "TravelPlaces" => EmojiCategory::TravelPlaces,
        "Activities" => EmojiCategory::Activities,
        "Objects" => EmojiCategory::Objects,
        "Symbols" => EmojiCategory::Symbols,
        "Flags" => EmojiCategory::Flags,
        _ => return None,
    })
}

/// Emoji version as tenths (`15.1` -> 151) so it compares exactly.
fn parse_version(version: &str) -> Option<u16> {
    let (major, minor) = version.split_once('.')?;
    Some(major.parse::<u16>().ok()? * 10 + minor.parse::<u16>().ok()?)
}

fn parse_emoji_data(data: &'static str) -> Vec<Emoji> {
    data.lines()
        .filter_map(|line| {
            let mut columns = line.split('\t');
            let emoji = columns.next()?;
            let name = columns.next()?;
            let category = parse_category(columns.next()?)?;
            let version = parse_version(columns.next()?)?;
            let tones = columns.next()?;
            let keywords = columns.next().unwrap_or_default();
            let tones = {
                let variants: Vec<&'static str> =
                    tones.split(' ').filter(|v| !v.is_empty()).collect();
                <[&'static str; 5]>::try_from(variants).ok()
            };
            let keywords: Vec<&'static str> = keywords
                .split('|')
                .filter(|word| !word.is_empty())
                .collect();
            Some(Emoji {
                emoji,
                name,
                keywords: Box::leak(keywords.into_boxed_slice()),
                category,
                tones,
                version,
            })
        })
        .collect()
}

/// Newest emoji version the running macOS can draw, as tenths.
///
/// Apple ships new emoji in point releases: 14.0 in macOS 12.3, 15.0 in 13.3,
/// 15.1 in 14.4, 16.0 in 15.4. Later releases are assumed to keep pace.
fn supported_emoji_version() -> u16 {
    let (major, minor) = macos_version().unwrap_or((26, 0));
    emoji_version_for_macos(major, minor)
}

pub(crate) fn emoji_version_for_macos(major: u32, minor: u32) -> u16 {
    match (major, minor) {
        (0..=11, _) => 131,
        (12, 0..=2) => 131,
        (12, _) => 140,
        (13, 0..=2) => 140,
        (13, _) => 150,
        (14, 0..=3) => 150,
        (14, _) => 151,
        (15, 0..=3) => 151,
        (15, _) => 160,
        (16..=26, 0..=3) => 160,
        (16..=26, _) => 170,
        _ => 180,
    }
}

fn macos_version() -> Option<(u32, u32)> {
    let text = std::process::Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()?
        .stdout;
    let text = String::from_utf8(text).ok()?;
    let mut parts = text.trim().split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().and_then(|part| part.parse().ok()).unwrap_or(0);
    Some((major, minor))
}

impl Emoji {
    /// The emoji with `tone` applied, or the plain emoji when it has no tones.
    pub fn with_tone(&self, tone: SkinTone) -> &'static str {
        match (tone.variant_index(), self.tones) {
            (Some(index), Some(variants)) => variants[index],
            _ => self.emoji,
        }
    }

    pub fn supports_skin_tones(&self) -> bool {
        self.tones.is_some()
    }
}

/// The emoji record for `value`, which may be a skin-tone variant.
pub fn emoji_for_variant(value: &str) -> Option<Emoji> {
    EMOJIS
        .iter()
        .find(|emoji| {
            emoji.emoji == value || emoji.tones.is_some_and(|tones| tones.contains(&value))
        })
        .copied()
}

pub fn emojis_by_category(category: EmojiCategory) -> Vec<&'static Emoji> {
    EMOJIS
        .iter()
        .filter(|emoji| emoji.category == category)
        .collect()
}

pub fn grouped_emojis() -> Vec<(EmojiCategory, Vec<&'static Emoji>)> {
    all_categories()
        .map(|category| (category, emojis_by_category(category)))
        .collect()
}

/// Number of columns in the emoji picker grid.
/// Shared between the renderer and the arrow-key navigation interceptor.
pub const GRID_COLS: usize = 12;

/// Fixed square size for each rendered emoji tile.
pub const GRID_TILE_SIZE: f32 = 48.0;

/// Horizontal spacing between emoji tiles. Combined with the tile size, this
/// matches the row height so horizontal and vertical centers share one rhythm.
pub const GRID_TILE_GAP: f32 = 8.0;

/// Fixed height for every emoji picker row (headers and cell rows).
pub const GRID_ROW_HEIGHT: f32 = 56.0;

/// Glyph scale relative to the tile, keeping emoji sizing derived from the
/// shared tile size instead of introducing a renderer-local pixel value.
pub const GRID_GLYPH_SCALE: f32 = 0.75;

/// Label rendered as the section header above the Frequently Used head
/// block. Pulled into a constant so the renderer, audits, and tests all
/// agree on the exact text.
pub const FREQUENTLY_USED_LABEL: &str = "Frequently Used";

/// Build the filtered, category-ordered emoji list used by both the renderer
/// and the arrow-key navigation handler. This ensures selection indices stay
/// in sync between rendering and navigation.
pub fn filtered_ordered_emojis(
    filter: &str,
    selected_category: Option<EmojiCategory>,
) -> Vec<Emoji> {
    let mut filtered: Vec<Emoji> = search_emojis(filter).into_iter().copied().collect();
    if let Some(cat) = selected_category {
        filtered.retain(|e| e.category == cat);
    }
    let mut ordered: Vec<Emoji> = Vec::with_capacity(filtered.len());
    for cat in all_categories() {
        ordered.extend(filtered.iter().copied().filter(|e| e.category == cat));
    }
    ordered
}

/// Return the canonical EMOJIS slice index for an emoji string, used as the
/// final stable tie-break when ranking frequent emojis. `None` for unknown
/// strings so callers can sort them last.
pub fn dataset_order_of(emoji: &str) -> Option<usize> {
    EMOJIS.iter().position(|e| e.emoji == emoji)
}

/// Look up the canonical Emoji record for a given string, or `None` if not in
/// the dataset (e.g. a usage entry left over from a prior dataset).
pub fn emoji_by_value(emoji: &str) -> Option<Emoji> {
    EMOJIS.iter().find(|e| e.emoji == emoji).copied()
}

/// Label for the pinned head section.
pub const PINNED_LABEL: &str = "Pinned";
/// Label for the ranked search results section.
pub const RESULTS_LABEL: &str = "Results";

/// What leads the picker before the categories: your pinned emoji, in the
/// order you arranged them, then the ones you use most. Frozen while the
/// picker is open so selection indices stay stable.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EmojiHead {
    pub pinned: Vec<String>,
    pub frequent: Vec<String>,
}

/// One titled run of emoji in the picker grid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmojiSection {
    pub title: String,
    /// Which head block this is, if any; categories and results are `None`.
    pub head: Option<EmojiHeadKind>,
    pub count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmojiHeadKind {
    Pinned,
    Frequent,
}

/// The emoji in display order and the sections they fall into. Render,
/// navigation, and actions all use this so the selected index means the
/// same emoji everywhere.
///
/// - Searching shows one ranked Results section.
/// - Browsing all emoji shows Pinned, then Frequently Used, then each
///   category, never repeating an emoji.
/// - Browsing one category shows just that category.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayOrder {
    pub emojis: Vec<Emoji>,
    pub sections: Vec<EmojiSection>,
}

impl DisplayOrder {
    /// Number of leading emojis in the Frequently Used section (0 if none).
    pub fn frequent_count(&self) -> usize {
        self.head_count(EmojiHeadKind::Frequent)
    }

    /// Number of leading emojis in the Pinned section (0 if none).
    pub fn pinned_count(&self) -> usize {
        self.head_count(EmojiHeadKind::Pinned)
    }

    fn head_count(&self, kind: EmojiHeadKind) -> usize {
        self.sections
            .iter()
            .find(|section| section.head == Some(kind))
            .map_or(0, |section| section.count)
    }

    /// Which section `index` falls in.
    pub fn section_of(&self, index: usize) -> Option<&EmojiSection> {
        let mut start = 0;
        for section in &self.sections {
            if index < start + section.count {
                return Some(section);
            }
            start += section.count;
        }
        None
    }

    /// Number of rendered rows: one header plus the cell rows per section.
    pub fn row_count(&self, cols: usize) -> usize {
        self.sections
            .iter()
            .map(|section| 1 + section.count.div_ceil(cols))
            .sum()
    }
}

pub fn display_ordered_emojis(
    filter: &str,
    selected_category: Option<EmojiCategory>,
    head: &EmojiHead,
) -> DisplayOrder {
    let mut emojis = Vec::new();
    let mut sections = Vec::new();
    let mut push =
        |title: &str, kind: Option<EmojiHeadKind>, items: Vec<Emoji>, emojis: &mut Vec<Emoji>| {
            if items.is_empty() {
                return;
            }
            sections.push(EmojiSection {
                title: title.to_string(),
                head: kind,
                count: items.len(),
            });
            emojis.extend(items);
        };

    if !filter.trim().is_empty() {
        let results = search_emojis(filter)
            .into_iter()
            .filter(|emoji| selected_category.is_none_or(|category| emoji.category == category))
            .copied()
            .collect();
        push(RESULTS_LABEL, None, results, &mut emojis);
        return DisplayOrder { emojis, sections };
    }

    if let Some(category) = selected_category {
        push(
            category.display_name(),
            None,
            emojis_by_category(category).into_iter().copied().collect(),
            &mut emojis,
        );
        return DisplayOrder { emojis, sections };
    }

    let mut seen = std::collections::HashSet::new();
    let mut resolve = |values: &[String]| -> Vec<Emoji> {
        values
            .iter()
            .filter_map(|value| emoji_for_variant(value))
            .filter(|emoji| seen.insert(emoji.emoji))
            .collect()
    };
    let pinned = resolve(&head.pinned);
    let frequent = resolve(&head.frequent);
    push(
        PINNED_LABEL,
        Some(EmojiHeadKind::Pinned),
        pinned,
        &mut emojis,
    );
    push(
        FREQUENTLY_USED_LABEL,
        Some(EmojiHeadKind::Frequent),
        frequent,
        &mut emojis,
    );
    for category in all_categories() {
        let items = emojis_by_category(category)
            .into_iter()
            .filter(|emoji| !seen.contains(emoji.emoji))
            .copied()
            .collect();
        push(category.display_name(), None, items, &mut emojis);
    }
    DisplayOrder { emojis, sections }
}

/// Compute the uniform-list row index that contains `selected_index`.
///
/// Row layout per category: 1 header row + ceil(count / GRID_COLS) cell rows.
/// Returns the row suitable for `scroll_handle.scroll_to_item(row, Nearest)`.
pub fn compute_scroll_row(selected_index: usize, ordered: &[Emoji]) -> usize {
    let cols = GRID_COLS;
    let mut flat_offset: usize = 0;
    let mut row_offset: usize = 0;
    for cat in all_categories() {
        let cat_count = ordered.iter().filter(|e| e.category == cat).count();
        if cat_count == 0 {
            continue;
        }
        if flat_offset + cat_count > selected_index {
            let idx_in_cat = selected_index - flat_offset;
            let cell_row = idx_in_cat / cols;
            row_offset += 1 + cell_row;
            return row_offset;
        }
        row_offset += 1 + cat_count.div_ceil(cols);
        flat_offset += cat_count;
    }
    row_offset
}

/// Return the number of rendered grid rows (headers + cell rows) for a given
/// filter, category, and head. Used by window sizing so the picker is sized
/// by actual visual rows, not raw emoji count.
pub fn filtered_grid_row_count(
    filter: &str,
    selected_category: Option<EmojiCategory>,
    head: &EmojiHead,
) -> usize {
    display_ordered_emojis(filter, selected_category, head).row_count(GRID_COLS)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmojiNavDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmojiCellRow {
    pub visible_row_index: usize,
    pub start_index: usize,
    pub count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmojiGridLayout {
    pub rows: Vec<EmojiCellRow>,
    pub item_to_row: Vec<usize>,
}

/// Build the grid layout for a display order: one header row per section,
/// then its cells in rows of `cols`.
pub fn build_display_grid_layout(display: &DisplayOrder, cols: usize) -> EmojiGridLayout {
    let mut rows: Vec<EmojiCellRow> = Vec::new();
    let mut item_to_row = vec![0usize; display.emojis.len()];
    let mut visible_row_index = 0usize;
    let mut start = 0usize;
    for section in &display.sections {
        visible_row_index += 1;
        let mut row_offset = 0usize;
        while row_offset < section.count {
            let count = (section.count - row_offset).min(cols);
            let start_index = start + row_offset;
            let cell_row_index = rows.len();
            rows.push(EmojiCellRow {
                visible_row_index,
                start_index,
                count,
            });
            for entry in item_to_row.iter_mut().skip(start_index).take(count) {
                *entry = cell_row_index;
            }
            visible_row_index += 1;
            row_offset += count;
        }
        start += section.count;
    }
    EmojiGridLayout { rows, item_to_row }
}

/// The visible row of `selected_index`, counting section header rows.
pub fn compute_display_scroll_row(selected_index: usize, display: &DisplayOrder) -> usize {
    let cols = GRID_COLS;
    let mut start = 0usize;
    let mut row = 0usize;
    for section in &display.sections {
        if selected_index < start + section.count {
            return row + 1 + (selected_index - start) / cols;
        }
        row += 1 + section.count.div_ceil(cols);
        start += section.count;
    }
    row
}

pub fn build_emoji_grid_layout<T>(
    ordered_emojis: &[T],
    cols: usize,
    category_of: impl Fn(&T) -> EmojiCategory,
) -> EmojiGridLayout {
    let mut rows = Vec::new();
    let mut item_to_row = vec![0; ordered_emojis.len()];
    let mut flat_offset = 0usize;
    let mut visible_row_index = 0usize;

    for category in all_categories() {
        let category_count = ordered_emojis[flat_offset..]
            .iter()
            .take_while(|emoji| category_of(emoji) == category)
            .count();

        if category_count == 0 {
            continue;
        }

        // Skip the header row for this category.
        visible_row_index += 1;

        let mut row_offset = 0usize;
        while row_offset < category_count {
            let count = (category_count - row_offset).min(cols);
            let start_index = flat_offset + row_offset;
            let cell_row_index = rows.len();

            rows.push(EmojiCellRow {
                visible_row_index,
                start_index,
                count,
            });

            for entry in item_to_row.iter_mut().skip(start_index).take(count) {
                *entry = cell_row_index;
            }

            visible_row_index += 1;
            row_offset += count;
        }

        flat_offset += category_count;
    }

    EmojiGridLayout { rows, item_to_row }
}

impl EmojiGridLayout {
    pub fn scroll_row_for_index(&self, index: usize) -> usize {
        self.item_to_row
            .get(index)
            .and_then(|row_ix| self.rows.get(*row_ix))
            .map(|row| row.visible_row_index)
            .unwrap_or(0)
    }

    pub fn move_index(&self, index: usize, direction: EmojiNavDirection) -> usize {
        let Some(&cell_row_index) = self.item_to_row.get(index) else {
            return 0;
        };
        let Some(row) = self.rows.get(cell_row_index) else {
            return index;
        };

        let column = index.saturating_sub(row.start_index);

        match direction {
            EmojiNavDirection::Left => {
                if column > 0 {
                    index - 1
                } else if cell_row_index > 0 {
                    let prev = &self.rows[cell_row_index - 1];
                    prev.start_index + prev.count.saturating_sub(1)
                } else {
                    index
                }
            }
            EmojiNavDirection::Right => {
                if column + 1 < row.count {
                    index + 1
                } else if let Some(next) = self.rows.get(cell_row_index + 1) {
                    next.start_index
                } else {
                    index
                }
            }
            EmojiNavDirection::Up => {
                if cell_row_index == 0 {
                    index
                } else {
                    let prev = &self.rows[cell_row_index - 1];
                    prev.start_index + column.min(prev.count.saturating_sub(1))
                }
            }
            EmojiNavDirection::Down => {
                if let Some(next) = self.rows.get(cell_row_index + 1) {
                    next.start_index + column.min(next.count.saturating_sub(1))
                } else {
                    index
                }
            }
        }
    }
}

pub fn search_emojis(query: &str) -> Vec<&'static Emoji> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return EMOJIS.iter().collect();
    }
    let terms: Vec<&str> = query.split_whitespace().collect();
    let mut scored: Vec<(u32, usize, &'static Emoji)> = EMOJIS
        .iter()
        .enumerate()
        .filter_map(|(order, emoji)| {
            match_score(emoji, &query, &terms).map(|score| (score, order, emoji))
        })
        .collect();
    scored.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    scored.into_iter().map(|(_, _, emoji)| emoji).collect()
}

/// How well `emoji` matches: every term must start a word of its name or a
/// keyword. Whole-word and name matches rank above prefixes and keywords.
fn match_score(emoji: &Emoji, query: &str, terms: &[&str]) -> Option<u32> {
    let name = emoji.name.to_lowercase();
    let name_words: Vec<&str> = name
        .split(|ch: char| !ch.is_alphanumeric() && ch != '\'' && ch != '+')
        .filter(|word| !word.is_empty())
        .collect();
    let mut score = 0;
    for term in terms {
        let term_score = if name_words.contains(term) {
            6
        } else if emoji.keywords.contains(term) {
            // The generator stores keywords lowercase.
            5
        } else if name_words.iter().any(|word| word.starts_with(term)) {
            4
        } else if emoji.keywords.iter().any(|keyword| {
            keyword
                .split_whitespace()
                .any(|word| word.starts_with(term))
        }) {
            2
        } else if term.chars().count() >= 3 && name.contains(term) {
            1
        } else {
            return None;
        };
        score += term_score;
    }
    if name == query {
        score += 20;
    } else if name.starts_with(query) {
        score += 8;
    }
    // Shorter names are the more general match ("cat" before "cat with wry smile").
    Some(score * 100 + 100u32.saturating_sub(name.len() as u32))
}

#[cfg(test)]
include!("mod_tests.rs");
