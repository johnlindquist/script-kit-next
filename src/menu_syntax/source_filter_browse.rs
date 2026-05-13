//! Source-filter-only browse mode.
//!
//! When the user types a query that is *only* a positive source head —
//! `c:`, `n:`, `f: `, etc. — the launcher list should fill up with that
//! source's default browse rows instead of the ordinary 3-row passive
//! preview. This module owns the detection rules, per-source resolution,
//! and the lazy/cold hint snapshot exposed via state for automation.

use serde::{Deserialize, Serialize};

use crate::config::UnifiedSearchSourceFilterBrowseResolvedConfig;
use crate::menu_syntax::{RootUnifiedSourceFilter, RootUnifiedSourceFilterSet};

/// Decision shape returned by [`SourceFilterBrowseMode::from_query`].
/// Holds the resolved per-source targets so call sites do not have to
/// re-derive them when overriding `max_results` or building the budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceFilterBrowseMode {
    /// Resolved (clamped) global default rows per source.
    pub global_target_visible_rows: usize,
    /// Number of positive includes that actually map to a passive
    /// source. Used to size the passive-budget total so multiple selected
    /// sources can each render their target rows.
    pub active_passive_source_count: usize,
    resolved: UnifiedSearchSourceFilterBrowseResolvedConfig,
}

impl SourceFilterBrowseMode {
    /// Detect source-filter-only browse mode for the current query.
    ///
    /// Active when *all* of the following hold:
    /// - `browse_config.enabled` is true
    /// - `advanced_predicate_active` is false (no `type:`, `shortcut:`, …)
    /// - `search_text.trim().is_empty()`
    /// - `source_filters.has_positive_includes()` — negative-only filter
    ///   sets such as `-n:` do not trigger browse-mode caps.
    pub fn from_query(
        search_text: &str,
        advanced_predicate_active: bool,
        source_filters: &RootUnifiedSourceFilterSet,
        browse_config: &UnifiedSearchSourceFilterBrowseResolvedConfig,
    ) -> Option<Self> {
        if !browse_config.enabled {
            return None;
        }
        if advanced_predicate_active {
            return None;
        }
        if !search_text.trim().is_empty() {
            return None;
        }
        if !source_filters.active() || !source_filters.has_positive_includes() {
            return None;
        }

        let active_passive_source_count = source_filters
            .positive_includes()
            .filter(|source| browse_config.applies_to(*source))
            .count();
        if active_passive_source_count == 0 {
            return None;
        }

        Some(Self {
            global_target_visible_rows: browse_config.target_visible_rows,
            active_passive_source_count,
            resolved: *browse_config,
        })
    }

    /// Does browse mode apply to `source`? Only positive-included sources
    /// with a resolved entry return true.
    pub fn applies_to(&self, source: RootUnifiedSourceFilter) -> bool {
        self.resolved.applies_to(source)
    }

    /// Per-source target row count, falling back to the global target.
    pub fn target_for(&self, source: RootUnifiedSourceFilter) -> Option<usize> {
        self.resolved.target_for(source)
    }
}

/// Status of an active source-filter-only browse hint. Drives the inline
/// affordance shown above the result list — never a selectable row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SourceFilterBrowseHintStatus {
    /// A cache or provider is warming asynchronously. Copy:
    /// `Warming {Source}…`.
    Warming,
    /// Cache is empty or this source needs typed text to return more rows.
    /// Copy: `Type to load more`.
    TypeToLoadMore,
    /// Recent-only source is under target but more requires typed text.
    /// Copy: `Showing recent {Source}; type to load more`.
    ShowingRecent,
}

impl SourceFilterBrowseHintStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Warming => "warming",
            Self::TypeToLoadMore => "typeToLoadMore",
            Self::ShowingRecent => "showingRecent",
        }
    }
}

/// Snapshot exposed via state for automation. Keep this separate from
/// [`crate::menu_syntax::MenuSyntaxMainHintSnapshot`] — source-filter
/// browse hints are not grammar-owned, must not be selectable, and must
/// not displace the main list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFilterBrowseHintSnapshot {
    /// Always `"sourceFilterBrowse"` so listeners can route this surface
    /// distinctly from the menu-syntax main hint.
    pub mode: String,
    /// Human-facing source label, e.g. `Browser History`.
    pub source: String,
    /// Machine-stable status. See [`SourceFilterBrowseHintStatus`].
    pub status: SourceFilterBrowseHintStatus,
    /// Pre-rendered hint text. UI surfaces should render this verbatim.
    pub text: String,
}

impl SourceFilterBrowseHintSnapshot {
    pub fn warming(source: RootUnifiedSourceFilter) -> Self {
        let label = source.label();
        Self {
            mode: "sourceFilterBrowse".to_string(),
            source: label.to_string(),
            status: SourceFilterBrowseHintStatus::Warming,
            text: format!("Warming {}…", label),
        }
    }

    pub fn type_to_load_more(source: RootUnifiedSourceFilter) -> Self {
        Self {
            mode: "sourceFilterBrowse".to_string(),
            source: source.label().to_string(),
            status: SourceFilterBrowseHintStatus::TypeToLoadMore,
            text: "Type to load more".to_string(),
        }
    }

    pub fn showing_recent(source: RootUnifiedSourceFilter) -> Self {
        let label = source.label();
        Self {
            mode: "sourceFilterBrowse".to_string(),
            source: label.to_string(),
            status: SourceFilterBrowseHintStatus::ShowingRecent,
            text: format!("Showing recent {}; type to load more", label),
        }
    }
}
