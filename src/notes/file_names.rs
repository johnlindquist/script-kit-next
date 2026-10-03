//! How a note's file is named: after its title, the way Obsidian, Foam and Logseq name notes,
//! so a `[[Title]]` link finds the same file in every app that opens the folder.

use std::path::Path;

use super::model::{Note, NoteId};

/// Longest file name a title becomes, in characters, before `.md`.
const MAX_STEM_CHARS: usize = 120;
const UNTITLED: &str = "Untitled";

/// The file name a title becomes. Characters that file systems refuse or that `[[links]]` cannot
/// hold are left out: `:` becomes `.`, `/` and `\` become `-`, and `* ? " < > | # ^ [ ]` go.
pub(crate) fn stem_for_title(title: &str) -> String {
    let mut stem = String::with_capacity(title.len());
    for ch in title.chars() {
        match ch {
            ':' => stem.push('.'),
            '/' | '\\' => stem.push('-'),
            '*' | '?' | '"' | '<' | '>' | '|' | '#' | '^' | '[' | ']' => {}
            ch if ch.is_control() => stem.push(' '),
            ch => stem.push(ch),
        }
    }
    let words = stem.split_whitespace().collect::<Vec<_>>().join(" ");
    let limited: String = words
        .trim_start_matches(['.', ' '])
        .chars()
        .take(MAX_STEM_CHARS)
        .collect();
    let limited = limited.trim_end_matches(['.', ' ']);
    if limited.is_empty() {
        UNTITLED.to_string()
    } else {
        limited.to_string()
    }
}

/// A name in `dir` that no other note holds: `base`, then `base 2`, `base 3`, … The note's own
/// file (`current`) never counts against it. Names compare without case, as the Mac's disks do.
pub(crate) fn unused_stem(dir: &Path, base: &str, current: Option<&str>) -> String {
    let taken = |stem: &str| {
        let own = current.is_some_and(|current| current.to_lowercase() == stem.to_lowercase());
        !own && dir.join(format!("{stem}.md")).exists()
    };
    if !taken(base) {
        return base.to_string();
    }
    (2_u32..)
        .map(|n| format!("{base} {n}"))
        .find(|stem| !taken(stem))
        .unwrap_or_else(|| base.to_string())
}

/// Whether a file's name still follows its note's title, so a new title renames the file. Also
/// true of the lowercase-hyphenated names notes had before they were named by title. A file
/// named some other way (by hand, or by another app) keeps its name.
pub(crate) fn stem_follows_title(stem: &str, title: &str) -> bool {
    let named = stem_for_title(title);
    let slug = crate::brain::substrate::slugify(title);
    is_base_or_numbered(stem, &named, ' ')
        || (!slug.is_empty() && is_base_or_numbered(stem, &slug, '-'))
}

fn is_base_or_numbered(stem: &str, base: &str, separator: char) -> bool {
    if stem == base {
        return true;
    }
    stem.strip_prefix(base)
        .and_then(|rest| rest.strip_prefix(separator))
        .is_some_and(|number| !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()))
}

/// The `# Heading` a note starts with, if it starts with one.
pub(crate) fn heading_title(content: &str) -> Option<String> {
    let first = super::metadata::strip_frontmatter(content)
        .lines()
        .find(|line| !line.trim().is_empty())?;
    first
        .trim_start()
        .starts_with("# ")
        .then(|| Note::title_from_content(content))
        .filter(|title| !title.is_empty())
}

/// A note file's title: its `# Heading`, or else its file name, as other apps title it.
pub(crate) fn title_for_file(content: &str, stem: &str) -> String {
    heading_title(content).unwrap_or_else(|| stem.to_string())
}

/// A note's file name split into its folder (`Travel/`, or nothing for the top of the notes
/// folder) and its own name (`Lisbon`).
pub(crate) fn split_folder(stem: &str) -> (&str, &str) {
    match stem.rfind('/') {
        Some(slash) => (&stem[..=slash], &stem[slash + 1..]),
        None => ("", stem),
    }
}

/// The identity of a Markdown file another app wrote, which has none of the store's front
/// matter. It comes from the file name, so it stays the same each time the folder is read; the
/// first save from Script Kit writes it into the file.
pub(crate) fn note_id_for_unmanaged_file(stem: &str) -> NoteId {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(format!("script-kit-note-file:{stem}").as_bytes());
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    NoteId(uuid::Builder::from_sha1_bytes(bytes).into_uuid())
}

/// `content` with every `[[old]]` link (also `[[old|label]]` and `[[old#Heading]]`) pointing at
/// `new` instead, or `None` when nothing links to `old`. Names match the way links resolve:
/// case, spaces and punctuation aside. Text in code (`` `spans` `` and fenced blocks) is left
/// exactly as written.
pub(crate) fn rewrite_wiki_links(content: &str, old: &str, new: &str) -> Option<String> {
    let old_ref = super::metadata::slugify_note_ref(old);
    if old_ref.is_empty() {
        return None;
    }
    let code = code_ranges(content);
    let in_code = |at: usize| code.iter().any(|range| range.contains(&at));
    let mut rewritten = String::with_capacity(content.len());
    let mut changed = false;
    let mut cursor = 0;
    while let Some(relative) = content[cursor..].find("[[") {
        let inner_start = cursor + relative + 2;
        let Some(inner_len) = content[inner_start..].find("]]") else {
            break;
        };
        let inner = &content[inner_start..inner_start + inner_len];
        let target_len = inner.find(['|', '#']).unwrap_or(inner.len());
        let target = &inner[..target_len];
        rewritten.push_str(&content[cursor..inner_start]);
        if !in_code(inner_start - 2) && super::metadata::slugify_note_ref(target) == old_ref {
            rewritten.push_str(new);
            rewritten.push_str(&inner[target_len..]);
            changed = true;
        } else {
            rewritten.push_str(inner);
        }
        cursor = inner_start + inner_len;
    }
    rewritten.push_str(&content[cursor..]);
    changed.then_some(rewritten)
}

/// Byte ranges of `content` that are code: fenced blocks (```` ``` ```` or `~~~`, fences
/// included) and inline `` `spans` ``.
fn code_ranges(content: &str) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut fence_start: Option<usize> = None;
    let mut offset = 0;
    for line in content.split_inclusive('\n') {
        let line_start = offset;
        offset += line.len();
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            match fence_start.take() {
                Some(start) => ranges.push(start..offset),
                None => fence_start = Some(line_start),
            }
            continue;
        }
        if fence_start.is_some() {
            continue;
        }
        let mut span_start = None;
        for (index, ch) in line.char_indices() {
            if ch == '`' {
                match span_start.take() {
                    Some(start) => ranges.push(line_start + start..line_start + index + 1),
                    None => span_start = Some(index),
                }
            }
        }
    }
    if let Some(start) = fence_start {
        ranges.push(start..content.len());
    }
    ranges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titles_become_names_other_apps_can_link_to() {
        assert_eq!(stem_for_title("Trip ideas"), "Trip ideas");
        assert_eq!(stem_for_title("Oct 3, 2:32 PM"), "Oct 3, 2.32 PM");
        assert_eq!(stem_for_title("Q3 / Q4 plan?"), "Q3 - Q4 plan");
        assert_eq!(stem_for_title("[[Draft]] #work"), "Draft work");
        assert_eq!(stem_for_title("  ...hidden  "), "hidden");
        assert_eq!(stem_for_title("###"), "Untitled");
        assert_eq!(
            stem_for_title(&"a".repeat(300)).chars().count(),
            MAX_STEM_CHARS
        );
    }

    #[test]
    fn a_taken_name_gets_the_next_number_but_never_counts_the_note_itself() {
        let dir = tempfile::tempdir().expect("dir");
        std::fs::write(dir.path().join("Trip ideas.md"), "x").expect("write");
        std::fs::write(dir.path().join("Trip ideas 2.md"), "x").expect("write");
        assert_eq!(unused_stem(dir.path(), "Trip ideas", None), "Trip ideas 3");
        assert_eq!(
            unused_stem(dir.path(), "Trip ideas", Some("Trip ideas")),
            "Trip ideas"
        );
        assert_eq!(
            unused_stem(dir.path(), "Trip Ideas", Some("Trip ideas")),
            "Trip Ideas"
        );
        assert_eq!(unused_stem(dir.path(), "Lisbon", None), "Lisbon");
    }

    #[test]
    fn only_names_that_follow_the_title_move_with_it() {
        assert!(stem_follows_title("Trip ideas", "Trip ideas"));
        assert!(stem_follows_title("Trip ideas 2", "Trip ideas"));
        assert!(stem_follows_title("Oct 3, 2.32 PM", "Oct 3, 2:32 PM"));
        assert!(stem_follows_title("launch-plan", "Launch plan"));
        assert!(stem_follows_title("launch-plan-3", "Launch plan"));
        assert!(!stem_follows_title("Meeting", "Meeting with Sam"));
        assert!(!stem_follows_title("Trip ideas copy", "Trip ideas"));
    }

    #[test]
    fn renaming_a_note_moves_the_links_that_point_at_it() {
        let day = "- 09:12 [[Oct 3, 2.32 PM]]\n- 09:40 see [[oct 3, 2:32 pm|the plan]] and [[Oct 3, 2.32 PM#Dates]]\n- [[Lisbon]]";
        assert_eq!(
            rewrite_wiki_links(day, "Oct 3, 2.32 PM", "Trip ideas").as_deref(),
            Some(
                "- 09:12 [[Trip ideas]]\n- 09:40 see [[Trip ideas|the plan]] and [[Trip ideas#Dates]]\n- [[Lisbon]]"
            )
        );
        assert_eq!(rewrite_wiki_links(day, "Trip", "Trip ideas"), None);
        assert_eq!(rewrite_wiki_links("[[unclosed", "unclosed", "x"), None);
    }

    #[test]
    fn links_written_as_code_are_left_as_written() {
        let note =
            "See [[Old]].\n```\n[[Old]] in a fence\n```\nand `[[Old]]` inline, then [[Old|again]]";
        assert_eq!(
            rewrite_wiki_links(note, "Old", "New").as_deref(),
            Some("See [[New]].\n```\n[[Old]] in a fence\n```\nand `[[Old]]` inline, then [[New|again]]")
        );
    }
    #[test]
    fn a_note_without_a_heading_is_titled_by_its_file_name() {
        assert_eq!(title_for_file("# Trip ideas\n\nLisbon", "x"), "Trip ideas");
        assert_eq!(
            title_for_file("---\ntags: [a]\n---\n\n# Trip ideas", "x"),
            "Trip ideas"
        );
        assert_eq!(
            title_for_file("Lisbon in spring", "Trip ideas"),
            "Trip ideas"
        );
        assert_eq!(title_for_file("## Packing", "Trip ideas"), "Trip ideas");
        assert_eq!(title_for_file("", "Trip ideas"), "Trip ideas");
    }

    #[test]
    fn an_unmanaged_file_keeps_one_identity_per_name() {
        assert_eq!(
            note_id_for_unmanaged_file("Trip ideas"),
            note_id_for_unmanaged_file("Trip ideas")
        );
        assert_ne!(
            note_id_for_unmanaged_file("Trip ideas"),
            note_id_for_unmanaged_file("Lisbon")
        );
    }
}
