//! Edits made by the focused Markdown editor, in UTF-8 byte ranges understood by GPUI.
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

/// The next checklist item starts unchecked, regardless of the previous item's state.
/// Return on an empty marker exits the list. Other lines keep normal editor Return.
pub(super) fn checklist_return(text: &str, caret: usize) -> Option<(Range<usize>, String)> {
    let before = text.get(..caret)?;
    let after = text.get(caret..)?;
    if !after.is_empty() && !after.starts_with('\n') {
        return None;
    }
    let start = before.rfind('\n').map_or(0, |i| i + 1);
    let line = &before[start..];
    let body = line.trim_start_matches(' ');
    let spaces = line.len() - body.len();
    // Four leading spaces may be an indented code block. Never add Markdown there.
    if spaces > 3 {
        return None;
    }
    let (bullet, rest) = body.split_once(' ')?;
    if !matches!(bullet, "-" | "+" | "*") {
        return None;
    }
    let rest = rest
        .strip_prefix("[ ]")
        .or_else(|| rest.strip_prefix("[x]"))
        .or_else(|| rest.strip_prefix("[X]"))?;
    if !(rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t')) {
        return None;
    }
    if in_fence(&before[..start]) {
        return None;
    }
    if rest.trim().is_empty() {
        return Some((start..caret, String::new()));
    }
    Some((caret..caret, format!("\n{}{bullet} [ ] ", &line[..spaces])))
}

fn in_fence(prefix: &str) -> bool {
    let mut active = None;
    for line in prefix.lines() {
        let body = line.trim_start_matches(' ');
        if line.len() - body.len() > 3 {
            continue;
        }
        let Some(first @ (b'`' | b'~')) = body.bytes().next() else {
            continue;
        };
        let width = body.bytes().take_while(|&b| b == first).count();
        if width < 3 {
            continue;
        }
        match active {
            Some((mark, minimum))
                if mark == first && width >= minimum && body[width..].trim().is_empty() =>
            {
                active = None
            }
            None => active = Some((first, width)),
            _ => (),
        }
    }
    active.is_some()
}

/// macOS transpose: the graphemes on either side of the caret, or the last two at EOL.
/// Do not transpose across lines, nor split an emoji or combining sequence.
pub(super) fn transpose(text: &str, caret: usize) -> Option<(Range<usize>, String)> {
    let before = text.get(..caret)?;
    let after = text.get(caret..)?;
    let start = before.rfind('\n').map_or(0, |i| i + 1);
    let end = after.find('\n').map_or(text.len(), |i| caret + i);
    let line = &text[start..end];
    let col = caret - start;
    let (left, right) = if col == line.len() {
        let mut chars = line.grapheme_indices(true).rev();
        let right = chars.next()?;
        (chars.next()?, right)
    } else {
        let mut chars = line.grapheme_indices(true).peekable();
        loop {
            let left = chars.next()?;
            let right = *chars.peek()?;
            if right.0 == col {
                break (left, right);
            }
            if right.0 > col {
                return None;
            }
        }
    };
    Some((
        start + left.0..start + right.0 + right.1.len(),
        format!("{}{}", right.1, left.1),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn apply(
        s: &str,
        p: usize,
        fnc: fn(&str, usize) -> Option<(Range<usize>, String)>,
    ) -> Option<String> {
        let (r, t) = fnc(s, p)?;
        let mut out = s.to_owned();
        out.replace_range(r, &t);
        Some(out)
    }
    #[test]
    fn checked_and_empty_items_continue_or_exit_without_touching_other_lines() {
        let s = "# Set\n  * [X] Paper\nClosing";
        let p = s.find("\nClosing").unwrap();
        assert_eq!(
            apply(s, p, checklist_return).unwrap(),
            "# Set\n  * [X] Paper\n  * [ ] \nClosing"
        );
        assert_eq!(apply("- [ ] ", 6, checklist_return).unwrap(), "");
        for s in [
            "```md\n- [ ] code",
            "~~~~\n```\n- [ ] literal",
            "    - [ ] example",
            "- not a checkbox",
        ] {
            assert!(checklist_return(s, s.len()).is_none());
        }
        assert!(checklist_return("- [ ] normal", 8).is_none());
    }
    #[test]
    fn transpose_uses_graphemes_and_does_not_cross_a_line() {
        let s = "Pack the lunch bxo";
        assert_eq!(apply(s, s.len(), transpose).unwrap(), "Pack the lunch box");
        let s = "A🇫🇷e\u{301}z";
        let p = "A🇫🇷".len();
        assert_eq!(apply(s, p, transpose).unwrap(), "Ae\u{301}🇫🇷z");
        assert!(transpose("a\nb", 2).is_none());
        assert!(transpose("a\nb", 3).is_none());
        assert!(transpose("a\u{301}b", 1).is_none());
    }
}
