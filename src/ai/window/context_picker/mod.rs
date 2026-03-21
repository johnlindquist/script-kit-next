//! Inline `@` context picker for the AI chat composer.
//!
//! Typing `@` in the composer opens a filtered list of context attachments
//! seeded from the canonical `context_attachment_specs()` plus local files
//! and folders. Accepting a row creates the matching `AiContextPart` and
//! schedules a preflight update automatically.

pub mod types;

mod render;
#[cfg(test)]
mod tests;

use super::*;
use crate::ai::context_contract::{context_attachment_specs, ContextAttachmentKind};
use crate::ai::message_parts::AiContextPart;
use types::{ContextPickerItem, ContextPickerItemKind, ContextPickerState};

/// Minimum query length before file/folder results are shown.
/// Built-in items always show (they are few and high-value).
const FILE_SEARCH_MIN_QUERY_LEN: usize = 2;

/// Maximum number of file/folder results to include.
const FILE_RESULTS_LIMIT: usize = 10;

impl AiApp {
    /// Open the context picker with an initial seed query.
    ///
    /// Called when the user types `@` in the composer. The query is the text
    /// after the `@` character (may be empty on initial trigger).
    pub(super) fn open_context_picker(
        &mut self,
        seed_query: String,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let items = build_picker_items(&seed_query);
        tracing::info!(
            target: "ai",
            query = %seed_query,
            item_count = items.len(),
            selected_index = 0,
            "ai_context_picker_opened"
        );
        self.context_picker = Some(ContextPickerState::new(seed_query, items));
        cx.notify();
    }

    /// Update the picker query and re-rank results.
    pub(super) fn update_context_picker_query(
        &mut self,
        query: String,
        cx: &mut Context<Self>,
    ) {
        let items = build_picker_items(&query);
        if let Some(picker) = self.context_picker.as_mut() {
            picker.query = query;
            picker.items = items;
            // Clamp selected_index to valid range
            if picker.selected_index >= picker.items.len() {
                picker.selected_index = picker.items.len().saturating_sub(1);
            }
        }
        cx.notify();
    }

    /// Accept the currently selected picker row.
    ///
    /// Creates the appropriate `AiContextPart`, adds it to pending parts,
    /// closes the picker, and strips the `@query` prefix from the composer.
    pub(super) fn accept_context_picker_selection(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (part, label, source) = {
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
            };
            let label = part.label().to_string();
            let source = part.source().to_string();
            (part, label, source)
        };

        tracing::info!(
            target: "ai",
            label = %label,
            source = %source,
            "ai_context_picker_accepted"
        );

        // Strip @query text from the composer input
        self.strip_at_query_from_composer(window, cx);

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
                cx.notify();
            }
        }
    }

    /// Move selection down in the picker.
    pub(super) fn context_picker_select_next(&mut self, cx: &mut Context<Self>) {
        if let Some(picker) = self.context_picker.as_mut() {
            if !picker.items.is_empty() {
                picker.selected_index = (picker.selected_index + 1) % picker.items.len();
                cx.notify();
            }
        }
    }

    /// Whether the context picker is currently open.
    pub(super) fn is_context_picker_open(&self) -> bool {
        self.context_picker.is_some()
    }

    /// Strip the `@<query>` text from the current composer input.
    ///
    /// Finds the last `@` in the input and removes everything from it
    /// to the current cursor position.
    fn strip_at_query_from_composer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let current_value = self.input_state.read(cx).value().to_string();
        if let Some(at_pos) = current_value.rfind('@') {
            let mut new_value = current_value[..at_pos].to_string();
            // Preserve any text after the query (cursor might not be at end)
            // For simplicity, we strip from `@` to end of current word
            let after_at = &current_value[at_pos + 1..];
            if let Some(space_pos) = after_at.find(char::is_whitespace) {
                new_value.push_str(&after_at[space_pos..]);
            }
            self.set_composer_value(new_value, window, cx);
        }
    }
}

/// Build the ranked list of picker items for a given query.
///
/// Items are grouped: built-ins first, then files, then folders.
/// Within each group, items are sorted by relevance score (descending),
/// with ties broken by original order (stable sort).
pub fn build_picker_items(query: &str) -> Vec<ContextPickerItem> {
    let query_lower = query.to_lowercase();
    let mut items = Vec::new();

    // 1. Built-in items from canonical specs
    for spec in context_attachment_specs() {
        let score = score_builtin(spec, &query_lower);
        if score > 0 || query_lower.is_empty() {
            let subtitle: SharedString = spec
                .mention
                .or(spec.slash_command)
                .unwrap_or(spec.action_title)
                .into();
            items.push(ContextPickerItem {
                id: SharedString::from(format!("builtin:{:?}", spec.kind).to_lowercase()),
                label: spec.label.into(),
                subtitle,
                kind: ContextPickerItemKind::BuiltIn(spec.kind),
                score: if query_lower.is_empty() {
                    100
                } else {
                    score
                },
            });
        }
    }

    // 2. File/folder results (only when query is long enough)
    if query_lower.len() >= FILE_SEARCH_MIN_QUERY_LEN {
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

    items
}

/// Score a built-in spec against the user query.
///
/// Returns 0 if no match, higher values for better matches.
/// Deterministic: same query always produces the same score.
pub fn score_builtin(
    spec: &crate::ai::context_contract::ContextAttachmentSpec,
    query: &str,
) -> u32 {
    if query.is_empty() {
        return 100;
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

    // Exact match on mention (e.g. "selection" for @selection)
    if mention_lower == query {
        return 1000;
    }

    // Prefix match on mention
    if !mention_lower.is_empty() && mention_lower.starts_with(query) {
        return 500 + (100 - query.len().min(99) as u32);
    }

    // Prefix match on label
    if label_lower.starts_with(query) {
        return 400;
    }

    // Prefix match on slash command
    if !slash_lower.is_empty() && slash_lower.starts_with(query) {
        return 300;
    }

    // Substring match on label
    if label_lower.contains(query) {
        return 200;
    }

    // Substring match on mention or slash
    if mention_lower.contains(query) || slash_lower.contains(query) {
        return 100;
    }

    0
}

/// Collect file and folder items from the given directory matching the query.
fn collect_file_items(
    dir: &std::path::Path,
    query: &str,
    items: &mut Vec<ContextPickerItem>,
) {
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

        if is_dir {
            if folder_count < FILE_RESULTS_LIMIT / 2 {
                items.push(ContextPickerItem {
                    id: SharedString::from(format!("folder:{}", path.display())),
                    label: name.into(),
                    subtitle: path.to_string_lossy().to_string().into(),
                    kind: ContextPickerItemKind::Folder(path),
                    score,
                });
                folder_count += 1;
            }
        } else if file_count < FILE_RESULTS_LIMIT / 2 {
            items.push(ContextPickerItem {
                id: SharedString::from(format!("file:{}", path.display())),
                label: name.into(),
                subtitle: path.to_string_lossy().to_string().into(),
                kind: ContextPickerItemKind::File(path),
                score,
            });
            file_count += 1;
        }
    }
}

/// Map item kind to section priority for stable sort grouping.
fn section_priority(kind: &ContextPickerItemKind) -> u8 {
    match kind {
        ContextPickerItemKind::BuiltIn(_) => 0,
        ContextPickerItemKind::File(_) => 1,
        ContextPickerItemKind::Folder(_) => 2,
    }
}
