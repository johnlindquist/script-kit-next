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

/// Minimum query length before file/folder results are shown.
/// Built-in items always show (they are few and high-value).
const FILE_SEARCH_MIN_QUERY_LEN: usize = 2;

/// Maximum number of file/folder results to include.
const FILE_RESULTS_LIMIT: usize = 10;

/// Parsed trigger + query extracted from the composer input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextTriggerQuery {
    pub trigger: ContextPickerTrigger,
    pub query: String,
}

/// Extract a trigger query from the composer text.
///
/// Looks for the last `@` or `/` that is not followed by whitespace.
/// Returns `None` when there is no active trigger.
pub(crate) fn extract_context_picker_query(input: &str) -> Option<ContextTriggerQuery> {
    let trigger_pos = input.rfind(['@', '/'])?;
    let trigger_char = input.as_bytes().get(trigger_pos).copied()?;

    let trigger = match trigger_char {
        b'@' => ContextPickerTrigger::Mention,
        b'/' => ContextPickerTrigger::Slash,
        _ => return None,
    };

    let tail = &input[trigger_pos + 1..];

    // If there's a space right after the trigger, the mention/command is complete
    if tail.starts_with(' ') {
        return None;
    }

    // For `/`, only trigger at start-of-line or after whitespace
    // (avoid triggering on file paths like `foo/bar`)
    if trigger_char == b'/' && trigger_pos > 0 {
        let before = input.as_bytes()[trigger_pos - 1];
        if before != b' ' && before != b'\n' && before != b'\t' {
            return None;
        }
    }

    // Extract word after trigger (up to next space or end)
    let query = match tail.find(char::is_whitespace) {
        Some(end) => &tail[..end],
        None => tail,
    };

    Some(ContextTriggerQuery {
        trigger,
        query: query.to_string(),
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

/// Hint chips for the empty state when no results match.
pub(crate) fn empty_state_hints(trigger: ContextPickerTrigger) -> &'static [&'static str] {
    match trigger {
        ContextPickerTrigger::Mention => &["@context", "@selection", "@browser"],
        ContextPickerTrigger::Slash => &["/context", "/selection", "/browser"],
    }
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
            // Clamp selected_index to valid range
            if picker.selected_index >= picker.items.len() {
                picker.selected_index = picker.items.len().saturating_sub(1);
            }
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
    /// Creates the appropriate `AiContextPart`, adds it to pending parts,
    /// closes the picker, and strips the trigger+query from the composer.
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
                ContextPickerItemKind::SlashCommand(_) => {
                    // SlashCommand items are ACP-only; the window picker
                    // should never encounter them, but handle gracefully.
                    return;
                }
            };
            let label = part.label().to_string();
            let source = part.source().to_string();
            let trigger = picker.trigger;
            (part, label, source, trigger)
        };

        tracing::info!(
            target: "ai",
            ?trigger,
            label = %label,
            source = %source,
            "ai_context_picker_accepted"
        );

        // Strip trigger+query text from the composer input
        self.strip_context_trigger_from_composer(trigger, window, cx);

        // Add the part (dedup + preflight handled internally)
        self.add_context_part(part, cx);

        // Close picker
        self.close_context_picker(cx);
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
            if !picker.items.is_empty() {
                if picker.selected_index > 0 {
                    picker.selected_index -= 1;
                } else {
                    picker.selected_index = picker.items.len() - 1;
                }
            }
        }
        self.reveal_selected_context_picker_item("keyboard_prev", cx);
    }

    /// Move selection down in the picker.
    pub(super) fn context_picker_select_next(&mut self, cx: &mut Context<Self>) {
        if let Some(picker) = self.context_picker.as_mut() {
            if !picker.items.is_empty() {
                picker.selected_index = (picker.selected_index + 1) % picker.items.len();
            }
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

/// Build the ranked list of picker items for a given trigger and query.
///
/// Items are grouped: built-ins first, then files, then folders.
/// Within each group, items are sorted by relevance score (descending),
/// with ties broken by original order (stable sort).
pub fn build_picker_items(trigger: ContextPickerTrigger, query: &str) -> Vec<ContextPickerItem> {
    let query_lower = query.to_lowercase();
    let mut items = Vec::new();

    // 1. Built-in items from canonical specs
    for spec in context_attachment_specs() {
        // In Slash mode, only include specs that have a slash_command
        if trigger == ContextPickerTrigger::Slash && spec.slash_command.is_none() {
            continue;
        }

        let (score, label_hits, meta_hits) =
            score_builtin_with_highlights(spec, trigger, &query_lower);

        if score > 0 || query_lower.is_empty() {
            // Build meta: trigger-aware — mention mode prefers @mention,
            // slash mode prefers /command.
            let meta_str: &str = match trigger {
                ContextPickerTrigger::Mention => spec
                    .mention
                    .or(spec.slash_command)
                    .unwrap_or(spec.action_title),
                ContextPickerTrigger::Slash => spec
                    .slash_command
                    .or(spec.mention)
                    .unwrap_or(spec.action_title),
            };

            items.push(ContextPickerItem {
                id: SharedString::from(format!("builtin:{:?}", spec.kind).to_lowercase()),
                label: spec.label.into(),
                meta: meta_str.into(),
                kind: ContextPickerItemKind::BuiltIn(spec.kind),
                score: if query_lower.is_empty() { 100 } else { score },
                label_highlight_indices: if query_lower.is_empty() {
                    Vec::new()
                } else {
                    label_hits
                },
                meta_highlight_indices: if query_lower.is_empty() {
                    Vec::new()
                } else {
                    meta_hits
                },
            });
        }
    }

    // 2. File/folder results (only when query is long enough, mention mode only)
    if trigger == ContextPickerTrigger::Mention && query_lower.len() >= FILE_SEARCH_MIN_QUERY_LEN {
        if let Ok(cwd) = std::env::current_dir() {
            collect_file_items(&cwd, &query_lower, &mut items);
        }
    }

    // Stable sort: by section priority first (BuiltIn < File < Folder), then by score descending
    items.sort_by(|a, b| {
        let section_a = section_priority(&a.kind);
        let section_b = section_priority(&b.kind);
        section_a.cmp(&section_b).then(b.score.cmp(&a.score))
    });

    // Log ranked items for observability (top 5 only to avoid noise)
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

    items
}

/// Score a built-in spec against the user query, returning (score, label_hits, meta_hits).
///
/// The `trigger` parameter determines which meta string is used for highlight
/// computation so that the returned `meta_hits` indices align with the
/// trigger-aware meta displayed in the picker row.
fn score_builtin_with_highlights(
    spec: &crate::ai::context_contract::ContextAttachmentSpec,
    trigger: ContextPickerTrigger,
    query: &str,
) -> (u32, Vec<usize>, Vec<usize>) {
    if query.is_empty() {
        return (100, Vec::new(), Vec::new());
    }

    let label_lower = spec.label.to_lowercase();
    let mention_lower = spec
        .mention
        .map(|m| m.trim_start_matches('@').to_lowercase())
        .unwrap_or_default();
    let slash_lower = spec
        .slash_command
        .map(|s| s.trim_start_matches('/').to_lowercase())
        .unwrap_or_default();

    // Use trigger-aware meta for highlight computation so indices
    // match what the picker row actually displays.
    let display_meta: &str = match trigger {
        ContextPickerTrigger::Mention => spec
            .mention
            .or(spec.slash_command)
            .unwrap_or(spec.action_title),
        ContextPickerTrigger::Slash => spec
            .slash_command
            .or(spec.mention)
            .unwrap_or(spec.action_title),
    };
    let meta_bare = display_meta.trim_start_matches(['@', '/']);

    let mut best_score = 0u32;
    let mut best_label_hits = Vec::new();
    let mut best_meta_hits = Vec::new();

    // Helper: compute both highlight vectors for the current query.
    let compute_hits = |q: &str| -> (Vec<usize>, Vec<usize>) {
        (
            match_query_chars(q, spec.label).unwrap_or_default(),
            match_query_chars(q, meta_bare).unwrap_or_default(),
        )
    };

    // ── Tier 1 (1000): Exact match on mention or slash command ──
    // In Slash mode, an exact slash-command match is equally strong;
    // in Mention mode, an exact mention match leads.
    if mention_lower == query || slash_lower == query {
        best_score = 1000;
        (best_label_hits, best_meta_hits) = compute_hits(query);
    }

    // ── Tier 2 (500+): Prefix match on trigger-primary identifier ──
    // In Slash mode, slash-command prefix is promoted to the same tier as
    // mention prefix; in Mention mode, mention prefix leads.
    if best_score < 500 {
        let primary_match = match trigger {
            ContextPickerTrigger::Slash => {
                !slash_lower.is_empty() && slash_lower.starts_with(query)
            }
            ContextPickerTrigger::Mention => {
                !mention_lower.is_empty() && mention_lower.starts_with(query)
            }
        };
        if primary_match {
            let s = 500 + (100 - query.len().min(99) as u32);
            if s > best_score {
                best_score = s;
                (best_label_hits, best_meta_hits) = compute_hits(query);
            }
        }
    }

    // ── Tier 2b (500+): Prefix on the secondary identifier ──
    // Mention prefix in slash mode, slash prefix in mention mode.
    if best_score < 500 {
        let secondary_match = match trigger {
            ContextPickerTrigger::Slash => {
                !mention_lower.is_empty() && mention_lower.starts_with(query)
            }
            ContextPickerTrigger::Mention => {
                !slash_lower.is_empty() && slash_lower.starts_with(query)
            }
        };
        if secondary_match {
            let s = 500 + (100 - query.len().min(99) as u32);
            if s > best_score {
                best_score = s;
                (best_label_hits, best_meta_hits) = compute_hits(query);
            }
        }
    }

    // ── Tier 3 (400): Prefix match on label ──
    if best_score < 400 && label_lower.starts_with(query) {
        best_score = 400;
        (best_label_hits, best_meta_hits) = compute_hits(query);
    }

    // ── Tier 4 (200): Substring match on label ──
    if best_score < 200 && label_lower.contains(query) {
        best_score = 200;
        best_label_hits = match_query_chars(query, spec.label).unwrap_or_default();
        best_meta_hits = Vec::new();
    }

    // ── Tier 5 (100): Substring match on mention or slash ──
    if best_score < 100 && (mention_lower.contains(query) || slash_lower.contains(query)) {
        best_score = 100;
        (best_label_hits, best_meta_hits) = compute_hits(query);
    }

    (best_score, best_label_hits, best_meta_hits)
}

/// Score a built-in spec against the user query (mention mode).
///
/// Returns 0 if no match, higher values for better matches.
/// Deterministic: same query always produces the same score.
pub fn score_builtin(
    spec: &crate::ai::context_contract::ContextAttachmentSpec,
    query: &str,
) -> u32 {
    score_builtin_with_highlights(spec, ContextPickerTrigger::Mention, query).0
}

/// Score a built-in spec against the user query with a specific trigger mode.
///
/// Returns `(score, label_highlight_indices, meta_highlight_indices)`.
/// Use this to verify trigger-aware ranking in integration tests.
pub fn score_builtin_with_trigger(
    spec: &crate::ai::context_contract::ContextAttachmentSpec,
    trigger: ContextPickerTrigger,
    query: &str,
) -> (u32, Vec<usize>, Vec<usize>) {
    score_builtin_with_highlights(spec, trigger, query)
}

/// Collect file and folder items from the given directory matching the query.
fn collect_file_items(dir: &std::path::Path, query: &str, items: &mut Vec<ContextPickerItem>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut file_count = 0;
    let mut folder_count = 0;

    for entry in entries.flatten() {
        if file_count + folder_count >= FILE_RESULTS_LIMIT {
            break;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        let name_lower = name.to_lowercase();

        // Skip hidden files/folders
        if name.starts_with('.') {
            continue;
        }

        if !name_lower.contains(query) {
            continue;
        }

        let path = entry.path();
        let is_dir = path.is_dir();

        let score = if name_lower.starts_with(query) {
            200
        } else {
            100
        };

        let label_hits = match_query_chars(query, &name).unwrap_or_default();

        // Use @file: prefix for meta so mention-mode rows display
        // the canonical inline token form.
        let meta = format!("@file:{}", path.display());

        if is_dir {
            if folder_count < FILE_RESULTS_LIMIT / 2 {
                items.push(ContextPickerItem {
                    id: SharedString::from(format!("folder:{}", path.display())),
                    label: name.into(),
                    meta: meta.into(),
                    kind: ContextPickerItemKind::Folder(path),
                    score,
                    label_highlight_indices: label_hits,
                    meta_highlight_indices: Vec::new(),
                });
                folder_count += 1;
            }
        } else if file_count < FILE_RESULTS_LIMIT / 2 {
            items.push(ContextPickerItem {
                id: SharedString::from(format!("file:{}", path.display())),
                label: name.into(),
                meta: meta.into(),
                kind: ContextPickerItemKind::File(path),
                score,
                label_highlight_indices: label_hits,
                meta_highlight_indices: Vec::new(),
            });
            file_count += 1;
        }
    }
}

/// Map item kind to section priority for stable sort grouping.
fn section_priority(kind: &ContextPickerItemKind) -> u8 {
    match kind {
        ContextPickerItemKind::BuiltIn(_) => 0,
        ContextPickerItemKind::SlashCommand(_) => 1,
        ContextPickerItemKind::File(_) => 2,
        ContextPickerItemKind::Folder(_) => 3,
    }
}
