//! Day Page markdown reference parsing and fragment path resolution.
//!
//! Fragment-reference and kept-URL lines stay plain markdown on disk and in the
//! editor. This module identifies them only for navigation/automation metadata.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use chrono_tz::Tz;

use crate::brain::substrate::BrainFrontmatter;

/// Back affordance when viewing a fragment inline on the Day Page surface.
pub const FRAGMENT_BACK_ID: &str = "day-page-fragment-back";

/// A parsed segment of today's day-page markdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DayPageSegment {
    Plain {
        text: String,
        start_line: usize,
    },
    FragmentRef {
        timestamp: String,
        excerpt: String,
        relative_link: String,
        start_line: usize,
        line_count: usize,
        index: usize,
    },
    KeptUrl {
        timestamp: String,
        url: String,
        start_line: usize,
        index: usize,
    },
}

impl DayPageSegment {
    pub fn start_line(&self) -> usize {
        match self {
            Self::Plain { start_line, .. }
            | Self::FragmentRef { start_line, .. }
            | Self::KeptUrl { start_line, .. } => *start_line,
        }
    }
}

/// Provenance metadata loaded from a fragment file's frontmatter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FragmentProvenance {
    pub source_label: String,
    pub created: DateTime<Utc>,
}

/// Parse day-page content into plain, fragment-reference, and kept-URL segments.
pub fn parse_day_page_segments(content: &str) -> Vec<DayPageSegment> {
    let lines: Vec<&str> = content.split('\n').collect();
    let mut segments = Vec::new();
    let mut plain_buffer = String::new();
    let mut plain_start: Option<usize> = None;
    let mut fragment_index = 0usize;
    let mut url_index = 0usize;
    let mut line_index = 0usize;

    let flush_plain = |segments: &mut Vec<DayPageSegment>,
                       plain_buffer: &mut String,
                       plain_start: &mut Option<usize>| {
        if plain_buffer.is_empty() {
            *plain_start = None;
            return;
        }
        segments.push(DayPageSegment::Plain {
            text: std::mem::take(plain_buffer),
            start_line: plain_start.unwrap_or(0),
        });
        *plain_start = None;
    };

    while line_index < lines.len() {
        let line = lines[line_index];
        if let Some((timestamp, excerpt, relative_link)) =
            parse_fragment_card_lines(&lines, line_index)
        {
            flush_plain(&mut segments, &mut plain_buffer, &mut plain_start);
            segments.push(DayPageSegment::FragmentRef {
                timestamp,
                excerpt,
                relative_link,
                start_line: line_index,
                line_count: 3,
                index: fragment_index,
            });
            fragment_index += 1;
            line_index += 3;
            continue;
        }

        if let Some((timestamp, excerpt, relative_link)) = parse_fragment_markdown_link_line(line) {
            flush_plain(&mut segments, &mut plain_buffer, &mut plain_start);
            segments.push(DayPageSegment::FragmentRef {
                timestamp,
                excerpt,
                relative_link,
                start_line: line_index,
                line_count: 1,
                index: fragment_index,
            });
            fragment_index += 1;
            line_index += 1;
            continue;
        }

        if let Some((timestamp, excerpt)) = parse_fragment_header_line(line) {
            flush_plain(&mut segments, &mut plain_buffer, &mut plain_start);
            if line_index + 1 < lines.len() {
                let link_line = lines[line_index + 1].trim();
                if let Some(relative_link) = parse_fragment_link_line(link_line) {
                    segments.push(DayPageSegment::FragmentRef {
                        timestamp,
                        excerpt,
                        relative_link,
                        start_line: line_index,
                        line_count: 2,
                        index: fragment_index,
                    });
                    fragment_index += 1;
                    line_index += 2;
                    continue;
                }
            }
        }

        if let Some((timestamp, url)) = parse_kept_url_line(line) {
            flush_plain(&mut segments, &mut plain_buffer, &mut plain_start);
            segments.push(DayPageSegment::KeptUrl {
                timestamp,
                url,
                start_line: line_index,
                index: url_index,
            });
            url_index += 1;
            line_index += 1;
            continue;
        }

        if plain_start.is_none() {
            plain_start = Some(line_index);
        }
        if !plain_buffer.is_empty() {
            plain_buffer.push('\n');
        }
        plain_buffer.push_str(line);
        line_index += 1;
    }

    flush_plain(&mut segments, &mut plain_buffer, &mut plain_start);
    segments
}

fn parse_fragment_card_lines(
    lines: &[&str],
    line_index: usize,
) -> Option<(String, String, String)> {
    if line_index + 2 >= lines.len() {
        return None;
    }
    let timestamp = parse_fragment_card_header_line(lines[line_index])?;
    let excerpt = parse_fragment_quote_line(lines[line_index + 1])?;
    let relative_link = parse_fragment_link_line(lines[line_index + 2])?;
    Some((timestamp, excerpt, relative_link))
}

fn parse_fragment_card_header_line(line: &str) -> Option<String> {
    let trimmed = line.trim_end();
    let (timestamp, rest) = trimmed.split_once(' ')?;
    if !is_timestamp(timestamp) || rest.trim() != "Fragment" {
        return None;
    }
    Some(timestamp.to_string())
}

fn parse_fragment_quote_line(line: &str) -> Option<String> {
    let excerpt = line.trim().strip_prefix('>')?.trim();
    if excerpt.is_empty() {
        return None;
    }
    Some(excerpt.to_string())
}

fn parse_fragment_markdown_link_line(line: &str) -> Option<(String, String, String)> {
    let trimmed = line.trim_end();
    let (timestamp, rest) = trimmed.split_once(' ')?;
    if !is_timestamp(timestamp) {
        return None;
    }
    let rest = rest.trim();
    let (excerpt, relative_link) = parse_markdown_link(rest)?;
    parse_fragment_link_line(&relative_link).map(|link| (timestamp.to_string(), excerpt, link))
}

fn parse_fragment_header_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim_end();
    let (timestamp, rest) = trimmed.split_once(" > ")?;
    if !is_timestamp(timestamp) {
        return None;
    }
    let excerpt = rest.trim();
    if excerpt.is_empty() {
        return None;
    }
    Some((timestamp.to_string(), excerpt.to_string()))
}

fn parse_fragment_link_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let relative_link = parse_markdown_link(trimmed)
        .map(|(_label, url)| url)
        .unwrap_or_else(|| trimmed.to_string());
    if relative_link.starts_with("../fragments/") && relative_link.ends_with(".md") {
        Some(relative_link)
    } else {
        None
    }
}

fn parse_kept_url_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim_end();
    let (timestamp, rest) = trimmed.split_once(' ')?;
    if !is_timestamp(timestamp) {
        return None;
    }
    if let Some((_label, url)) = parse_markdown_link(rest.trim()) {
        if is_single_token_http_url(&url) {
            return Some((timestamp.to_string(), url));
        }
    }
    let url = rest.trim();
    if !is_single_token_http_url(url) {
        return None;
    }
    Some((timestamp.to_string(), url.to_string()))
}

fn parse_markdown_link(text: &str) -> Option<(String, String)> {
    let label_start = text.find('[')?;
    if label_start != 0 {
        return None;
    }
    let label_end = text[label_start + 1..].find("](")? + label_start + 1;
    let link_start = label_end + 2;
    let link_end = text[link_start..].rfind(')')? + link_start;
    if link_end + 1 != text.len() {
        return None;
    }
    let label = text[label_start + 1..label_end].trim();
    let url = text[link_start..link_end].trim();
    if label.is_empty() || url.is_empty() {
        return None;
    }
    Some((label.to_string(), url.to_string()))
}

fn is_timestamp(value: &str) -> bool {
    value.len() == 5
        && value.as_bytes().get(2) == Some(&b':')
        && value.chars().all(|ch| ch.is_ascii_digit() || ch == ':')
}

fn is_single_token_http_url(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.split_whitespace().count() != 1 {
        return false;
    }
    trimmed.starts_with("http://") || trimmed.starts_with("https://")
}

/// Resolve a day-page relative fragment link against the bound day file path.
pub fn resolve_fragment_path(day_page_path: &Path, relative_link: &str) -> Option<PathBuf> {
    let brain_dir = day_page_path.parent()?.parent()?;
    let relative = relative_link.strip_prefix("../").unwrap_or(relative_link);
    Some(brain_dir.join(relative))
}

/// Load provenance metadata from a fragment markdown file.
pub fn load_fragment_provenance(fragment_path: &Path) -> Option<FragmentProvenance> {
    let content = std::fs::read_to_string(fragment_path).ok()?;
    let (frontmatter, _) = BrainFrontmatter::parse(&content).ok()?;
    Some(FragmentProvenance {
        source_label: format_source_label(frontmatter.source.as_deref()),
        created: frontmatter.created,
    })
}

/// Human-readable provenance hint for fragment references.
pub fn format_provenance_hint(provenance: &FragmentProvenance, tz: Tz) -> String {
    let time = provenance
        .created
        .with_timezone(&tz)
        .format("%H:%M")
        .to_string();
    format!("{} · {time}", provenance.source_label)
}

fn format_source_label(source: Option<&str>) -> String {
    match source {
        Some(uri) if uri.starts_with("scriptkit://clipboard/") => "Clipboard".to_string(),
        Some(uri) if uri.starts_with("scriptkit://agent-chat/") => "Agent Chat".to_string(),
        Some(uri) if uri.starts_with("scriptkit://") => "Script Kit".to_string(),
        Some(uri) if !uri.is_empty() => uri.to_string(),
        Some(_) | None => "Captured".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use chrono_tz::Tz;

    #[test]
    fn parse_fragment_and_kept_url_segments() {
        let content = "09:00 morning note\n\
            09:15 Fragment\n\
            > First words of the pasted article without cutting mid-word...\n\
            [Open fragment](../fragments/2026-06-11-0942-clipboard.md)\n\
            09:20 [example.com/docs](https://example.com/docs)\n\
            09:30 closing thought";

        let segments = parse_day_page_segments(content);
        assert_eq!(segments.len(), 4);

        match &segments[0] {
            DayPageSegment::Plain { text, .. } => assert_eq!(text, "09:00 morning note"),
            other => panic!("expected plain segment, got {other:?}"),
        }

        match &segments[1] {
            DayPageSegment::FragmentRef {
                excerpt,
                relative_link,
                index,
                line_count,
                ..
            } => {
                assert_eq!(index, &0);
                assert_eq!(line_count, &3);
                assert!(excerpt.contains("First words"));
                assert_eq!(relative_link, "../fragments/2026-06-11-0942-clipboard.md");
            }
            other => panic!("expected fragment segment, got {other:?}"),
        }

        match &segments[2] {
            DayPageSegment::KeptUrl { url, index, .. } => {
                assert_eq!(index, &0);
                assert_eq!(url, "https://example.com/docs");
            }
            other => panic!("expected kept url segment, got {other:?}"),
        }

        match &segments[3] {
            DayPageSegment::Plain { text, .. } => assert_eq!(text, "09:30 closing thought"),
            other => panic!("expected plain segment, got {other:?}"),
        }
    }

    #[test]
    fn parses_legacy_one_line_fragment_reference_for_navigation() {
        let content = "09:15 [First words of the pasted article without cutting mid-word...](../fragments/2026-06-11-0942-clipboard.md)\n";

        let segments = parse_day_page_segments(content);
        assert_eq!(segments.len(), 1);
        match &segments[0] {
            DayPageSegment::FragmentRef {
                excerpt,
                relative_link,
                line_count,
                ..
            } => {
                assert_eq!(line_count, &1);
                assert!(excerpt.contains("First words"));
                assert_eq!(relative_link, "../fragments/2026-06-11-0942-clipboard.md");
            }
            other => panic!("expected fragment segment, got {other:?}"),
        }
    }

    #[test]
    fn parses_legacy_raw_kept_url_lines_for_existing_day_pages() {
        let content = "09:20 https://example.com/docs\n";

        let segments = parse_day_page_segments(content);
        assert_eq!(segments.len(), 1);
        match &segments[0] {
            DayPageSegment::KeptUrl { url, index, .. } => {
                assert_eq!(index, &0);
                assert_eq!(url, "https://example.com/docs");
            }
            other => panic!("expected kept url segment, got {other:?}"),
        }
    }

    #[test]
    fn parses_legacy_two_line_fragment_reference_for_navigation() {
        let content = "09:15 > First words of the pasted article without cutting mid-word...\n\
              ../fragments/2026-06-11-0942-clipboard.md\n";

        let segments = parse_day_page_segments(content);
        assert_eq!(segments.len(), 1);
        match &segments[0] {
            DayPageSegment::FragmentRef {
                excerpt,
                relative_link,
                line_count,
                ..
            } => {
                assert_eq!(line_count, &2);
                assert!(excerpt.contains("First words"));
                assert_eq!(relative_link, "../fragments/2026-06-11-0942-clipboard.md");
            }
            other => panic!("expected fragment segment, got {other:?}"),
        }
    }

    #[test]
    fn resolve_fragment_path_from_day_page() {
        let day = PathBuf::from("/tmp/brain/days/2026-06-11.md");
        let resolved = resolve_fragment_path(&day, "../fragments/2026-06-11-0942-clipboard.md")
            .expect("resolve");
        assert_eq!(
            resolved,
            PathBuf::from("/tmp/brain/fragments/2026-06-11-0942-clipboard.md")
        );
    }

    #[test]
    fn format_provenance_hint_uses_source_and_time() {
        let provenance = FragmentProvenance {
            source_label: "Clipboard".to_string(),
            created: Utc.with_ymd_and_hms(2026, 6, 11, 14, 42, 0).unwrap(),
        };
        let hint = format_provenance_hint(&provenance, Tz::UTC);
        assert_eq!(hint, "Clipboard · 14:42");
    }
}
