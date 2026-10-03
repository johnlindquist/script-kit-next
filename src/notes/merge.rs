//! Line-by-line three-way merge for a document changed in Script Kit and in another app (Obsidian,
//! Finder, a phone) at the same time.
//!
//! `base` is the text both sides started from, `ours` is the editor's text and `theirs` is what
//! the file holds now. Each side's changes are found as hunks against `base`; hunks that touch
//! different lines are both applied, and a line added right beside a line changed elsewhere is not
//! a clash. When both sides changed the same lines differently, both versions are kept, the
//! editor's first, so nothing anyone typed is lost and the reader can see both.

/// What a merge produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Merged {
    pub(crate) text: String,
    /// Both sides changed the same lines differently, so both versions were kept.
    pub(crate) kept_both: bool,
}

/// Above this many line comparisons the changed middle is treated as one hunk instead of being
/// aligned line by line, which keeps both versions instead of spending unbounded memory.
const MAX_ALIGNMENT_CELLS: usize = 4_000_000;

pub(crate) fn merge_lines(base: &str, ours: &str, theirs: &str) -> Merged {
    if ours == theirs || theirs == base {
        return Merged {
            text: ours.to_string(),
            kept_both: false,
        };
    }
    if ours == base {
        return Merged {
            text: theirs.to_string(),
            kept_both: false,
        };
    }

    let base_lines: Vec<&str> = base.lines().collect();
    let our_lines: Vec<&str> = ours.lines().collect();
    let their_lines: Vec<&str> = theirs.lines().collect();

    let mut hunks: Vec<(Side, Hunk)> = hunks(&base_lines, &our_lines)
        .into_iter()
        .map(|hunk| (Side::Ours, hunk))
        .chain(
            hunks(&base_lines, &their_lines)
                .into_iter()
                .map(|hunk| (Side::Theirs, hunk)),
        )
        .collect();
    // An insertion before line `p` sorts ahead of a change that starts at `p`.
    hunks.sort_by_key(|(side, hunk)| (hunk.start, hunk.end, *side));

    let mut out: Vec<&str> = Vec::with_capacity(our_lines.len().max(their_lines.len()));
    let mut kept_both = false;
    let mut position = 0;
    let mut index = 0;
    while index < hunks.len() {
        let (start, mut end) = (hunks[index].1.start, hunks[index].1.end);
        let mut last = index + 1;
        while last < hunks.len() && touches(start, end, &hunks[last].1) {
            end = end.max(hunks[last].1.end);
            last += 1;
        }
        out.extend_from_slice(&base_lines[position..start]);

        let cluster = &hunks[index..last];
        let our_version = side_version(&base_lines, start, end, cluster, Side::Ours);
        let their_version = side_version(&base_lines, start, end, cluster, Side::Theirs);
        match (our_version, their_version) {
            (Some(ours), None) => out.extend(ours),
            (None, Some(theirs)) => out.extend(theirs),
            (Some(ours), Some(theirs)) if ours == theirs => out.extend(ours),
            (Some(ours), Some(theirs)) => {
                // Lines both sides added at the same spot are additions, not a clash.
                kept_both |= start != end;
                let mut unmatched_theirs = theirs;
                for line in &ours {
                    if let Some(found) = unmatched_theirs.iter().position(|other| other == line) {
                        unmatched_theirs.remove(found);
                    }
                }
                out.extend(ours);
                out.extend(unmatched_theirs);
            }
            (None, None) => out.extend_from_slice(&base_lines[start..end]),
        }
        position = end;
        index = last;
    }
    out.extend_from_slice(&base_lines[position..]);

    let mut text = out.join("\n");
    if !text.is_empty() && (ours.ends_with('\n') || theirs.ends_with('\n')) {
        text.push('\n');
    }
    Merged { text, kept_both }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Side {
    Ours,
    Theirs,
}

/// `base[start..end]` is replaced by `lines`; `start == end` is a pure insertion before `start`.
#[derive(Debug)]
struct Hunk<'a> {
    start: usize,
    end: usize,
    lines: Vec<&'a str>,
}

/// Whether `hunk` belongs to the cluster covering `base[start..end]`: it overlaps those lines, or
/// both are insertions at the same spot. Touching only at an edge is not overlapping.
fn touches(start: usize, end: usize, hunk: &Hunk<'_>) -> bool {
    (hunk.start < end && start < hunk.end)
        || (start == end && hunk.start == start && hunk.end == start)
}

/// One side's text for `base[start..end]`, or `None` when that side changed nothing there.
fn side_version<'a>(
    base: &[&'a str],
    start: usize,
    end: usize,
    cluster: &[(Side, Hunk<'a>)],
    side: Side,
) -> Option<Vec<&'a str>> {
    let mut version = Vec::new();
    let mut position = start;
    let mut changed = false;
    for (_, hunk) in cluster.iter().filter(|(owner, _)| *owner == side) {
        version.extend_from_slice(&base[position..hunk.start]);
        version.extend_from_slice(&hunk.lines);
        position = hunk.end;
        changed = true;
    }
    version.extend_from_slice(&base[position..end]);
    changed.then_some(version)
}

/// The changes that turn `base` into `other`, from a longest common subsequence of lines.
fn hunks<'a>(base: &[&'a str], other: &[&'a str]) -> Vec<Hunk<'a>> {
    let mut hunks = Vec::new();
    let (mut base_at, mut other_at) = (0, 0);
    for (base_match, other_match) in matching_lines(base, other) {
        if base_match > base_at || other_match > other_at {
            hunks.push(Hunk {
                start: base_at,
                end: base_match,
                lines: other[other_at..other_match].to_vec(),
            });
        }
        base_at = base_match + 1;
        other_at = other_match + 1;
    }
    if base_at < base.len() || other_at < other.len() {
        hunks.push(Hunk {
            start: base_at,
            end: base.len(),
            lines: other[other_at..].to_vec(),
        });
    }
    hunks
}

/// Index pairs of lines `a` and `b` share, in order.
fn matching_lines(a: &[&str], b: &[&str]) -> Vec<(usize, usize)> {
    let prefix = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let suffix = a[prefix..]
        .iter()
        .rev()
        .zip(b[prefix..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let middle_a = &a[prefix..a.len() - suffix];
    let middle_b = &b[prefix..b.len() - suffix];

    let mut pairs: Vec<(usize, usize)> = (0..prefix).map(|i| (i, i)).collect();
    let (rows, cols) = (middle_a.len(), middle_b.len());
    if rows > 0 && cols > 0 && rows.saturating_mul(cols) <= MAX_ALIGNMENT_CELLS {
        // lengths[i][j] = longest common subsequence of middle_a[i..] and middle_b[j..].
        let width = cols + 1;
        let mut lengths = vec![0u32; (rows + 1) * width];
        for i in (0..rows).rev() {
            for j in (0..cols).rev() {
                lengths[i * width + j] = if middle_a[i] == middle_b[j] {
                    lengths[(i + 1) * width + j + 1] + 1
                } else {
                    lengths[(i + 1) * width + j].max(lengths[i * width + j + 1])
                };
            }
        }
        let (mut i, mut j) = (0, 0);
        while i < rows && j < cols {
            if middle_a[i] == middle_b[j] {
                pairs.push((prefix + i, prefix + j));
                i += 1;
                j += 1;
            } else if lengths[(i + 1) * width + j] >= lengths[i * width + j + 1] {
                i += 1;
            } else {
                j += 1;
            }
        }
    }
    let (a_tail, b_tail) = (a.len() - suffix, b.len() - suffix);
    pairs.extend((0..suffix).map(|k| (a_tail + k, b_tail + k)));
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn merge(base: &str, ours: &str, theirs: &str) -> Merged {
        merge_lines(base, ours, theirs)
    }

    #[test]
    fn a_line_corrected_elsewhere_and_a_line_typed_here_both_land_in_place() {
        let merged = merge(
            "- 09:00 first\n",
            "- 09:00 first\n- 09:05 typed here\n",
            "- 09:00 first, corrected on phone\n",
        );
        assert_eq!(
            merged.text,
            "- 09:00 first, corrected on phone\n- 09:05 typed here\n"
        );
        assert!(!merged.kept_both);
    }

    #[test]
    fn edits_to_different_lines_both_survive() {
        let merged = merge("a\nb\nc\nd\n", "a edited\nb\nc\nd\n", "a\nb\nc\nd edited\n");
        assert_eq!(merged.text, "a edited\nb\nc\nd edited\n");
        assert!(!merged.kept_both);
    }

    #[test]
    fn lines_added_at_the_end_on_both_sides_are_all_kept_editor_first() {
        let merged = merge("", "morning thought", "09:45 - [ ] buy milk\n");
        assert_eq!(merged.text, "morning thought\n09:45 - [ ] buy milk\n");
        assert!(!merged.kept_both);
    }

    #[test]
    fn the_same_line_changed_differently_keeps_both_versions_and_says_so() {
        let merged = merge(
            "a\nshared\nz\n",
            "a\nshared here\nz\n",
            "a\nshared there\nz\n",
        );
        assert_eq!(merged.text, "a\nshared here\nshared there\nz\n");
        assert!(merged.kept_both);
    }

    #[test]
    fn a_line_deleted_elsewhere_stays_deleted_when_the_editor_did_not_touch_it() {
        let merged = merge("a\nb\nc\n", "a\nb\nc\nd\n", "a\nc\n");
        assert_eq!(merged.text, "a\nc\nd\n");
    }

    #[test]
    fn a_wholesale_rewrite_keeps_every_line_from_both_sides() {
        let merged = merge(
            "line one\nline two",
            "line one edited\nline two",
            "completely different disk content",
        );
        assert_eq!(
            merged.text,
            "line one edited\nline two\ncompletely different disk content"
        );
        assert!(merged.kept_both);
    }

    #[test]
    fn identical_changes_on_both_sides_appear_once() {
        let merged = merge("a\n", "a\nb\n", "a\nb\n");
        assert_eq!(merged.text, "a\nb\n");
    }
}
