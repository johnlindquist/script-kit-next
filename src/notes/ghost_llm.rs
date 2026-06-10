//! Brain-aware on-device LLM ghost text for the Notes editor.
//!
//! The deterministic predictor in [`super::ghost`] answers instantly from
//! lexical matches over the current note, other notes, and the clipboard.
//! This module is the debounced side-channel that upgrades the hint when
//! determinism has nothing: it folds brain recall (hybrid memory search over
//! everything the user has noted, chatted, and done) plus a bounded excerpt
//! of the current note into a continuation prompt for the local GGUF model
//! (`crate::ai::local_llm`), then sanitizes the model output back into the
//! same `NotesGhostPrediction` shape the deterministic path produces.
//!
//! Privacy: prompt composition filters sensitive-looking lines on the way in
//! (`looks_sensitive`) and the shared sanitizer rejects sensitive or URL-ish
//! output on the way out. Generation is fully on-device; nothing leaves the
//! machine.

use super::ghost::{
    looks_sensitive, NotesGhostPrediction, NotesGhostSourceKind, MAX_NOTES_GHOST_SUFFIX_CHARS,
};

/// Debounce before the LLM side-channel fires (matches the launcher's).
pub(crate) const NOTES_GHOST_LLM_DEBOUNCE_MS: u64 = 320;
/// Max cached LLM suffixes per Notes window.
pub(crate) const NOTES_GHOST_LLM_CACHE_LIMIT: usize = 32;
/// How long a cached LLM suffix stays servable.
pub(crate) const NOTES_GHOST_LLM_CACHE_TTL: std::time::Duration =
    std::time::Duration::from_secs(10 * 60);
/// Minimum whitespace-separated words in the line prefix before the LLM may
/// fire. Shorter prefixes are too ambiguous to be worth a model call.
const MIN_PREFIX_WORDS: usize = 3;
/// Confidence assigned to LLM predictions (deterministic exact-prefix matches
/// score higher; wiki-link completions higher still).
const NOTES_GHOST_LLM_CONFIDENCE: f32 = 0.7;
const MAX_TITLE_CHARS: usize = 120;
const MAX_EXCERPT_BEFORE_CHARS: usize = 700;
const MAX_EXCERPT_AFTER_CHARS: usize = 300;
const MAX_BRAIN_BLOCK_CHARS: usize = 3_000;

/// Whether a line prefix is worth an on-device generation.
pub(crate) fn line_prefix_is_eligible(line_prefix: &str) -> bool {
    let trimmed = line_prefix.trim_start();
    trimmed.split_whitespace().count() >= MIN_PREFIX_WORDS && !looks_sensitive(trimmed)
}

/// A bounded, sensitivity-filtered window of the note around the cursor.
/// `before` excludes the current line prefix (it is sent separately as the
/// text to continue); `after` starts at the cursor.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct NoteExcerpt {
    pub(crate) before: String,
    pub(crate) after: String,
}

/// Extract the excerpt around `cursor` (a byte offset on a char boundary, as
/// guaranteed by `ghost::current_line_prefix` succeeding on the same input).
pub(crate) fn excerpt_around_cursor(editor_text: &str, cursor: usize) -> NoteExcerpt {
    let cursor = cursor.min(editor_text.len());
    if !editor_text.is_char_boundary(cursor) {
        return NoteExcerpt::default();
    }
    let line_start = editor_text[..cursor].rfind('\n').map_or(0, |idx| idx + 1);
    NoteExcerpt {
        before: tail_filtered_lines(&editor_text[..line_start], MAX_EXCERPT_BEFORE_CHARS),
        after: head_filtered_lines(&editor_text[cursor..], MAX_EXCERPT_AFTER_CHARS),
    }
}

/// Compose the continuation prompt. Mirrors the launcher's Cotabby-derived
/// layout: instructions first, context blocks next, and the partial line as
/// the LAST payload with no trailing cue, so small instruct models continue
/// the trailing text directly instead of re-answering.
pub(crate) fn build_notes_ghost_prompt(
    line_prefix: &str,
    note_title: &str,
    excerpt: &NoteExcerpt,
    brain_block: Option<&str>,
) -> String {
    let mut prompt = String::from(
        r#"Task:
- Continue the user's note exactly where it stops.
- This is inline autocomplete in a personal notes editor, not chat. Do not answer or address the user.
- Use the note and the remembered context only as a bias when they help.
- Continue on the same line; output only the text that comes next.
- Return plain text only: no labels, bullets, markdown, quotes, or explanation.
- Keep it short: usually 4 to 14 words.
- Do not output secrets, passwords, paths, URLs, or API keys.
"#,
    );
    let title: String = note_title.trim().chars().take(MAX_TITLE_CHARS).collect();
    if !title.is_empty() && !looks_sensitive(&title) {
        prompt.push_str(&format!("Note title: {title}\n"));
    }
    if !excerpt.before.is_empty() {
        prompt.push_str(&format!(
            "---BEGIN NOTE BEFORE CURSOR---\n{}\n---END NOTE BEFORE CURSOR---\n",
            excerpt.before
        ));
    }
    if !excerpt.after.is_empty() {
        prompt.push_str(&format!(
            "---BEGIN NOTE AFTER CURSOR---\n{}\n---END NOTE AFTER CURSOR---\n",
            excerpt.after
        ));
    }
    if let Some(block) = brain_block {
        let block = head_filtered_lines(block, MAX_BRAIN_BLOCK_CHARS);
        if !block.is_empty() {
            prompt.push_str(&format!(
                "---BEGIN REMEMBERED CONTEXT---\n{block}\n---END REMEMBERED CONTEXT---\n"
            ));
        }
    }
    prompt.push_str(&format!(
        "Continue the partial note line below. Output only the words that follow it.\n\n{}",
        line_prefix.trim_end()
    ));
    prompt
}

/// Sanitize a raw model response into a notes ghost suffix: the shared
/// launcher sanitizer first (label/quote stripping, echo removal, single
/// line, URL/sensitive rejection), then the tighter notes cap.
pub(crate) fn sanitize_notes_llm_suffix(raw_response: &str, line_prefix: &str) -> Option<String> {
    let suffix =
        crate::scripts::search::ghost::sanitize_llm_completion_suffix(raw_response, line_prefix)?;
    let capped: String = suffix.chars().take(MAX_NOTES_GHOST_SUFFIX_CHARS).collect();
    let capped = capped.trim_end().to_string();
    if capped.trim().is_empty() || looks_sensitive(&capped) {
        return None;
    }
    Some(capped)
}

/// Build a Tab-acceptable `Brain` prediction from a raw model response, or
/// `None` when the sanitized suffix is empty/unsafe.
pub(crate) fn llm_prediction_from_response(
    line_prefix: &str,
    raw_response: &str,
    generation: u64,
) -> Option<NotesGhostPrediction> {
    let suffix = sanitize_notes_llm_suffix(raw_response, line_prefix)?;
    Some(prediction_from_suffix(line_prefix, suffix, generation))
}

/// Wrap an already-sanitized suffix (fresh or cached) in a `Brain` prediction
/// bound to the current accept generation.
pub(crate) fn prediction_from_suffix(
    line_prefix: &str,
    suffix: String,
    generation: u64,
) -> NotesGhostPrediction {
    NotesGhostPrediction {
        query_prefix: line_prefix.to_string(),
        suffix,
        source_kind: NotesGhostSourceKind::Brain,
        source_rank: 0,
        confidence: NOTES_GHOST_LLM_CONFIDENCE,
        generation,
        accepts_tab: true,
    }
}

/// Whether a finished LLM result may still be shown. A deterministic
/// prediction that appeared while the model ran always wins, and the editor's
/// current line prefix must still equal the prefix the request was built for
/// (anything else means the user kept typing, moved the cursor, or switched
/// notes).
pub(crate) fn should_apply_llm_result(
    deterministic_present: bool,
    request_prefix: &str,
    current_prefix: Option<&str>,
) -> bool {
    !deterministic_present && current_prefix == Some(request_prefix)
}

/// Last `max_chars` of `text` with sensitive lines removed, truncated at a
/// line boundary so the model never sees a half line of stale context.
fn tail_filtered_lines(text: &str, max_chars: usize) -> String {
    let lines = filtered_lines(text);
    let mut kept: Vec<&str> = Vec::new();
    let mut total = 0usize;
    for line in lines.iter().rev() {
        let cost = line.chars().count() + 1;
        if total + cost > max_chars && !kept.is_empty() {
            break;
        }
        total += cost;
        kept.push(line);
        if total > max_chars {
            break;
        }
    }
    kept.reverse();
    kept.join("\n")
}

/// First `max_chars` of `text` with sensitive lines removed, truncated at a
/// line boundary.
fn head_filtered_lines(text: &str, max_chars: usize) -> String {
    let lines = filtered_lines(text);
    let mut kept: Vec<&str> = Vec::new();
    let mut total = 0usize;
    for line in &lines {
        let cost = line.chars().count() + 1;
        if total + cost > max_chars && !kept.is_empty() {
            break;
        }
        total += cost;
        kept.push(line);
        if total > max_chars {
            break;
        }
    }
    kept.join("\n")
}

fn filtered_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim_end)
        .filter(|line| !looks_sensitive(line))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eligibility_requires_three_words_and_no_secrets() {
        assert!(!line_prefix_is_eligible("call"));
        assert!(!line_prefix_is_eligible("call alice"));
        assert!(line_prefix_is_eligible("call alice about"));
        assert!(!line_prefix_is_eligible("my password is hunter2 and"));
    }

    #[test]
    fn prompt_includes_brain_block_title_and_excerpt() {
        let excerpt = NoteExcerpt {
            before: "Launch checklist".to_string(),
            after: "Remaining items".to_string(),
        };
        let prompt = build_notes_ghost_prompt(
            "Follow up with",
            "Alpha Launch",
            &excerpt,
            Some("- memory: Alice owns the alpha launch blockers"),
        );
        assert!(prompt.contains("Note title: Alpha Launch"));
        assert!(prompt.contains("Launch checklist"));
        assert!(prompt.contains("Remaining items"));
        assert!(prompt.contains("Alice owns the alpha launch blockers"));
        // The partial line is the final payload with no trailing cue.
        assert!(prompt.trim_end().ends_with("Follow up with"));
    }

    #[test]
    fn prompt_omits_empty_sections() {
        let prompt = build_notes_ghost_prompt("note line here", "", &NoteExcerpt::default(), None);
        assert!(!prompt.contains("Note title:"));
        assert!(!prompt.contains("BEGIN NOTE BEFORE CURSOR"));
        assert!(!prompt.contains("BEGIN NOTE AFTER CURSOR"));
        assert!(!prompt.contains("BEGIN REMEMBERED CONTEXT"));
    }

    #[test]
    fn prompt_excludes_sensitive_lines_from_excerpt_and_brain_block() {
        let text = "Plan the rollout\napi_key=abc123\nShip on Friday";
        let excerpt = excerpt_around_cursor(
            &format!("{text}\nFollow up wi"),
            text.len() + "\nFollow up wi".len(),
        );
        assert!(excerpt.before.contains("Plan the rollout"));
        assert!(excerpt.before.contains("Ship on Friday"));
        assert!(!excerpt.before.contains("api_key"));

        let prompt = build_notes_ghost_prompt(
            "Follow up wi",
            "",
            &excerpt,
            Some("token=secret\nAlice owns rollout"),
        );
        assert!(!prompt.contains("token=secret"));
        assert!(prompt.contains("Alice owns rollout"));
    }

    #[test]
    fn excerpt_excludes_current_line_prefix_and_bounds_lengths() {
        let before = "line one\n".repeat(200);
        let text = format!("{before}current li");
        let excerpt = excerpt_around_cursor(&text, text.len());
        assert!(!excerpt.before.contains("current li"));
        assert!(excerpt.before.chars().count() <= 700 + "line one".len() + 1);
        assert!(excerpt.after.is_empty());
    }

    #[test]
    fn sanitize_caps_to_notes_suffix_limit() {
        let long = "word ".repeat(60);
        let suffix = sanitize_notes_llm_suffix(&long, "continue this line").expect("capped");
        assert!(suffix.chars().count() <= MAX_NOTES_GHOST_SUFFIX_CHARS);
        assert!(!suffix.is_empty());
    }

    #[test]
    fn sanitize_rejects_urls_and_secrets() {
        assert!(sanitize_notes_llm_suffix("see https://example.com", "go to").is_none());
        assert!(sanitize_notes_llm_suffix("password is hunter2", "the login").is_none());
    }

    #[test]
    fn llm_prediction_is_tab_acceptable_brain_sourced() {
        let prediction = llm_prediction_from_response("call alice about", "the alpha launch", 7)
            .expect("valid prediction");
        assert_eq!(prediction.source_kind, NotesGhostSourceKind::Brain);
        assert_eq!(prediction.generation, 7);
        assert!(prediction.accepts_tab);
        assert_eq!(prediction.query_prefix, "call alice about");
        assert_eq!(prediction.suffix, " the alpha launch");
    }

    #[test]
    fn llm_prediction_rejects_echo_only_responses() {
        assert!(llm_prediction_from_response("call alice about", "call alice about", 1).is_none());
    }

    #[test]
    fn apply_guard_drops_result_when_deterministic_prediction_appeared() {
        assert!(!should_apply_llm_result(
            true,
            "call alice about",
            Some("call alice about"),
        ));
    }

    #[test]
    fn apply_guard_drops_result_when_line_prefix_changed_or_vanished() {
        assert!(!should_apply_llm_result(
            false,
            "call alice about",
            Some("call alice about the"),
        ));
        assert!(!should_apply_llm_result(false, "call alice about", None));
    }

    #[test]
    fn apply_guard_accepts_result_for_unchanged_prefix_without_deterministic() {
        assert!(should_apply_llm_result(
            false,
            "call alice about",
            Some("call alice about"),
        ));
    }
}
