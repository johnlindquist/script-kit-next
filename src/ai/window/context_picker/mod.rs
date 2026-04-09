//! Inline `@`/`/` context picker for the AI chat composer.
//!
//! Typing `@` or `/` in the composer opens a filtered list of context
//! attachments seeded from the canonical `context_attachment_specs()` plus
//! local files and folders. Accepting a row creates the matching
//! `AiContextPart` and schedules a preflight update automatically.

pub mod types;

mod render;
#[cfg(test)]
mod tests;

use super::*;
use crate::ai::context_contract::{context_attachment_specs, ContextAttachmentKind};
use crate::ai::message_parts::AiContextPart;
use types::{ContextPickerItem, ContextPickerItemKind, ContextPickerState, ContextPickerTrigger};

use std::sync::OnceLock;

/// Maximum number of file/folder results to include.
const FILE_RESULTS_LIMIT: usize = 10;

/// Parsed trigger + query extracted from the composer input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextTriggerQuery {
    pub trigger: ContextPickerTrigger,
    pub query: String,
}

/// Cursor-aware trigger extraction result with char range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextTriggerQueryAtCursor {
    pub trigger: ContextPickerTrigger,
    pub char_range: std::ops::Range<usize>,
    pub query: String,
}

fn char_to_byte_offset(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(ix, _)| ix)
        .unwrap_or(text.len())
}

/// Extract a trigger query from the composer text at a specific cursor position.
///
/// Shared implementation used by both the AI window picker and ACP.
/// Returns `None` when there is no active trigger before the cursor.
pub(crate) fn extract_context_picker_query_before_cursor(
    input: &str,
    cursor: usize,
) -> Option<ContextTriggerQueryAtCursor> {
    if cursor > input.chars().count() {
        return None;
    }

    let cursor_byte = char_to_byte_offset(input, cursor);
    let before_cursor = &input[..cursor_byte];

    let trigger_pos = before_cursor.rfind(['@', '/'])?;
    let trigger_byte = before_cursor.as_bytes().get(trigger_pos).copied()?;

    let trigger = match trigger_byte {
        b'@' => ContextPickerTrigger::Mention,
        b'/' => ContextPickerTrigger::Slash,
        _ => return None,
    };

    // Trigger must be at start of text or preceded by appropriate chars
    if trigger_pos > 0 {
        let prev = before_cursor.as_bytes()[trigger_pos - 1];
        match trigger_byte {
            // `@` requires non-alnum/underscore before it (reject `me@home`)
            b'@' if prev.is_ascii_alphanumeric() || prev == b'_' => return None,
            // `/` requires whitespace before it (reject `foo/bar`)
            b'/' if prev != b' ' && prev != b'\n' && prev != b'\t' => return None,
            _ => {}
        }
    }

    let query = &before_cursor[trigger_pos + 1..];

    // Reject if whitespace immediately follows trigger
    if query.starts_with(' ') || query.starts_with('\n') || query.starts_with('\t') {
        return None;
    }

    // Reject if query contains another trigger char or any whitespace
    let trigger_char = match trigger {
        ContextPickerTrigger::Mention => '@',
        ContextPickerTrigger::Slash => '/',
    };
    if query.contains(trigger_char) || query.chars().any(char::is_whitespace) {
        return None;
    }

    let trigger_char_idx = before_cursor[..trigger_pos].chars().count();

    tracing::debug!(
        target: "ai",
        ?trigger,
        cursor,
        trigger_char_idx,
        query = %query,
        "context_picker_trigger_extracted"
    );

    Some(ContextTriggerQueryAtCursor {
        trigger,
        char_range: trigger_char_idx..cursor,
        query: query.to_string(),
    })
}

/// Extract a trigger query from the composer text (end-of-string cursor).
///
/// Thin wrapper around `extract_context_picker_query_before_cursor`.
pub(crate) fn extract_context_picker_query(input: &str) -> Option<ContextTriggerQuery> {
    let result = extract_context_picker_query_before_cursor(input, input.chars().count())?;
    Some(ContextTriggerQuery {
        trigger: result.trigger,
        query: result.query,
    })
}

/// Fuzzy match query characters in order against a candidate string.
///
/// Returns the indices of matched characters in the candidate, or `None`
/// if the query cannot be matched in order.
pub(crate) fn match_query_chars(query: &str, candidate: &str) -> Option<Vec<usize>> {
    if query.is_empty() {
        return Some(Vec::new());
    }
    let candidate_chars: Vec<char> = candidate.chars().collect();
    let mut hits = Vec::with_capacity(query.len());
    let mut from = 0usize;
    for needle in query.chars().map(|ch| ch.to_ascii_lowercase()) {
        let mut found = None;
        for (ix, ch) in candidate_chars.iter().enumerate().skip(from) {
            if ch.to_ascii_lowercase() == needle {
                found = Some(ix);
                break;
            }
        }
        let ix = found?;
        hits.push(ix);
        from = ix + 1;
    }
    Some(hits)
}

/// Match query characters against rendered meta text, offsetting indices
/// past any leading `@` or `/` prefix so highlights land on the matched
/// characters rather than the trigger symbol.
pub(crate) fn match_query_chars_in_display_meta(
    query: &str,
    display_meta: &str,
) -> Option<Vec<usize>> {
    if query.is_empty() {
        return Some(Vec::new());
    }
    let prefix_len = display_meta
        .chars()
        .take_while(|ch| *ch == '@' || *ch == '/')
        .count();
    let bare = display_meta.trim_start_matches(['@', '/']);
    let hits = match_query_chars(query, bare)?;
    Some(hits.into_iter().map(|ix| ix + prefix_len).collect())
}

/// A hint chip for the empty state when no results match.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ContextPickerEmptyStateHint {
    /// What is displayed in the hint chip.
    pub display: &'static str,
    /// What is inserted into the composer when clicked.
    pub insertion: &'static str,
}

/// Hint chips for the empty state when no results match.
///
/// These are the canonical entries shared by both the AI window picker and
/// ACP.  `@file:<path>` uses `insertion: "@file:"` so clicking it keeps
/// the picker open for file suggestions instead of fabricating a fake path.
pub(crate) fn empty_state_hints(
    trigger: ContextPickerTrigger,
) -> std::borrow::Cow<'static, [ContextPickerEmptyStateHint]> {
    static MENTION_HINTS: &[ContextPickerEmptyStateHint] = &[
        ContextPickerEmptyStateHint {
            display: "@screenshot",
            insertion: "@screenshot",
        },
        ContextPickerEmptyStateHint {
            display: "@clipboard",
            insertion: "@clipboard",
        },
        ContextPickerEmptyStateHint {
            display: "@git-diff",
            insertion: "@git-diff",
        },
        ContextPickerEmptyStateHint {
            display: "@recent-scripts",
            insertion: "@recent-scripts",
        },
        ContextPickerEmptyStateHint {
            display: "@calendar",
            insertion: "@calendar",
        },
        ContextPickerEmptyStateHint {
            display: "@file:<path>",
            insertion: "@file:",
        },
    ];
    static SLASH_HINTS: &[ContextPickerEmptyStateHint] = &[
        ContextPickerEmptyStateHint {
            display: "/compact",
            insertion: "/compact ",
        },
        ContextPickerEmptyStateHint {
            display: "/clear",
            insertion: "/clear ",
        },
        ContextPickerEmptyStateHint {
            display: "/help",
            insertion: "/help ",
        },
    ];
    let base = match trigger {
        ContextPickerTrigger::Mention => MENTION_HINTS,
        ContextPickerTrigger::Slash => SLASH_HINTS,
    };

    if trigger != ContextPickerTrigger::Mention {
        tracing::debug!(
            target: "ai",
            event = "ai_context_picker_empty_state_hints_selected",
            trigger = ?trigger,
            hint_count = base.len(),
            filtered_hint_count = base.len(),
        );
        return std::borrow::Cow::Borrowed(base);
    }

    let filtered: Vec<ContextPickerEmptyStateHint> = base
        .iter()
        .copied()
        .filter(|hint| {
            crate::ai::context_contract::ContextAttachmentKind::from_mention_line(hint.insertion)
                .map(|kind| kind.provider_data_available())
                .unwrap_or(true)
        })
        .collect();

    tracing::debug!(
        target: "ai",
        event = "ai_context_picker_empty_state_hints_selected",
        trigger = ?trigger,
        hint_count = base.len(),
        filtered_hint_count = filtered.len(),
    );

    std::borrow::Cow::Owned(filtered)
}

// ── Cached built-in picker seeds ──────────────────────────────────────

#[derive(Debug, Clone)]
struct BuiltinPickerSeed {
    kind: ContextAttachmentKind,
    label: &'static str,
    label_lower: String,
    mention_meta: &'static str,
    mention_meta_lower: String,
    slash_meta: &'static str,
    slash_meta_lower: String,
    has_slash_command: bool,
}

fn builtin_picker_seeds() -> &'static [BuiltinPickerSeed] {
    static CACHE: OnceLock<Vec<BuiltinPickerSeed>> = OnceLock::new();
    CACHE.get_or_init(|| {
        context_attachment_specs()
            .iter()
            .map(|spec| {
                let mention_meta = spec
                    .mention
                    .or(spec.slash_command)
                    .unwrap_or(spec.action_title);
                let slash_meta = spec
                    .slash_command
                    .or(spec.mention)
                    .unwrap_or(spec.action_title);
                BuiltinPickerSeed {
                    kind: spec.kind,
                    label: spec.label,
                    label_lower: spec.label.to_lowercase(),
                    mention_meta,
                    mention_meta_lower: mention_meta.to_lowercase(),
                    slash_meta,
                    slash_meta_lower: slash_meta.to_lowercase(),
                    has_slash_command: spec.slash_command.is_some(),
                }
            })
            .collect()
    })
}

fn builtin_seed(kind: ContextAttachmentKind) -> &'static BuiltinPickerSeed {
    builtin_picker_seeds()
        .iter()
        .find(|seed| seed.kind == kind)
        .unwrap_or_else(|| unreachable!("missing BuiltinPickerSeed"))
}

fn file_search_query(trigger: ContextPickerTrigger, query: &str) -> Option<String> {
    if trigger != ContextPickerTrigger::Mention {
        return None;
    }
    let lower = query.trim().to_lowercase();
    if lower == "file" || lower == "file:" {
        return Some(String::new());
    }
    if lower.starts_with("file:") {
        return Some(query.trim().get(5..).unwrap_or_default().to_string());
    }
    None
}

fn split_file_query(base_dir: &std::path::Path, raw_query: &str) -> (std::path::PathBuf, String) {
    if raw_query.is_empty() {
        return (base_dir.to_path_buf(), String::new());
    }
    let query_path = std::path::Path::new(raw_query);
    let ends_with_sep = raw_query.ends_with(std::path::MAIN_SEPARATOR);
    let parent = if ends_with_sep {
        query_path
    } else {
        query_path.parent().unwrap_or(std::path::Path::new(""))
    };
    let name_filter = if ends_with_sep {
        ""
    } else {
        query_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
    };
    let search_dir = if parent.as_os_str().is_empty() {
        base_dir.to_path_buf()
    } else if parent.is_absolute() {
        parent.to_path_buf()
    } else {
        base_dir.join(parent)
    };
    (search_dir, name_filter.to_lowercase())
}

fn score_builtin_seed(
    seed: &BuiltinPickerSeed,
    trigger: ContextPickerTrigger,
    query: &str,
) -> (u32, Vec<usize>, Vec<usize>) {
    if query.is_empty() {
        return (100, Vec::new(), Vec::new());
    }

    let (display_meta, primary, secondary): (&str, &str, &str) = match trigger {
        ContextPickerTrigger::Mention => (
            seed.mention_meta,
            seed.mention_meta_lower.trim_start_matches(['@', '/']),
            seed.slash_meta_lower.trim_start_matches(['@', '/']),
        ),
        ContextPickerTrigger::Slash => (
            seed.slash_meta,
            seed.slash_meta_lower.trim_start_matches(['@', '/']),
            seed.mention_meta_lower.trim_start_matches(['@', '/']),
        ),
    };

    let mut best_score = 0u32;
    let mut best_label_hits = Vec::new();
    let mut best_meta_hits = Vec::new();

    let compute_hits = |q: &str| -> (Vec<usize>, Vec<usize>) {
        (
            match_query_chars(q, seed.label).unwrap_or_default(),
            match_query_chars_in_display_meta(q, display_meta).unwrap_or_default(),
        )
    };

    if primary == query {
        best_score = 1000;
        (best_label_hits, best_meta_hits) = compute_hits(query);
    }
    if best_score < 500 && !primary.is_empty() && primary.starts_with(query) {
        best_score = 500 + (100 - query.len().min(99) as u32);
        (best_label_hits, best_meta_hits) = compute_hits(query);
    }
    if best_score < 500 && !secondary.is_empty() && secondary.starts_with(query) {
        best_score = 500 + (100 - query.len().min(99) as u32);
        (best_label_hits, best_meta_hits) = compute_hits(query);
    }
    if best_score < 400 && seed.label_lower.starts_with(query) {
        best_score = 400;
        (best_label_hits, best_meta_hits) = compute_hits(query);
    }
    if best_score < 200 && seed.label_lower.contains(query) {
        best_score = 200;
        best_label_hits = match_query_chars(query, seed.label).unwrap_or_default();
        best_meta_hits = Vec::new();
    }
    if best_score < 100
        && ((!primary.is_empty() && primary.contains(query))
            || (!secondary.is_empty() && secondary.contains(query)))
    {
        best_score = 100;
        (best_label_hits, best_meta_hits) = compute_hits(query);
    }

    // Fuzzy fallback: admit scattered-character matches when no stronger
    // rule matched (e.g. "gst" → "git-status").
    if best_score == 0 {
        let label_fuzzy = match_query_chars(query, seed.label);
        let meta_fuzzy = match_query_chars_in_display_meta(query, display_meta);
        if label_fuzzy.is_some() || meta_fuzzy.is_some() {
            best_score = 50;
            best_label_hits = label_fuzzy.unwrap_or_default();
            best_meta_hits = meta_fuzzy.unwrap_or_default();
            tracing::debug!(
                target: "ai",
                kind = ?seed.kind,
                query = %query,
                score = best_score,
                label_hits = ?best_label_hits,
                meta_hits = ?best_meta_hits,
                "ai_context_picker_builtin_fuzzy_match"
            );
        }
    }

    (best_score, best_label_hits, best_meta_hits)
}

impl AiApp {
    /// Synchronize the `context_picker_list_state` item count with the
    /// current picker items. Must be called after the picker opens or
    /// its filtered result set changes.
    pub(super) fn sync_context_picker_list_state(&mut self) {
        let item_count = self
            .context_picker
            .as_ref()
            .map(|p| p.items.len())
            .unwrap_or(0);
        let old_count = self.context_picker_list_state.item_count();
        if old_count != item_count {
            self.context_picker_list_state
                .splice(0..old_count, item_count);
        }
        self.context_picker_last_scrolled_index = None;
    }

    /// Scroll the picker list so the currently selected row is visible.
    /// De-duplicates consecutive calls for the same index.
    pub(super) fn reveal_selected_context_picker_item(
        &mut self,
        reason: &'static str,
        cx: &mut Context<Self>,
    ) {
        let Some(target) = self.context_picker.as_ref().map(|p| p.selected_index) else {
            return;
        };
        if self.context_picker_last_scrolled_index == Some(target) {
            return;
        }
        self.context_picker_list_state.scroll_to_reveal_item(target);
        self.context_picker_last_scrolled_index = Some(target);
        tracing::info!(
            target: "ai",
            reason,
            selected_index = target,
            "ai_context_picker_scrolled_to_selected"
        );
        cx.notify();
    }

    /// Open the context picker with an initial seed query.
    pub(super) fn open_context_picker(
        &mut self,
        trigger: ContextPickerTrigger,
        seed_query: String,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let items = build_picker_items(trigger, &seed_query);
        tracing::info!(
            target: "ai",
            ?trigger,
            layout = "dense_monoline_shared",
            item_count = items.len(),
            selected_index = 0,
            "ai_context_picker_opened"
        );
        self.context_picker = Some(ContextPickerState::new(trigger, seed_query, items));
        self.sync_context_picker_list_state();
        self.reveal_selected_context_picker_item("picker_opened", cx);
    }

    /// Update the picker query and re-rank results.
    pub(super) fn update_context_picker_query(
        &mut self,
        trigger: ContextPickerTrigger,
        query: String,
        cx: &mut Context<Self>,
    ) {
        let items = build_picker_items(trigger, &query);
        if let Some(picker) = self.context_picker.as_mut() {
            picker.trigger = trigger;
            picker.query = query.clone();
            picker.items = items;
            picker.selected_index =
                crate::components::inline_dropdown::inline_dropdown_clamp_selected_index(
                    picker.selected_index,
                    picker.items.len(),
                );
            tracing::info!(
                target: "ai",
                ?trigger,
                item_count = picker.items.len(),
                selected_index = picker.selected_index,
                "ai_context_picker_filtered"
            );
        }
        self.sync_context_picker_list_state();
        self.reveal_selected_context_picker_item("picker_filtered", cx);
    }

    /// Accept the currently selected picker row.
    ///
    /// For parts that have a canonical inline token (`part_to_inline_token`),
    /// replaces the active trigger/query text with the short `@token` plus a
    /// trailing space and claims inline ownership. For parts without an inline
    /// representation (e.g. slash commands), falls back to the strip-and-chip
    /// path.
    pub(super) fn accept_context_picker_selection(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (part, label, source, trigger) = {
            let picker = match &self.context_picker {
                Some(p) if !p.items.is_empty() => p,
                _ => return,
            };
            let item = match picker.items.get(picker.selected_index) {
                Some(i) => i,
                None => return,
            };
            let part = match &item.kind {
                ContextPickerItemKind::BuiltIn(kind) => kind.part(),
                ContextPickerItemKind::File(path) => AiContextPart::FilePath {
                    path: path.to_string_lossy().to_string(),
                    label: item.label.to_string(),
                },
                ContextPickerItemKind::Folder(path) => AiContextPart::FilePath {
                    path: path.to_string_lossy().to_string(),
                    label: item.label.to_string(),
                },
                ContextPickerItemKind::SlashCommand(_)
                | ContextPickerItemKind::Portal(_)
                | ContextPickerItemKind::Inert => {
                    // SlashCommand, Portal, and Inert items are ACP-only; the
                    // window picker should never encounter them, but handle
                    // gracefully.
                    return;
                }
            };
            let label = part.label().to_string();
            let source = part.source().to_string();
            let trigger = picker.trigger;
            (part, label, source, trigger)
        };

        let current_value = self.input_state.read(cx).value().to_string();
        let cursor = current_value.chars().count();

        if let Some(token) = crate::ai::context_mentions::part_to_inline_token(&part) {
            let replacement = format!("{token} ");
            let next_value = extract_context_picker_query_before_cursor(&current_value, cursor)
                .filter(|query| query.trigger == trigger)
                .map(|query| {
                    crate::ai::context_mentions::replace_text_in_char_range(
                        &current_value,
                        query.char_range,
                        &replacement,
                    )
                })
                .unwrap_or_else(|| format!("{current_value}{replacement}"));

            tracing::info!(
                target: "ai",
                event = "ai_context_picker_token_inserted",
                ?trigger,
                label = %label,
                source = %source,
                token = %token,
            );

            self.set_composer_value(next_value, window, cx);
            self.inline_owned_context_tokens.insert(token);
            self.add_context_part(part, cx);
            self.sync_inline_mentions(cx);
        } else {
            tracing::info!(
                target: "ai",
                event = "ai_context_picker_non_inline_part_attached",
                ?trigger,
                label = %label,
                source = %source,
            );

            self.strip_context_trigger_from_composer(trigger, window, cx);
            self.add_context_part(part, cx);
        }

        // Close picker
        self.close_context_picker(cx);
    }

    /// Synchronise `pending_context_parts` from the live inline `@mention`
    /// tokens in the composer. Removes stale parts whose token was deleted
    /// and adds new parts for freshly typed tokens.
    pub(super) fn sync_inline_mentions(&mut self, cx: &mut Context<Self>) {
        let text = self.input_state.read(cx).value().to_string();

        let plan = crate::ai::context_mentions::build_inline_mention_sync_plan(
            &text,
            &self.pending_context_parts,
            &self.inline_owned_context_tokens,
        );

        // Remove stale parts in reverse order to preserve indices.
        for ix in plan.stale_indices.iter().rev().copied() {
            self.remove_context_part(ix, cx);
        }
        for part in &plan.added_parts {
            self.add_context_part(part.clone(), cx);
        }

        self.inline_owned_context_tokens
            .retain(|token| plan.desired_tokens.contains(token));
        self.inline_owned_context_tokens
            .extend(plan.added_tokens.iter().cloned());

        // Invalidate preview if it targets a part now hidden inline.
        let visible: std::collections::HashSet<usize> =
            crate::ai::context_mentions::visible_context_chip_indices(
                &text,
                &self.pending_context_parts,
            )
            .into_iter()
            .collect();

        if self
            .context_preview_index
            .is_some_and(|ix| !visible.contains(&ix))
        {
            self.context_preview_index = None;
        }

        tracing::info!(
            target: "ai",
            event = "ai_inline_mentions_synced",
            desired_count = plan.desired_parts.len(),
            added_count = plan.added_parts.len(),
            removed_count = plan.stale_indices.len(),
            token_count = self.inline_owned_context_tokens.len(),
        );

        cx.notify();
    }

    /// Close the context picker without accepting.
    pub(super) fn close_context_picker(&mut self, cx: &mut Context<Self>) {
        if self.context_picker.is_some() {
            tracing::info!(target: "ai", "ai_context_picker_closed");
            self.context_picker = None;
            cx.notify();
        }
    }

    /// Move selection up in the picker.
    pub(super) fn context_picker_select_prev(&mut self, cx: &mut Context<Self>) {
        if let Some(picker) = self.context_picker.as_mut() {
            picker.selected_index = crate::components::inline_dropdown::inline_dropdown_select_prev(
                picker.selected_index,
                picker.items.len(),
            );
        }
        self.reveal_selected_context_picker_item("keyboard_prev", cx);
    }

    /// Move selection down in the picker.
    pub(super) fn context_picker_select_next(&mut self, cx: &mut Context<Self>) {
        if let Some(picker) = self.context_picker.as_mut() {
            picker.selected_index = crate::components::inline_dropdown::inline_dropdown_select_next(
                picker.selected_index,
                picker.items.len(),
            );
        }
        self.reveal_selected_context_picker_item("keyboard_next", cx);
    }

    /// Whether the context picker is currently open.
    pub(super) fn is_context_picker_open(&self) -> bool {
        self.context_picker.is_some()
    }

    /// Strip the trigger character and its query from the current composer input.
    fn strip_context_trigger_from_composer(
        &mut self,
        trigger: ContextPickerTrigger,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let needle = match trigger {
            ContextPickerTrigger::Mention => '@',
            ContextPickerTrigger::Slash => '/',
        };
        let current_value = self.input_state.read(cx).value().to_string();
        if let Some(pos) = current_value.rfind(needle) {
            let mut new_value = current_value[..pos].to_string();
            let after_trigger = &current_value[pos + 1..];
            if let Some(space_pos) = after_trigger.find(char::is_whitespace) {
                new_value.push_str(&after_trigger[space_pos..]);
            }
            self.set_composer_value(new_value, window, cx);
        }
    }
}

/// Populate `items` with built-in context attachment entries and optional
/// file/folder results. Shared by both `build_picker_items` and
/// `build_slash_picker_items`.
fn extend_builtin_picker_items(
    trigger: ContextPickerTrigger,
    query: &str,
    query_lower: &str,
    items: &mut Vec<ContextPickerItem>,
) {
    for seed in builtin_picker_seeds() {
        if trigger == ContextPickerTrigger::Slash && !seed.has_slash_command {
            continue;
        }

        // Hide provider-backed items when no real data exists
        if !seed.kind.provider_data_available() {
            tracing::info!(
                target: "ai",
                event = "ai_context_picker_seed_skipped_provider_unavailable",
                kind = ?seed.kind,
                trigger = ?trigger,
            );
            continue;
        }

        let (score, label_hits, meta_hits) = score_builtin_seed(seed, trigger, query_lower);

        if score == 0 && !query_lower.is_empty() {
            continue;
        }

        let meta = match trigger {
            ContextPickerTrigger::Mention => seed.mention_meta,
            ContextPickerTrigger::Slash => seed.slash_meta,
        };

        items.push(ContextPickerItem {
            id: SharedString::from(format!("builtin:{:?}", seed.kind).to_lowercase()),
            label: SharedString::from(seed.label),
            description: SharedString::from(seed.kind.spec().action_title),
            meta: SharedString::from(meta),
            kind: ContextPickerItemKind::BuiltIn(seed.kind),
            score: if query_lower.is_empty() { 100 } else { score },
            label_highlight_indices: label_hits,
            meta_highlight_indices: meta_hits,
        });
    }

    // File/folder results only when the user explicitly types @file: intent
    if let Some(file_query) = file_search_query(trigger, query) {
        if let Ok(cwd) = std::env::current_dir() {
            collect_file_items(&cwd, &file_query, items);
        }
    } else {
        tracing::debug!(
            target: "ai",
            ?trigger,
            query = %query,
            "ai_context_picker_file_scan_skipped"
        );
    }

    // Portal items — "Browse Files..." / "Browse Clipboard..." for rich browsing.
    // Only in mention mode; slash mode is command-only.
    if trigger == ContextPickerTrigger::Mention {
        inject_portal_items(query_lower, items);
    }
}

/// Inject "Browse Files..." and "Browse Clipboard..." portal items for rich
/// browsing. These open the full built-in view as a temporary portal that
/// returns the selection to the ACP chat.
fn inject_portal_items(query_lower: &str, items: &mut Vec<ContextPickerItem>) {
    use types::PortalKind;

    struct PortalDef {
        kind: PortalKind,
        id: &'static str,
        label: &'static str,
        description: &'static str,
        meta: &'static str,
        match_terms: &'static [&'static str],
    }

    let portals: &[PortalDef] = &[
        PortalDef {
            kind: PortalKind::FileSearch,
            id: "portal:file_search",
            label: "Browse Files\u{2026}",
            description: "Search files with Spotlight and browse folders",
            meta: "@file",
            match_terms: &["file", "files", "browse", "search"],
        },
        PortalDef {
            kind: PortalKind::ClipboardHistory,
            id: "portal:clipboard_history",
            label: "Browse Clipboard\u{2026}",
            description: "Browse clipboard history with previews",
            meta: "@clipboard",
            match_terms: &["clipboard", "clip", "paste"],
        },
        PortalDef {
            kind: PortalKind::AcpHistory,
            id: "portal:acp_history",
            label: "Browse History\u{2026}",
            description: "Search prior ACP conversations",
            meta: "@history",
            match_terms: &["history", "conversation", "chat", "resume", "reuse"],
        },
    ];

    for def in portals {
        let PortalDef {
            kind,
            id,
            label,
            description,
            meta,
            match_terms,
        } = def;
        let (score, label_hits) = if query_lower.is_empty() {
            // Higher than built-in default (100) to appear at top of the list.
            (200u32, Vec::new())
        } else if match_terms.iter().any(|t| t.starts_with(query_lower)) {
            (
                80,
                match_query_chars(query_lower, &label.to_lowercase()).unwrap_or_default(),
            )
        } else if match_terms.iter().any(|t| t.contains(query_lower)) {
            (
                40,
                match_query_chars(query_lower, &label.to_lowercase()).unwrap_or_default(),
            )
        } else if let Some(hits) = match_query_chars(query_lower, &label.to_lowercase()) {
            (20, hits)
        } else {
            continue;
        };

        let meta_hits = match_query_chars_in_display_meta(query_lower, meta).unwrap_or_default();

        items.push(ContextPickerItem {
            id: SharedString::from(*id),
            label: SharedString::from(*label),
            description: SharedString::from(*description),
            meta: SharedString::from(*meta),
            kind: ContextPickerItemKind::Portal(*kind),
            score,
            label_highlight_indices: label_hits,
            meta_highlight_indices: meta_hits,
        });
    }
}

/// Populate `items` with agent slash command entries (e.g. `/compact`,
/// `/clear`). Shared by ACP and any future slash-command surface.
///
/// Uses bare `(name, description)` pairs — all entries get `Default` payload.
fn extend_agent_slash_command_items<'a, I>(
    query_lower: &str,
    commands: I,
    items: &mut Vec<ContextPickerItem>,
) where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    use types::SlashCommandPayload;
    let payloads: Vec<(SlashCommandPayload, String)> = commands
        .into_iter()
        .map(|(name, desc)| {
            (
                SlashCommandPayload::Default {
                    name: name.to_string(),
                },
                desc.to_string(),
            )
        })
        .collect();
    extend_agent_slash_command_items_with_payloads(
        query_lower,
        payloads.iter().map(|(p, d)| (p, d.as_str())),
        items,
    );
}

/// Populate `items` with source-aware slash command entries.
///
/// Each entry carries a `SlashCommandPayload` so duplicate skill slugs
/// from different plugins produce rows with distinct stable IDs.
fn extend_agent_slash_command_items_with_payloads<'a, I>(
    query_lower: &str,
    commands: I,
    items: &mut Vec<ContextPickerItem>,
) where
    I: IntoIterator<Item = (&'a types::SlashCommandPayload, &'a str)>,
{
    for (payload, description) in commands {
        let name = payload.slash_name();
        let name_lower = name.to_lowercase();
        let score = if query_lower.is_empty() {
            50
        } else if name_lower.starts_with(query_lower) {
            90
        } else if name_lower.contains(query_lower) {
            50
        } else if match_query_chars(query_lower, &name_lower).is_some() {
            10
        } else {
            continue;
        };

        let meta_str = match payload {
            types::SlashCommandPayload::Default { .. } => format!("/{name}"),
            _ => format!("/{name} \u{b7} {}", payload.owner_label()),
        };
        let label_hits = if query_lower.is_empty() {
            Vec::new()
        } else {
            match_query_chars(query_lower, name).unwrap_or_default()
        };
        let meta_hits = if query_lower.is_empty() {
            Vec::new()
        } else {
            match_query_chars_in_display_meta(query_lower, &meta_str).unwrap_or_default()
        };

        tracing::debug!(
            item_id = %payload.stable_id(),
            slash_name = %name,
            owner = %payload.owner_label(),
            "acp_slash_picker_entry_built"
        );

        items.push(ContextPickerItem {
            id: SharedString::from(format!("slash-cmd:{}", payload.stable_id())),
            label: SharedString::from(name.to_string()),
            description: SharedString::from(slash_command_description(name, description)),
            meta: SharedString::from(meta_str),
            kind: ContextPickerItemKind::SlashCommand(payload.clone()),
            score,
            label_highlight_indices: label_hits,
            meta_highlight_indices: meta_hits,
        });
    }
}

fn slash_command_description(name: &str, discovered_description: &str) -> String {
    let trimmed = discovered_description.trim();
    if !trimmed.is_empty() {
        return trimmed.to_string();
    }

    match name {
        "compact" => "Compact the conversation to reduce context usage.".to_string(),
        "clear" => "Clear the current conversation from the composer.".to_string(),
        "bug" => "Report a problem with the current session.".to_string(),
        "help" => "Show slash command help and usage guidance.".to_string(),
        "init" => "Initialize the current workspace for the agent.".to_string(),
        "login" => "Authenticate the current agent session.".to_string(),
        "logout" => "Sign out of the current agent session.".to_string(),
        "status" => "Show the current session and account status.".to_string(),
        "cost" => "Show current usage and cost details.".to_string(),
        "doctor" => "Run diagnostics for the current agent setup.".to_string(),
        "review" => "Ask the agent to review the current work.".to_string(),
        "memory" => "Inspect or manage the agent memory store.".to_string(),
        _ => format!("Run /{name}."),
    }
}

/// Sort items by section priority then score (descending).
fn sort_picker_items(items: &mut [ContextPickerItem]) {
    items.sort_by(|a, b| {
        let section_a = section_priority(&a.kind);
        let section_b = section_priority(&b.kind);
        section_a.cmp(&section_b).then(b.score.cmp(&a.score))
    });
}

/// Log the top ranked items for debugging.
fn log_top_ranked_items(items: &[ContextPickerItem]) {
    for (rank, item) in items.iter().enumerate().take(5) {
        tracing::debug!(
            target: "ai",
            rank,
            item_id = %item.id,
            score = item.score,
            label_hits = ?item.label_highlight_indices,
            meta_hits = ?item.meta_highlight_indices,
            "ai_context_picker_ranked_item"
        );
    }
}

/// Build the ranked list of picker items for a given trigger and query.
///
/// Uses the cached `BuiltinPickerSeed` catalog to avoid per-query
/// lowercasing and metadata reconstruction. File results are only
/// included when the query resolves to explicit `@file:` intent.
pub fn build_picker_items(trigger: ContextPickerTrigger, query: &str) -> Vec<ContextPickerItem> {
    let query_lower = query.to_lowercase();
    let mut items = Vec::with_capacity(builtin_picker_seeds().len() + FILE_RESULTS_LIMIT);

    extend_builtin_picker_items(trigger, query, &query_lower, &mut items);
    sort_picker_items(&mut items);

    tracing::debug!(
        target: "ai",
        ?trigger,
        query = %query,
        item_count = items.len(),
        "ai_context_picker_items_built"
    );
    log_top_ranked_items(&items);

    items
}

/// Build a ranked list of picker items for slash mode using only agent slash
/// commands.
///
/// Slash mode is command-only. Context attachments belong behind `@`.
pub fn build_slash_picker_items<'a, I>(query: &str, agent_commands: I) -> Vec<ContextPickerItem>
where
    I: IntoIterator<Item = &'a str>,
{
    build_slash_picker_items_with_descriptions(
        query,
        agent_commands.into_iter().map(|name| (name, "")),
    )
}

pub fn build_slash_picker_items_with_descriptions<'a, I>(
    query: &str,
    agent_commands: I,
) -> Vec<ContextPickerItem>
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let query_lower = query.to_lowercase();
    let commands: Vec<(&str, &str)> = agent_commands.into_iter().collect();
    let command_count = commands.len();
    let mut items = Vec::with_capacity(command_count);

    extend_agent_slash_command_items(&query_lower, commands, &mut items);
    sort_picker_items(&mut items);

    tracing::info!(
        target: "ai",
        query = %query,
        command_count,
        item_count = items.len(),
        "ai_context_picker_slash_items_built"
    );
    log_top_ranked_items(&items);

    items
}

/// Build a ranked list of picker items for slash mode using source-aware
/// payloads. Each payload carries plugin/Claude ownership so duplicate
/// skill slugs produce rows with distinct stable IDs.
pub fn build_slash_picker_items_with_payloads<'a, I>(
    query: &str,
    payload_commands: I,
) -> Vec<ContextPickerItem>
where
    I: IntoIterator<Item = (&'a types::SlashCommandPayload, &'a str)>,
{
    let query_lower = query.to_lowercase();
    let commands: Vec<(&types::SlashCommandPayload, &str)> = payload_commands.into_iter().collect();
    let command_count = commands.len();
    let mut items = Vec::with_capacity(command_count);

    extend_agent_slash_command_items_with_payloads(&query_lower, commands, &mut items);
    sort_picker_items(&mut items);

    tracing::info!(
        target: "ai",
        query = %query,
        command_count,
        item_count = items.len(),
        "ai_context_picker_slash_items_built"
    );
    log_top_ranked_items(&items);

    items
}

/// Score a built-in spec against the user query (mention mode).
pub fn score_builtin(
    spec: &crate::ai::context_contract::ContextAttachmentSpec,
    query: &str,
) -> u32 {
    score_builtin_seed(
        builtin_seed(spec.kind),
        ContextPickerTrigger::Mention,
        &query.to_lowercase(),
    )
    .0
}

/// Score a built-in spec against the user query with a specific trigger mode.
pub fn score_builtin_with_trigger(
    spec: &crate::ai::context_contract::ContextAttachmentSpec,
    trigger: ContextPickerTrigger,
    query: &str,
) -> (u32, Vec<usize>, Vec<usize>) {
    score_builtin_seed(builtin_seed(spec.kind), trigger, &query.to_lowercase())
}

/// Collect file and folder items from the given directory matching the query.
fn collect_file_items(dir: &std::path::Path, raw_query: &str, items: &mut Vec<ContextPickerItem>) {
    let (search_dir, name_filter) = split_file_query(dir, raw_query);

    let read_dir = match std::fs::read_dir(&search_dir) {
        Ok(rd) => rd,
        Err(error) => {
            tracing::debug!(
                target: "ai",
                query = %raw_query,
                dir = %search_dir.display(),
                %error,
                "ai_context_picker_file_scan_failed"
            );
            return;
        }
    };

    let mut entries: Vec<_> = read_dir.flatten().collect();
    entries.sort_by_key(|entry| entry.file_name());

    let mut file_count = 0usize;
    let mut folder_count = 0usize;

    for entry in entries {
        if file_count + folder_count >= FILE_RESULTS_LIMIT {
            break;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }

        let name_lower = name.to_lowercase();
        if !name_filter.is_empty() && !name_lower.contains(&name_filter) {
            continue;
        }

        let path = entry.path();
        let is_dir = path.is_dir();

        let score = if name_filter.is_empty() || name_lower.starts_with(&name_filter) {
            200
        } else {
            100
        };

        let meta = format!("@file:{}", path.display());

        let item = ContextPickerItem {
            id: SharedString::from(format!(
                "{}:{}",
                if is_dir { "folder" } else { "file" },
                path.display()
            )),
            label: SharedString::from(name.clone()),
            description: SharedString::from(path.display().to_string()),
            meta: SharedString::from(meta.clone()),
            kind: if is_dir {
                ContextPickerItemKind::Folder(path.clone())
            } else {
                ContextPickerItemKind::File(path.clone())
            },
            score,
            label_highlight_indices: if name_filter.is_empty() {
                Vec::new()
            } else {
                match_query_chars(&name_filter, &name).unwrap_or_default()
            },
            meta_highlight_indices: if raw_query.is_empty() {
                Vec::new()
            } else {
                match_query_chars(raw_query, &meta).unwrap_or_default()
            },
        };

        if is_dir {
            if folder_count < FILE_RESULTS_LIMIT / 2 {
                items.push(item);
                folder_count += 1;
            }
        } else if file_count < FILE_RESULTS_LIMIT / 2 {
            items.push(item);
            file_count += 1;
        }
    }

    tracing::info!(
        target: "ai",
        query = %raw_query,
        dir = %search_dir.display(),
        file_count,
        folder_count,
        "ai_context_picker_file_scan_complete"
    );
}

/// Map item kind to section priority for stable sort grouping.
fn section_priority(kind: &ContextPickerItemKind) -> u8 {
    match kind {
        ContextPickerItemKind::BuiltIn(_) => 0,
        ContextPickerItemKind::SlashCommand(_) => 1,
        ContextPickerItemKind::File(_) => 2,
        ContextPickerItemKind::Folder(_) => 3,
        ContextPickerItemKind::Portal(_) => 0,
        // Inert rows (loading / empty state) sort last so live results
        // always appear above them.
        ContextPickerItemKind::Inert => 255,
    }
}

// ── Slash picker loading and empty-state rows ───────────────────────

/// Build a non-actionable "Discovering skills…" placeholder row.
///
/// Shown when the ACP slash picker opens before async discovery completes
/// (i.e. `cached_slash_commands` is still empty).
pub(crate) fn slash_picker_loading_row() -> ContextPickerItem {
    tracing::debug!("acp_slash_picker_loading");
    ContextPickerItem {
        id: SharedString::from("slash-loading"),
        label: SharedString::from("Discovering skills\u{2026}"),
        description: SharedString::from("Scanning plugins and Claude skills"),
        meta: SharedString::from(""),
        kind: ContextPickerItemKind::Inert,
        score: 0,
        label_highlight_indices: Vec::new(),
        meta_highlight_indices: Vec::new(),
    }
}

/// Build a non-actionable "No commands or skills found" row.
///
/// Shown when the slash query filters out all discovered commands.
pub(crate) fn slash_picker_empty_row() -> ContextPickerItem {
    tracing::debug!("acp_slash_picker_empty_state");
    ContextPickerItem {
        id: SharedString::from("slash-empty"),
        label: SharedString::from("No commands or skills found"),
        description: SharedString::from("Try another slash name or plugin skill"),
        meta: SharedString::from(""),
        kind: ContextPickerItemKind::Inert,
        score: 0,
        label_highlight_indices: Vec::new(),
        meta_highlight_indices: Vec::new(),
    }
}
