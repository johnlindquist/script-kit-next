//! Configuration type definitions
//!
//! This module contains all the struct and enum definitions for configuration.

// --- merged from part_01.rs ---
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::defaults::*;

// ============================================
// BUILT-IN CONFIG
// ============================================

/// Configuration for built-in features (clipboard history, app launcher, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuiltInConfig {
    /// Enable clipboard history built-in (default: true)
    #[serde(default = "default_clipboard_history")]
    pub clipboard_history: bool,
    /// Enable app launcher built-in (default: true)
    #[serde(default = "default_app_launcher")]
    pub app_launcher: bool,
    /// Enable window switcher built-in (default: true)
    #[serde(default = "default_window_switcher")]
    pub window_switcher: bool,
}

fn default_clipboard_history() -> bool {
    DEFAULT_CLIPBOARD_HISTORY
}
fn default_app_launcher() -> bool {
    DEFAULT_APP_LAUNCHER
}
fn default_window_switcher() -> bool {
    DEFAULT_WINDOW_SWITCHER
}

impl Default for BuiltInConfig {
    fn default() -> Self {
        BuiltInConfig {
            clipboard_history: DEFAULT_CLIPBOARD_HISTORY,
            app_launcher: DEFAULT_APP_LAUNCHER,
            window_switcher: DEFAULT_WINDOW_SWITCHER,
        }
    }
}

// ============================================
// UNIFIED SEARCH CONFIG
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct UnifiedSearchConfig {
    pub enabled: bool,
    #[serde(default = "default_unified_search_passive_source_order")]
    pub passive_source_order: Vec<UnifiedSearchPassiveSource>,
    pub passive_result_limits: UnifiedSearchPassiveResultLimitsConfig,
    pub files: UnifiedSearchFilesConfig,
    pub todos: UnifiedSearchTodosConfig,
    pub notes: UnifiedSearchNotesConfig,
    pub acp_history: UnifiedSearchAcpHistoryConfig,
    pub ai_vault: UnifiedSearchAiVaultConfig,
    pub clipboard_history: UnifiedSearchClipboardHistoryConfig,
    pub dictation_history: UnifiedSearchDictationHistoryConfig,
    pub browser_tabs: UnifiedSearchBrowserTabsConfig,
    pub browser_history: UnifiedSearchBrowserHistoryConfig,
}

impl Default for UnifiedSearchConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_UNIFIED_SEARCH_ENABLED,
            passive_source_order: default_unified_search_passive_source_order(),
            passive_result_limits: UnifiedSearchPassiveResultLimitsConfig::default(),
            files: UnifiedSearchFilesConfig::default(),
            todos: UnifiedSearchTodosConfig::default(),
            notes: UnifiedSearchNotesConfig::default(),
            acp_history: UnifiedSearchAcpHistoryConfig::default(),
            ai_vault: UnifiedSearchAiVaultConfig::default(),
            clipboard_history: UnifiedSearchClipboardHistoryConfig::default(),
            dictation_history: UnifiedSearchDictationHistoryConfig::default(),
            browser_tabs: UnifiedSearchBrowserTabsConfig::default(),
            browser_history: UnifiedSearchBrowserHistoryConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, std::hash::Hash)]
#[serde(rename_all = "camelCase")]
pub enum UnifiedSearchPassiveSource {
    BrowserTabs,
    Todos,
    Notes,
    ClipboardHistory,
    DictationHistory,
    AcpHistory,
    AiVault,
    BrowserHistory,
}

impl UnifiedSearchPassiveSource {
    pub(crate) const DEFAULT_ORDER: [Self; 8] = [
        Self::BrowserTabs,
        Self::Todos,
        Self::Notes,
        Self::ClipboardHistory,
        Self::DictationHistory,
        Self::AcpHistory,
        Self::AiVault,
        Self::BrowserHistory,
    ];
}

fn default_unified_search_passive_source_order() -> Vec<UnifiedSearchPassiveSource> {
    UnifiedSearchPassiveSource::DEFAULT_ORDER.to_vec()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct UnifiedSearchPassiveResultLimitsConfig {
    pub max_total_results: usize,
    pub max_total_results_when_primary_visible: usize,
    pub max_results_per_source_when_primary_visible: usize,
}

impl Default for UnifiedSearchPassiveResultLimitsConfig {
    fn default() -> Self {
        Self {
            max_total_results: DEFAULT_UNIFIED_SEARCH_PASSIVE_MAX_TOTAL_RESULTS,
            max_total_results_when_primary_visible:
                DEFAULT_UNIFIED_SEARCH_PASSIVE_MAX_TOTAL_RESULTS_WHEN_PRIMARY_VISIBLE,
            max_results_per_source_when_primary_visible:
                DEFAULT_UNIFIED_SEARCH_PASSIVE_MAX_PER_SOURCE_WHEN_PRIMARY_VISIBLE,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct UnifiedSearchFilesConfig {
    pub enabled: bool,
    pub global_search: bool,
    pub recent_files: bool,
    pub directory_browse: bool,
    pub promotion: RootFilePromotionConfig,
}

impl Default for UnifiedSearchFilesConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_UNIFIED_SEARCH_FILES_ENABLED,
            global_search: DEFAULT_UNIFIED_SEARCH_FILES_GLOBAL_SEARCH,
            recent_files: DEFAULT_UNIFIED_SEARCH_FILES_RECENT_FILES,
            directory_browse: DEFAULT_UNIFIED_SEARCH_FILES_DIRECTORY_BROWSE,
            promotion: RootFilePromotionConfig::Never,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct UnifiedSearchTodosConfig {
    pub enabled: bool,
    pub max_results: usize,
    pub min_query_chars: usize,
}

impl Default for UnifiedSearchTodosConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_results: 10,
            min_query_chars: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct UnifiedSearchAcpHistoryConfig {
    pub enabled: bool,
    pub max_results: usize,
    pub min_query_chars: usize,
}

impl Default for UnifiedSearchAcpHistoryConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_UNIFIED_SEARCH_ACP_HISTORY_ENABLED,
            max_results: DEFAULT_UNIFIED_SEARCH_ACP_HISTORY_MAX_RESULTS,
            min_query_chars: DEFAULT_UNIFIED_SEARCH_ACP_HISTORY_MIN_QUERY_CHARS,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct UnifiedSearchAiVaultConfig {
    pub enabled: bool,
    pub max_results: usize,
    pub min_query_chars: usize,
    pub providers: Vec<AiVaultProvider>,
    pub cache_ttl_ms: u64,
    pub search_content: bool,
    pub resume_terminal: AiVaultResumeTerminal,
    pub exclude_patterns: Vec<String>,
}

impl Default for UnifiedSearchAiVaultConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_UNIFIED_SEARCH_AI_VAULT_ENABLED,
            max_results: DEFAULT_UNIFIED_SEARCH_AI_VAULT_MAX_RESULTS,
            min_query_chars: DEFAULT_UNIFIED_SEARCH_AI_VAULT_MIN_QUERY_CHARS,
            providers: AiVaultProvider::default_root_providers(),
            cache_ttl_ms: DEFAULT_UNIFIED_SEARCH_AI_VAULT_CACHE_TTL_MS,
            search_content: DEFAULT_UNIFIED_SEARCH_AI_VAULT_SEARCH_CONTENT,
            resume_terminal: AiVaultResumeTerminal::default(),
            exclude_patterns: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct UnifiedSearchNotesConfig {
    pub enabled: bool,
    pub max_results: usize,
    pub min_query_chars: usize,
    pub search_content: bool,
}

impl Default for UnifiedSearchNotesConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_UNIFIED_SEARCH_NOTES_ENABLED,
            max_results: DEFAULT_UNIFIED_SEARCH_NOTES_MAX_RESULTS,
            min_query_chars: DEFAULT_UNIFIED_SEARCH_NOTES_MIN_QUERY_CHARS,
            search_content: DEFAULT_UNIFIED_SEARCH_NOTES_SEARCH_CONTENT,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct UnifiedSearchClipboardHistoryConfig {
    pub enabled: bool,
    pub max_results: usize,
    pub min_query_chars: usize,
    pub scan_limit: usize,
}

impl Default for UnifiedSearchClipboardHistoryConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_UNIFIED_SEARCH_CLIPBOARD_HISTORY_ENABLED,
            max_results: DEFAULT_UNIFIED_SEARCH_CLIPBOARD_HISTORY_MAX_RESULTS,
            min_query_chars: DEFAULT_UNIFIED_SEARCH_CLIPBOARD_HISTORY_MIN_QUERY_CHARS,
            scan_limit: DEFAULT_UNIFIED_SEARCH_CLIPBOARD_HISTORY_SCAN_LIMIT,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct UnifiedSearchDictationHistoryConfig {
    pub enabled: bool,
    pub max_results: usize,
    pub min_query_chars: usize,
    pub scan_limit: usize,
}

impl Default for UnifiedSearchDictationHistoryConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_UNIFIED_SEARCH_DICTATION_HISTORY_ENABLED,
            max_results: DEFAULT_UNIFIED_SEARCH_DICTATION_HISTORY_MAX_RESULTS,
            min_query_chars: DEFAULT_UNIFIED_SEARCH_DICTATION_HISTORY_MIN_QUERY_CHARS,
            scan_limit: DEFAULT_UNIFIED_SEARCH_DICTATION_HISTORY_SCAN_LIMIT,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct UnifiedSearchBrowserTabsConfig {
    pub enabled: bool,
    pub max_results: usize,
    pub min_query_chars: usize,
    pub scan_limit: usize,
    pub search_urls: bool,
    pub providers: Vec<BrowserTabProvider>,
    pub cache_ttl_ms: u64,
}

impl Default for UnifiedSearchBrowserTabsConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_UNIFIED_SEARCH_BROWSER_TABS_ENABLED,
            max_results: DEFAULT_UNIFIED_SEARCH_BROWSER_TABS_MAX_RESULTS,
            min_query_chars: DEFAULT_UNIFIED_SEARCH_BROWSER_TABS_MIN_QUERY_CHARS,
            scan_limit: DEFAULT_UNIFIED_SEARCH_BROWSER_TABS_SCAN_LIMIT,
            search_urls: DEFAULT_UNIFIED_SEARCH_BROWSER_TABS_SEARCH_URLS,
            providers: BrowserTabProvider::default_root_providers(),
            cache_ttl_ms: DEFAULT_UNIFIED_SEARCH_BROWSER_TABS_CACHE_TTL_MS,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct UnifiedSearchBrowserHistoryConfig {
    pub enabled: bool,
    pub max_results: usize,
    pub min_query_chars: usize,
    pub max_age_days: u32,
    pub providers: Vec<BrowserHistoryProvider>,
    pub search_urls: bool,
    pub scan_limit: usize,
    pub cache_ttl_ms: u64,
}

impl Default for UnifiedSearchBrowserHistoryConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_UNIFIED_SEARCH_BROWSER_HISTORY_ENABLED,
            max_results: DEFAULT_UNIFIED_SEARCH_BROWSER_HISTORY_MAX_RESULTS,
            min_query_chars: DEFAULT_UNIFIED_SEARCH_BROWSER_HISTORY_MIN_QUERY_CHARS,
            max_age_days: DEFAULT_UNIFIED_SEARCH_BROWSER_HISTORY_MAX_AGE_DAYS,
            providers: BrowserHistoryProvider::default_root_providers(),
            search_urls: DEFAULT_UNIFIED_SEARCH_BROWSER_HISTORY_SEARCH_URLS,
            scan_limit: DEFAULT_UNIFIED_SEARCH_BROWSER_HISTORY_SCAN_LIMIT,
            cache_ttl_ms: DEFAULT_UNIFIED_SEARCH_BROWSER_HISTORY_CACHE_TTL_MS,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, std::hash::Hash)]
#[serde(rename_all = "camelCase")]
pub enum BrowserTabProvider {
    Arc,
    Chrome,
    Brave,
    Edge,
}

impl BrowserTabProvider {
    pub(crate) fn default_root_providers() -> Vec<Self> {
        vec![Self::Arc, Self::Chrome, Self::Brave, Self::Edge]
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, std::hash::Hash)]
#[serde(rename_all = "camelCase")]
pub enum BrowserHistoryProvider {
    Arc,
    Chrome,
    Brave,
    Edge,
}

impl BrowserHistoryProvider {
    pub(crate) fn default_root_providers() -> Vec<Self> {
        vec![Self::Arc, Self::Chrome, Self::Brave, Self::Edge]
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, std::hash::Hash)]
#[serde(rename_all = "camelCase")]
pub enum AiVaultProvider {
    Claude,
    Codex,
    HermesAgent,
    RovoDev,
}

impl AiVaultProvider {
    pub(crate) fn default_root_providers() -> Vec<Self> {
        vec![Self::Claude, Self::Codex, Self::HermesAgent, Self::RovoDev]
    }

    pub(crate) fn cmux_id(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::HermesAgent => "hermes-agent",
            Self::RovoDev => "rovodev",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, std::hash::Hash)]
#[serde(rename_all = "camelCase")]
pub enum AiVaultResumeTerminal {
    #[default]
    Cmux,
    QuickTerminal,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RootFilePromotionConfig {
    #[default]
    Never,
    ExactFilenameOnly,
}

impl From<RootFilePromotionConfig> for crate::file_search::RootFilePromotionPolicy {
    fn from(value: RootFilePromotionConfig) -> Self {
        match value {
            RootFilePromotionConfig::Never => Self::Never,
            RootFilePromotionConfig::ExactFilenameOnly => Self::ExactFilenameOnly,
        }
    }
}

impl UnifiedSearchConfig {
    #[allow(dead_code)]
    pub(crate) fn passive_source_order(&self) -> Vec<UnifiedSearchPassiveSource> {
        let mut seen = std::collections::HashSet::new();
        let mut order = Vec::new();
        for source in &self.passive_source_order {
            if seen.insert(*source) {
                order.push(*source);
            }
        }
        for source in UnifiedSearchPassiveSource::DEFAULT_ORDER {
            if seen.insert(source) {
                order.push(source);
            }
        }
        order
    }

    #[allow(dead_code)]
    pub(crate) fn passive_result_limits(&self) -> UnifiedSearchPassiveResultLimitsConfig {
        UnifiedSearchPassiveResultLimitsConfig {
            max_total_results: self.passive_result_limits.max_total_results.clamp(0, 24),
            max_total_results_when_primary_visible: self
                .passive_result_limits
                .max_total_results_when_primary_visible
                .clamp(0, 12),
            max_results_per_source_when_primary_visible: self
                .passive_result_limits
                .max_results_per_source_when_primary_visible
                .clamp(0, 5),
        }
    }

    pub fn root_file_section_options(&self) -> crate::file_search::RootFileSectionOptions {
        crate::file_search::RootFileSectionOptions {
            files_enabled: self.enabled && self.files.enabled,
            recent_files_enabled: self.enabled && self.files.enabled && self.files.recent_files,
            global_search_enabled: self.enabled && self.files.enabled && self.files.global_search,
            directory_browse_enabled: self.enabled
                && self.files.enabled
                && self.files.directory_browse,
            promotion_policy: self.files.promotion.into(),
            query_intent: crate::file_search::RootFileQueryIntent::OrdinaryRoot,
            source_filter_browse_target_visible_rows: None,
            source_chip_visible_limit: None,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn acp_history_section_options(
        &self,
    ) -> crate::ai::acp::history::RootAcpHistorySectionOptions {
        crate::ai::acp::history::RootAcpHistorySectionOptions {
            enabled: self.enabled && self.acp_history.enabled,
            max_results: self.acp_history.max_results.clamp(1, 5),
            min_query_chars: self.acp_history.min_query_chars.clamp(2, 32),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn ai_vault_section_options(&self) -> crate::ai_vault::RootAiVaultSectionOptions {
        crate::ai_vault::RootAiVaultSectionOptions {
            enabled: self.enabled && self.ai_vault.enabled,
            max_results: self.ai_vault.max_results.clamp(1, 5),
            min_query_chars: self.ai_vault.min_query_chars.clamp(3, 32),
            providers: self.ai_vault.providers.clone(),
            cache_ttl_ms: self.ai_vault.cache_ttl_ms.clamp(5_000, 120_000),
            search_content: self.ai_vault.search_content,
            exclude_patterns: self.ai_vault.exclude_patterns.clone(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn notes_section_options(&self) -> crate::notes::RootNotesSectionOptions {
        crate::notes::RootNotesSectionOptions {
            enabled: self.enabled && self.notes.enabled,
            max_results: self.notes.max_results.clamp(1, 5),
            min_query_chars: self.notes.min_query_chars.clamp(2, 32),
            search_content: self.notes.search_content,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn todo_section_options(&self) -> crate::menu_syntax::RootTodoSectionOptions {
        crate::menu_syntax::RootTodoSectionOptions {
            enabled: self.enabled && self.todos.enabled,
            max_results: self.todos.max_results.clamp(1, 24),
            min_query_chars: self.todos.min_query_chars.clamp(0, 32),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn browser_history_section_options(
        &self,
    ) -> crate::browser_history::RootBrowserHistorySectionOptions {
        crate::browser_history::RootBrowserHistorySectionOptions {
            enabled: self.enabled && self.browser_history.enabled,
            max_results: self.browser_history.max_results.clamp(1, 5),
            min_query_chars: self.browser_history.min_query_chars.clamp(4, 32),
            max_age_days: self.browser_history.max_age_days.clamp(1, 365),
            providers: self.browser_history.providers.clone(),
            search_urls: self.browser_history.search_urls,
            scan_limit: self.browser_history.scan_limit.clamp(25, 2_000),
            cache_ttl_ms: self.browser_history.cache_ttl_ms.clamp(5_000, 120_000),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn browser_tabs_section_options(
        &self,
    ) -> crate::browser_tabs::RootBrowserTabsSectionOptions {
        crate::browser_tabs::RootBrowserTabsSectionOptions {
            enabled: self.enabled && self.browser_tabs.enabled,
            max_results: self.browser_tabs.max_results.clamp(1, 5),
            min_query_chars: self.browser_tabs.min_query_chars.clamp(2, 32),
            scan_limit: self.browser_tabs.scan_limit.clamp(10, 250),
            search_urls: self.browser_tabs.search_urls,
            providers: self.browser_tabs.providers.clone(),
            cache_ttl_ms: self.browser_tabs.cache_ttl_ms.clamp(1_000, 60_000),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn dictation_history_section_options(
        &self,
    ) -> crate::dictation::RootDictationHistorySectionOptions {
        crate::dictation::RootDictationHistorySectionOptions {
            enabled: self.enabled && self.dictation_history.enabled,
            max_results: self.dictation_history.max_results.clamp(1, 5),
            min_query_chars: self.dictation_history.min_query_chars.clamp(4, 32),
            scan_limit: self.dictation_history.scan_limit.clamp(25, 200),
        }
    }
}

// ============================================
// PROCESS LIMITS
// ============================================

/// Configuration for process resource limits and health monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessLimits {
    /// Maximum memory usage in MB (None = no limit)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_memory_mb: Option<u64>,
    /// Maximum runtime in seconds (None = no limit)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_runtime_seconds: Option<u64>,
    /// Health check interval in milliseconds (default: 5000)
    #[serde(default = "default_health_check_interval_ms")]
    pub health_check_interval_ms: u64,
}

fn default_health_check_interval_ms() -> u64 {
    DEFAULT_HEALTH_CHECK_INTERVAL_MS
}

impl Default for ProcessLimits {
    fn default() -> Self {
        ProcessLimits {
            max_memory_mb: None,
            max_runtime_seconds: None,
            health_check_interval_ms: DEFAULT_HEALTH_CHECK_INTERVAL_MS,
        }
    }
}

// ============================================
// SUGGESTED CONFIG
// ============================================

/// Configuration for the "Suggested" section (frecency-based ranking)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestedConfig {
    /// Whether the Suggested section is shown (default: true)
    #[serde(default = "default_suggested_enabled")]
    pub enabled: bool,
    /// Maximum number of items to show in SUGGESTED section (default: 10)
    #[serde(default = "default_suggested_max_items")]
    pub max_items: usize,
    /// Minimum score threshold for items to appear in Suggested (default: 0.1)
    /// Items with scores below this won't appear even if there's room
    #[serde(default = "default_suggested_min_score")]
    pub min_score: f64,
    /// Half-life in days for score decay (default: 7.0)
    /// Lower values = more weight on recent items
    /// Higher values = more weight on frequently used items
    #[serde(default = "default_suggested_half_life_days")]
    pub half_life_days: f64,
    /// Whether to track script usage for suggestions (default: true)
    /// If false, no new usage is recorded but existing data is preserved
    #[serde(default = "default_suggested_track_usage")]
    pub track_usage: bool,
    /// Commands to exclude from frecency tracking (default: ["builtin-quit-script-kit"])
    /// These commands won't appear in the Suggested section
    #[serde(default = "default_suggested_excluded_commands")]
    pub excluded_commands: Vec<String>,
}

fn default_suggested_enabled() -> bool {
    DEFAULT_SUGGESTED_ENABLED
}
fn default_suggested_max_items() -> usize {
    DEFAULT_SUGGESTED_MAX_ITEMS
}
fn default_suggested_min_score() -> f64 {
    DEFAULT_SUGGESTED_MIN_SCORE
}
fn default_suggested_half_life_days() -> f64 {
    DEFAULT_SUGGESTED_HALF_LIFE_DAYS
}
fn default_suggested_track_usage() -> bool {
    DEFAULT_SUGGESTED_TRACK_USAGE
}
fn default_suggested_excluded_commands() -> Vec<String> {
    DEFAULT_FRECENCY_EXCLUDED_COMMANDS
        .iter()
        .map(|s| s.to_string())
        .collect()
}

impl Default for SuggestedConfig {
    fn default() -> Self {
        SuggestedConfig {
            enabled: DEFAULT_SUGGESTED_ENABLED,
            max_items: DEFAULT_SUGGESTED_MAX_ITEMS,
            min_score: DEFAULT_SUGGESTED_MIN_SCORE,
            half_life_days: DEFAULT_SUGGESTED_HALF_LIFE_DAYS,
            track_usage: DEFAULT_SUGGESTED_TRACK_USAGE,
            excluded_commands: default_suggested_excluded_commands(),
        }
    }
}

// ============================================
// CONTENT PADDING
// ============================================

/// Content padding configuration for prompts (terminal, editor, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPadding {
    #[serde(default = "default_padding_top")]
    pub top: f32,
    #[serde(default = "default_padding_left")]
    pub left: f32,
    #[serde(default = "default_padding_right")]
    pub right: f32,
}

fn default_padding_top() -> f32 {
    DEFAULT_PADDING_TOP
}
fn default_padding_left() -> f32 {
    DEFAULT_PADDING_LEFT
}
fn default_padding_right() -> f32 {
    DEFAULT_PADDING_RIGHT
}

impl Default for ContentPadding {
    fn default() -> Self {
        ContentPadding {
            top: DEFAULT_PADDING_TOP,
            left: DEFAULT_PADDING_LEFT,
            right: DEFAULT_PADDING_RIGHT,
        }
    }
}

// ============================================
// WATCHER + LAYOUT CONFIG
// ============================================

/// File watcher tuning values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatcherConfig {
    /// Debounce window for file-system events.
    #[serde(default = "default_watcher_debounce_ms")]
    pub debounce_ms: u64,
    /// Event storm threshold before collapsing to full reload.
    #[serde(default = "default_watcher_storm_threshold")]
    pub storm_threshold: usize,
    /// Initial supervisor restart delay.
    #[serde(default = "default_watcher_initial_backoff_ms")]
    pub initial_backoff_ms: u64,
    /// Maximum supervisor restart delay.
    #[serde(default = "default_watcher_max_backoff_ms")]
    pub max_backoff_ms: u64,
    /// Maximum consecutive notify errors before restart.
    #[serde(default = "default_watcher_max_notify_errors")]
    pub max_notify_errors: u32,
}

fn default_watcher_debounce_ms() -> u64 {
    DEFAULT_WATCHER_DEBOUNCE_MS
}
fn default_watcher_storm_threshold() -> usize {
    DEFAULT_WATCHER_STORM_THRESHOLD
}
fn default_watcher_initial_backoff_ms() -> u64 {
    DEFAULT_WATCHER_INITIAL_BACKOFF_MS
}
fn default_watcher_max_backoff_ms() -> u64 {
    DEFAULT_WATCHER_MAX_BACKOFF_MS
}
fn default_watcher_max_notify_errors() -> u32 {
    DEFAULT_WATCHER_MAX_NOTIFY_ERRORS
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            debounce_ms: DEFAULT_WATCHER_DEBOUNCE_MS,
            storm_threshold: DEFAULT_WATCHER_STORM_THRESHOLD,
            initial_backoff_ms: DEFAULT_WATCHER_INITIAL_BACKOFF_MS,
            max_backoff_ms: DEFAULT_WATCHER_MAX_BACKOFF_MS,
            max_notify_errors: DEFAULT_WATCHER_MAX_NOTIFY_ERRORS,
        }
    }
}

/// Core launcher sizing configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayoutConfig {
    /// Standard list/panel height in pixels.
    #[serde(default = "default_layout_standard_height")]
    pub standard_height: f32,
    /// Full-height content views (editor, terminal) in pixels.
    #[serde(default = "default_layout_max_height")]
    pub max_height: f32,
}

fn default_layout_standard_height() -> f32 {
    DEFAULT_LAYOUT_STANDARD_HEIGHT
}
fn default_layout_max_height() -> f32 {
    DEFAULT_LAYOUT_MAX_HEIGHT
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            standard_height: DEFAULT_LAYOUT_STANDARD_HEIGHT,
            max_height: DEFAULT_LAYOUT_MAX_HEIGHT,
        }
    }
}

/// Theme preset selection loaded from `config.ts`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ThemeSelectionPreferences {
    /// Optional preset identifier (for example: "catppuccin-mocha").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset_id: Option<String>,
}

/// Dictation preferences loaded from `config.ts`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DictationPreferences {
    /// Persisted microphone device ID. `None` means use system default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_device_id: Option<String>,
}

/// Projection of config-backed runtime preferences.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptKitUserPreferences {
    /// Launcher/window layout settings.
    #[serde(default)]
    pub layout: LayoutConfig,
    /// Theme selection settings.
    #[serde(default)]
    pub theme: ThemeSelectionPreferences,
    /// Dictation / microphone settings.
    #[serde(default)]
    pub dictation: DictationPreferences,
    /// AI chat settings.
    #[serde(default)]
    pub ai: AiPreferences,
    /// Window snapping and related desktop window-management settings.
    #[serde(default)]
    pub window_management: WindowManagementPreferences,
}

/// Agent Chat backend selected for a profile or runtime preference.
///
/// All profiles now resolve through Pi. Legacy `"acp"` values in persisted
/// config deserialize as `Pi` via the serde alias.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AgentChatBackend {
    #[default]
    #[serde(alias = "acp")]
    Pi,
}

/// AI chat preferences loaded from `config.ts`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AiPreferences {
    /// Last-selected model ID (e.g. "claude-sonnet-4-6").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_model_id: Option<String>,

    /// Last-selected global working directory (the footer cwd chip). Restored
    /// on launch so the user keeps the same directory across app restarts.
    /// Stored as an absolute path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,

    /// Last-selected Agent Chat profile id. Takes precedence over the legacy
    /// selected profile name when both are present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_profile_id: Option<String>,

    /// Last-selected Agent Chat backend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_backend: Option<AgentChatBackend>,

    /// Custom path to the Pi Rust agent binary used by Pi-backed Agent Chat profiles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pi_binary: Option<String>,

    /// Pre-configured Agent Chat profiles. Each profile bundles a display name,
    /// optional agent id + model hint, and a custom system prompt.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<AcpProfile>,

    /// Name of the profile currently applied. Matches one of `profiles[].name`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_profile_name: Option<String>,
}

/// A pre-configured Agent Chat profile authored in `config.ts`.
///
/// Profiles bundle a display name, optional agent/model hints, and a custom
/// system prompt that is forwarded through `--append-system-prompt` (or the
/// agent's equivalent) when the profile is active.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AcpProfile {
    /// Stable profile id. Legacy profiles may omit this and use the name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Human-readable profile name used as the menu label and selection key.
    pub name: String,

    /// Bundled icon name shown in Agent Chat profile affordances.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_name: Option<String>,

    /// Backend for this profile. All profiles resolve to Pi.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend: Option<AgentChatBackend>,

    /// Optional Pi Rust binary override for this profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pi_binary: Option<String>,

    /// Optional agent identifier forwarded to the Pi backend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    /// Optional model hint for the agent (e.g. `"claude-sonnet-4-6"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Optional Pi provider id. This is not mapped onto ACP launch args.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Custom system prompt text appended to the agent's system prompt on
    /// `session/new`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,

    /// Extra system prompt text appended to the backend's default prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub append_system_prompt: Option<String>,

    /// Working directory for process-backed Agent Chat backends.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,

    /// Pi tool allow-list. `Some([])` means `--no-tools`; `None` keeps Pi defaults.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,

    /// Structured Pi tool policy. `allow` supersedes the legacy `tools` array.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_policy: Option<AgentChatToolPolicyConfig>,

    /// Runtime filesystem scope forwarded to Pi for read/write-capable tools.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path_policy: Option<AgentChatPathPolicyConfig>,

    /// Message Pi should return when a blocked capability/path is requested.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_action_message: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_extensions: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_skills: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_prompt_templates: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_context_files: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hide_cwd_in_prompt: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extension_policy: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_dir: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_session: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_durability: Option<String>,
}

/// Tool policy for Pi-backed Agent Chat profiles.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgentChatToolPolicyConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<String>>,
}

/// Filesystem policy for Pi-backed Agent Chat profiles.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgentChatPathPolicyConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_read: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_write: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deny: Option<Vec<String>>,
}

/// Window-management preferences loaded from `config.ts`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WindowManagementPreferences {
    /// Persisted drag-snap density/mode. `None` falls back to the app default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snap_mode: Option<crate::window_control::SnapMode>,
}

// ============================================
// COMMAND CONFIG
// ============================================

/// Configuration for a specific command (script, built-in, or app).
///
/// Used to set per-command shortcuts and visibility options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandConfig {
    /// Optional keyboard shortcut to invoke this command directly
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<HotkeyConfig>,
    /// Whether this command should be hidden from the main menu
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    /// Whether this command requires confirmation before execution.
    /// Overrides the default behavior from DEFAULT_CONFIRMATION_COMMANDS.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmation_required: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptTargetConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Executable or shell command to launch for this prompt target.
    pub command: String,
    /// Arguments passed to `command`. Supports `{prompt}` and `{promptFile}` placeholders.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// Working directory override. When omitted, the current prompt cwd is used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Extra environment variables. Supports `{prompt}` and `{promptFile}` placeholders.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}

// Command ID validation and deeplink helpers are now in `crate::config::command_ids`.

// ============================================
// CLAUDE CODE CLI CONFIG
// ============================================

/// Configuration for the Claude Code CLI provider.
///
/// This allows Script Kit to use the local `claude` CLI as an AI provider,
/// speaking JSONL over stdin/stdout for streaming responses.
///
/// # Example
///
/// ```typescript
/// // In ~/.scriptkit/config.ts
/// export default {
///   hotkey: { modifiers: ["meta"], key: "Semicolon" },
///   claudeCode: {
///     enabled: true,
///     permissionMode: "plan",
///     allowedTools: "Read,Edit,Bash(git:*)",
///     addDirs: ["/home/user/projects"]
///   }
/// } satisfies Config;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeConfig {
    /// Enable the Claude Code CLI provider.
    /// When enabled, "Claude Code" models will appear in the AI chat model picker.
    ///
    /// @default false (requires explicit opt-in)
    #[serde(default = "default_claude_code_enabled")]
    pub enabled: bool,

    /// Custom path to the `claude` CLI binary.
    /// If not specified, will look for `claude` in PATH.
    ///
    /// @default undefined (uses "claude" from PATH)
    /// @example "/opt/homebrew/bin/claude"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Permission mode for Claude Code.
    /// - "plan": Safe default - Claude plans but asks before executing tools
    /// - "dontAsk": Agent can execute tools without confirmation (sandbox only!)
    ///
    /// @default "plan"
    #[serde(default = "default_claude_code_permission_mode")]
    pub permission_mode: String,

    /// Comma-separated list of allowed tools.
    /// Restricts which tools Claude Code can use.
    ///
    /// @default undefined (uses Claude Code defaults)
    /// @example "Read,Edit,Bash(git:*)"
    /// @example "Read,Edit,Bash,Write"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<String>,

    /// Additional directories to add to Claude Code's workspace.
    /// Each path is passed as `--add-dir` to the CLI.
    ///
    /// @default [] (empty)
    /// @example ["/home/user/projects", "/tmp/scratch"]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub add_dirs: Vec<String>,
}

fn default_claude_code_enabled() -> bool {
    DEFAULT_CLAUDE_CODE_ENABLED
}

fn default_claude_code_permission_mode() -> String {
    DEFAULT_CLAUDE_CODE_PERMISSION_MODE.to_string()
}

impl Default for ClaudeCodeConfig {
    fn default() -> Self {
        ClaudeCodeConfig {
            enabled: DEFAULT_CLAUDE_CODE_ENABLED,
            path: None,
            permission_mode: DEFAULT_CLAUDE_CODE_PERMISSION_MODE.to_string(),
            allowed_tools: None,
            add_dirs: vec![],
        }
    }
}

// ============================================
// MCP CONFIG
// ============================================

/// External MCP servers that Script Kit scripts and Agent Chat can use.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpConfig {
    /// Master switch for Script Kit-managed MCP integrations.
    #[serde(default = "default_mcp_enabled")]
    pub enabled: bool,

    /// Named MCP server definitions keyed by server id.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub servers: HashMap<String, McpServerConfig>,
}

fn default_mcp_enabled() -> bool {
    true
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            servers: HashMap::new(),
        }
    }
}

impl McpConfig {
    pub fn enabled_servers(&self) -> impl Iterator<Item = (&String, &McpServerConfig)> {
        self.servers
            .iter()
            .filter(|(_, server)| self.enabled && server.is_enabled())
    }
}

/// Supported MCP server transports.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "transport")]
pub enum McpServerConfig {
    #[serde(rename = "stdio")]
    Stdio(McpStdioServerConfig),
    #[serde(rename = "http")]
    Http(McpHttpServerConfig),
}

impl McpServerConfig {
    pub fn is_enabled(&self) -> bool {
        match self {
            McpServerConfig::Stdio(config) => config.enabled,
            McpServerConfig::Http(config) => config.enabled,
        }
    }
}

fn default_mcp_server_enabled() -> bool {
    true
}

/// A local stdio-backed MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpStdioServerConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default = "default_mcp_server_enabled")]
    pub enabled: bool,
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

/// A remote HTTP-backed MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpHttpServerConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default = "default_mcp_server_enabled")]
    pub enabled: bool,
    pub endpoint: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
}

// --- merged from part_02.rs ---
// ============================================
// HOTKEY CONFIG
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HotkeyConfig {
    pub modifiers: Vec<String>,
    pub key: String,
}

impl HotkeyConfig {
    /// Create a default AI hotkey (Cmd+Shift+Space)
    pub fn default_ai_hotkey() -> Self {
        HotkeyConfig {
            modifiers: vec!["meta".to_string(), "shift".to_string()],
            key: "Space".to_string(),
        }
    }

    /// Create a default logs capture hotkey (Cmd+Shift+L)
    pub fn default_logs_hotkey() -> Self {
        HotkeyConfig {
            modifiers: vec!["meta".to_string(), "shift".to_string()],
            key: "KeyL".to_string(),
        }
    }

    /// Create the default dictation toggle hotkey (Cmd+Shift+;).
    pub fn default_dictation_hotkey() -> Self {
        HotkeyConfig {
            modifiers: vec!["meta".to_string(), "shift".to_string()],
            key: "Semicolon".to_string(),
        }
    }

    /// Create the default inline AI focused-text edit hotkey (Cmd+Ctrl+I).
    pub fn default_inline_ai_hotkey() -> Self {
        HotkeyConfig {
            modifiers: vec!["meta".to_string(), "ctrl".to_string()],
            key: "KeyI".to_string(),
        }
    }

    /// Convert to a human-readable display string using macOS symbols (e.g., "⌘⇧K").
    ///
    /// Uses standard macOS modifier symbols in order: ⌃ (Control), ⌥ (Option), ⇧ (Shift), ⌘ (Command)
    pub fn to_display_string(&self) -> String {
        let mut result = String::new();

        // Standard macOS order: Control, Option, Shift, Command
        let has_ctrl = self.modifiers.iter().any(|m| m == "ctrl" || m == "control");
        let has_alt = self.modifiers.iter().any(|m| m == "alt" || m == "option");
        let has_shift = self.modifiers.iter().any(|m| m == "shift");
        let has_cmd = self.modifiers.iter().any(|m| m == "meta" || m == "cmd");

        if has_ctrl {
            result.push('⌃');
        }
        if has_alt {
            result.push('⌥');
        }
        if has_shift {
            result.push('⇧');
        }
        if has_cmd {
            result.push('⌘');
        }

        // Normalize key for display
        let key_display = if self.key.starts_with("Key") {
            // "KeyA" -> "A"
            self.key[3..].to_uppercase()
        } else if self.key.starts_with("Digit") {
            // "Digit0" -> "0"
            self.key[5..].to_string()
        } else {
            // Keep as-is but uppercase first char for consistency
            let mut chars = self.key.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        };
        result.push_str(&key_display);

        result
    }

    /// Convert to canonical shortcut string format (e.g., "cmd+shift+k").
    ///
    /// Maps modifier names from config format to shortcut format:
    /// - "meta" -> "cmd"
    /// - "ctrl" -> "ctrl"
    /// - "alt" -> "alt"
    /// - "shift" -> "shift"
    ///
    /// Keys are normalized:
    /// - "KeyX" -> "x" (strip Key prefix, lowercase)
    /// - "Digit0" -> "0" (strip Digit prefix)
    /// - Other keys kept as-is but lowercased
    pub fn to_shortcut_string(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        // Convert modifiers (maintain consistent order: alt, cmd, ctrl, shift)
        let has_alt = self.modifiers.iter().any(|m| m == "alt" || m == "option");
        let has_cmd = self.modifiers.iter().any(|m| m == "meta" || m == "cmd");
        let has_ctrl = self.modifiers.iter().any(|m| m == "ctrl" || m == "control");
        let has_shift = self.modifiers.iter().any(|m| m == "shift");

        if has_alt {
            parts.push("alt".to_string());
        }
        if has_cmd {
            parts.push("cmd".to_string());
        }
        if has_ctrl {
            parts.push("ctrl".to_string());
        }
        if has_shift {
            parts.push("shift".to_string());
        }

        // Normalize key
        let key = if self.key.starts_with("Key") {
            // "KeyA" -> "a"
            self.key[3..].to_lowercase()
        } else if self.key.starts_with("Digit") {
            // "Digit0" -> "0"
            self.key[5..].to_string()
        } else {
            // Keep as-is but lowercase
            self.key.to_lowercase()
        };
        parts.push(key);

        parts.join("+")
    }
}

fn default_main_hotkey() -> HotkeyConfig {
    HotkeyConfig {
        modifiers: vec!["meta".to_string()],
        key: "Semicolon".to_string(),
    }
}

fn default_ai_hotkey_enabled() -> bool {
    DEFAULT_AI_HOTKEY_ENABLED
}

fn default_logs_hotkey_enabled() -> bool {
    DEFAULT_LOGS_HOTKEY_ENABLED
}

fn default_dictation_hotkey_enabled() -> bool {
    DEFAULT_DICTATION_HOTKEY_ENABLED
}

fn default_inline_ai_hotkey_enabled() -> bool {
    DEFAULT_INLINE_AI_HOTKEY_ENABLED
}

// ============================================
// DESIGNS CONFIG
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct DesignsConfig {
    pub active_id: Option<String>,
    pub cmd1_behavior: Option<Cmd1Behavior>,
    pub overrides: Option<HashMap<String, DesignOverrides>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Cmd1Behavior {
    Picker,
    Cycle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct DesignOverrides {
    pub accent: Option<String>,
    pub density: Option<DesignDensityChoice>,
    pub font_family: Option<FontFamilyChoice>,
    pub font_scale: Option<i8>,
    pub vibrancy: Option<VibrancyChoice>,
    pub chrome_opacity: Option<ChromeOpacityChoice>,
    pub icon_style: Option<IconStyleChoice>,
    pub separator_style: Option<SeparatorStyleChoice>,
    pub row_height_nudge: Option<i8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum DesignDensityChoice {
    Compact,
    Comfortable,
    Spacious,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum FontFamilyChoice {
    System,
    Monospace,
    Serif,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum VibrancyChoice {
    None,
    Light,
    Medium,
    Heavy,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ChromeOpacityChoice {
    Low,
    Med,
    High,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum IconStyleChoice {
    Mono,
    Color,
    Hidden,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum SeparatorStyleChoice {
    None,
    Hairline,
    Rule,
    Grid,
}

// ============================================
// MAIN CONFIG
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_main_hotkey")]
    pub hotkey: HotkeyConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bun_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,
    /// Padding for content areas (terminal, editor, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<ContentPadding>,
    /// Font size for the editor prompt (in pixels)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "editorFontSize"
    )]
    pub editor_font_size: Option<f32>,
    /// Font size for the terminal prompt (in pixels)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "terminalFontSize"
    )]
    pub terminal_font_size: Option<f32>,
    /// UI scale factor (1.0 = 100%)
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "uiScale")]
    pub ui_scale: Option<f32>,
    /// Built-in features configuration (clipboard history, app launcher, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "builtIns")]
    pub built_ins: Option<BuiltInConfig>,
    /// Process resource limits and health monitoring configuration
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "processLimits"
    )]
    pub process_limits: Option<ProcessLimits>,
    /// Maximum text length for clipboard history entries (bytes). 0 = no limit.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "clipboardHistoryMaxTextLength"
    )]
    pub clipboard_history_max_text_length: Option<usize>,
    /// Suggested section configuration (frecency-based ranking)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested: Option<SuggestedConfig>,
    /// Unified root-search sources such as passive local file rows.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "unifiedSearch"
    )]
    pub unified_search: Option<UnifiedSearchConfig>,
    /// Hotkey for opening Notes window (no default; user-configured only)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "notesHotkey"
    )]
    pub notes_hotkey: Option<HotkeyConfig>,
    /// Hotkey for opening Agent Chat window (default: Cmd+Shift+Space)
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "aiHotkey")]
    pub ai_hotkey: Option<HotkeyConfig>,
    /// Whether AI hotkey is enabled (default: true)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "aiHotkeyEnabled"
    )]
    pub ai_hotkey_enabled: Option<bool>,
    /// Hotkey for toggling log capture (default: Cmd+Shift+L)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "logsHotkey"
    )]
    pub logs_hotkey: Option<HotkeyConfig>,
    /// Whether logs hotkey is enabled (default: true)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "logsHotkeyEnabled"
    )]
    pub logs_hotkey_enabled: Option<bool>,
    /// Hotkey for toggling dictation (default: Cmd+Shift+;)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "dictationHotkey"
    )]
    pub dictation_hotkey: Option<HotkeyConfig>,
    /// Whether dictation hotkey registration is enabled (default: true)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "dictationHotkeyEnabled"
    )]
    pub dictation_hotkey_enabled: Option<bool>,
    /// Hotkey for launching inline AI focused-text editing (default: Cmd+Ctrl+I)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "inlineAiHotkey"
    )]
    pub inline_ai_hotkey: Option<HotkeyConfig>,
    /// Whether inline AI focused-text hotkey registration is enabled (default: true)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "inlineAiHotkeyEnabled"
    )]
    pub inline_ai_hotkey_enabled: Option<bool>,
    /// Watcher tuning settings
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub watcher: Option<WatcherConfig>,
    /// Window/layout sizing settings
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<LayoutConfig>,
    /// Theme preset selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<ThemeSelectionPreferences>,
    /// Design picker preferences and per-design token overrides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub designs: Option<DesignsConfig>,
    /// Dictation runtime preferences, including microphone selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dictation: Option<DictationPreferences>,
    /// Agent Chat runtime preferences, including the preferred agent and model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai: Option<AiPreferences>,
    /// Window-management preferences such as snap mode.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "windowManagement"
    )]
    pub window_management: Option<WindowManagementPreferences>,
    /// Per-command configuration overrides (shortcuts, visibility)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commands: Option<HashMap<String, CommandConfig>>,
    /// User-defined prompt handoff targets surfaced as `prompt-target/<id>` actions.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "promptTargets"
    )]
    pub prompt_targets: Option<HashMap<String, PromptTargetConfig>>,
    /// Claude Code CLI provider configuration.
    /// Enable and configure the local `claude` CLI as an AI provider.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "claudeCode"
    )]
    pub claude_code: Option<ClaudeCodeConfig>,
    /// External MCP servers available to scripts and Agent Chat integrations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp: Option<McpConfig>,
    /// Canonical command IDs to hide from the launcher main menu.
    ///
    /// Hidden commands remain resolvable via `triggerBuiltin`, hotkeys, and
    /// other programmatic paths — they are only filtered from visible lists.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "hiddenCommands"
    )]
    pub hidden_commands: Option<Vec<String>>,
}

// --- merged from part_03.rs ---
impl Default for Config {
    fn default() -> Self {
        Config {
            hotkey: default_main_hotkey(),
            bun_path: None,           // Will use system PATH if not specified
            editor: None,             // Will use $EDITOR or fallback to "code"
            padding: None,            // Will use ContentPadding::default() via getter
            editor_font_size: None,   // Will use DEFAULT_EDITOR_FONT_SIZE via getter
            terminal_font_size: None, // Will use DEFAULT_TERMINAL_FONT_SIZE via getter
            ui_scale: None,           // Will use DEFAULT_UI_SCALE via getter
            built_ins: None,          // Will use BuiltInConfig::default() via getter
            process_limits: None,     // Will use ProcessLimits::default() via getter
            clipboard_history_max_text_length: None, // Will use default via getter
            suggested: None,          // Will use SuggestedConfig::default() via getter
            unified_search: None,     // Will use UnifiedSearchConfig::default() via getter
            notes_hotkey: None,       // No default shortcut; must be explicitly configured
            ai_hotkey: None,          // Will use HotkeyConfig::default_ai_hotkey() via getter
            ai_hotkey_enabled: None,  // Defaults to true via getter
            logs_hotkey: None,        // Will use HotkeyConfig::default_logs_hotkey() via getter
            logs_hotkey_enabled: None, // Defaults to true via getter
            dictation_hotkey: None, // Will use HotkeyConfig::default_dictation_hotkey() via getter
            dictation_hotkey_enabled: None, // Defaults to true via getter
            inline_ai_hotkey: None, // Will use HotkeyConfig::default_inline_ai_hotkey() via getter
            inline_ai_hotkey_enabled: None, // Defaults to true via getter
            watcher: None,          // Will use WatcherConfig::default() via getter
            layout: None,           // Will use LayoutConfig::default() via getter
            theme: None,            // Will use ThemeSelectionPreferences::default() via getter
            designs: None,          // No design overrides by default
            dictation: None,        // Will use DictationPreferences::default() via getter
            ai: None,               // Will use AiPreferences::default() via getter
            window_management: None, // Will use WindowManagementPreferences::default() via getter
            commands: None,         // No per-command overrides by default
            prompt_targets: None,   // No custom prompt targets by default
            claude_code: None,      // Will use ClaudeCodeConfig::default() via getter
            mcp: None,              // External MCP servers are opt-in via config.ts
            hidden_commands: None,  // No commands hidden by default
        }
    }
}

fn sanitize_positive_f32(value: Option<f32>, fallback: f32) -> f32 {
    match value {
        Some(value) if value.is_finite() && value > 0.0 => value,
        _ => fallback,
    }
}

fn sanitize_process_limits(mut limits: ProcessLimits) -> ProcessLimits {
    if limits.health_check_interval_ms == 0 {
        limits.health_check_interval_ms = DEFAULT_HEALTH_CHECK_INTERVAL_MS;
    }
    limits
}

impl Config {
    /// Returns the configured editor, falling back to $EDITOR env var or "code" (VS Code)
    /// Used by ActionsDialog "Open in Editor" action
    #[allow(dead_code)] // Will be used by ActionsDialog worker
    pub fn get_editor(&self) -> String {
        self.editor
            .clone()
            .or_else(|| std::env::var("EDITOR").ok())
            .unwrap_or_else(|| "code".to_string())
    }

    /// Returns the content padding, or defaults if not configured
    #[allow(dead_code)] // Will be used by TermPrompt/EditorPrompt workers
    pub fn get_padding(&self) -> ContentPadding {
        self.padding.clone().unwrap_or_default()
    }

    /// Returns the editor font size, or DEFAULT_EDITOR_FONT_SIZE if not configured
    #[allow(dead_code)] // Will be used by EditorPrompt worker
    pub fn get_editor_font_size(&self) -> f32 {
        sanitize_positive_f32(self.editor_font_size, DEFAULT_EDITOR_FONT_SIZE)
    }

    /// Returns the terminal font size, or DEFAULT_TERMINAL_FONT_SIZE if not configured
    #[allow(dead_code)] // Will be used by TermPrompt worker
    pub fn get_terminal_font_size(&self) -> f32 {
        sanitize_positive_f32(self.terminal_font_size, DEFAULT_TERMINAL_FONT_SIZE)
    }

    /// Returns the UI scale factor, or DEFAULT_UI_SCALE if not configured
    #[cfg(test)]
    fn get_ui_scale(&self) -> f32 {
        sanitize_positive_f32(self.ui_scale, DEFAULT_UI_SCALE)
    }

    /// Returns the built-in features configuration, or defaults if not configured
    #[allow(dead_code)] // Will be used by builtins module
    pub fn get_builtins(&self) -> BuiltInConfig {
        self.built_ins.clone().unwrap_or_default()
    }

    /// Returns max clipboard history text length (bytes), or default if not configured
    #[allow(dead_code)] // Used for clipboard history limits
    pub fn get_clipboard_history_max_text_length(&self) -> usize {
        self.clipboard_history_max_text_length
            .unwrap_or(DEFAULT_CLIPBOARD_HISTORY_MAX_TEXT_LENGTH)
    }

    /// Returns the process limits configuration, or defaults if not configured
    pub fn get_process_limits(&self) -> ProcessLimits {
        sanitize_process_limits(self.process_limits.clone().unwrap_or_default())
    }

    /// Returns the suggested section configuration, or defaults if not configured
    pub fn get_suggested(&self) -> SuggestedConfig {
        self.suggested.clone().unwrap_or_default()
    }

    /// Returns unified root-search configuration, or defaults if not configured.
    pub fn get_unified_search(&self) -> UnifiedSearchConfig {
        self.unified_search.clone().unwrap_or_default()
    }

    /// Returns root clipboard-history options, gated by unified search and the built-in feature.
    #[allow(dead_code)]
    pub(crate) fn root_clipboard_history_section_options(
        &self,
    ) -> crate::clipboard_history::RootClipboardHistorySectionOptions {
        let unified = self.get_unified_search();
        let builtins = self.get_builtins();
        crate::clipboard_history::RootClipboardHistorySectionOptions {
            enabled: unified.enabled
                && builtins.clipboard_history
                && unified.clipboard_history.enabled,
            max_results: unified.clipboard_history.max_results.clamp(1, 5),
            min_query_chars: unified.clipboard_history.min_query_chars.clamp(2, 32),
            scan_limit: unified.clipboard_history.scan_limit.clamp(25, 1000),
        }
    }

    /// Returns the notes hotkey configuration, or None if not configured.
    /// No default shortcut is provided - users must explicitly configure one.
    #[allow(dead_code)]
    pub fn get_notes_hotkey(&self) -> Option<HotkeyConfig> {
        self.notes_hotkey.clone()
    }

    /// Returns true if AI hotkey registration is enabled.
    pub fn is_ai_hotkey_enabled(&self) -> bool {
        self.ai_hotkey_enabled
            .unwrap_or_else(default_ai_hotkey_enabled)
    }

    /// Returns true if logs hotkey registration is enabled.
    pub fn is_logs_hotkey_enabled(&self) -> bool {
        self.logs_hotkey_enabled
            .unwrap_or_else(default_logs_hotkey_enabled)
    }

    /// Returns the AI hotkey configuration when enabled.
    /// Falls back to default (Cmd+Shift+Space) when enabled but not configured.
    #[allow(dead_code)]
    pub fn get_ai_hotkey(&self) -> Option<HotkeyConfig> {
        if !self.is_ai_hotkey_enabled() {
            return None;
        }
        Some(
            self.ai_hotkey
                .clone()
                .unwrap_or_else(HotkeyConfig::default_ai_hotkey),
        )
    }

    /// Returns the logs hotkey configuration when enabled.
    /// Falls back to default (Cmd+Shift+L) when enabled but not configured.
    #[allow(dead_code)]
    pub fn get_logs_hotkey(&self) -> Option<HotkeyConfig> {
        if !self.is_logs_hotkey_enabled() {
            return None;
        }
        Some(
            self.logs_hotkey
                .clone()
                .unwrap_or_else(HotkeyConfig::default_logs_hotkey),
        )
    }

    /// Returns true if dictation hotkey registration is enabled.
    pub fn is_dictation_hotkey_enabled(&self) -> bool {
        self.dictation_hotkey_enabled
            .unwrap_or_else(default_dictation_hotkey_enabled)
    }

    /// Returns the dictation hotkey configuration when enabled.
    /// Falls back to default (Cmd+Shift+;) when enabled but not configured.
    #[allow(dead_code)]
    pub fn get_dictation_hotkey(&self) -> Option<HotkeyConfig> {
        if !self.is_dictation_hotkey_enabled() {
            return None;
        }
        Some(
            self.dictation_hotkey
                .clone()
                .unwrap_or_else(HotkeyConfig::default_dictation_hotkey),
        )
    }

    /// Returns true if inline AI focused-text hotkey registration is enabled.
    pub fn is_inline_ai_hotkey_enabled(&self) -> bool {
        self.inline_ai_hotkey_enabled
            .unwrap_or_else(default_inline_ai_hotkey_enabled)
    }

    /// Returns the inline AI focused-text hotkey configuration when enabled.
    /// Falls back to default (Cmd+Ctrl+I) when enabled but not configured.
    #[allow(dead_code)]
    pub fn get_inline_ai_hotkey(&self) -> Option<HotkeyConfig> {
        if !self.is_inline_ai_hotkey_enabled() {
            return None;
        }
        Some(
            self.inline_ai_hotkey
                .clone()
                .unwrap_or_else(HotkeyConfig::default_inline_ai_hotkey),
        )
    }

    /// Returns watcher tuning config, or defaults.
    pub fn get_watcher(&self) -> WatcherConfig {
        self.watcher.clone().unwrap_or_default()
    }

    /// Returns layout sizing config, or defaults.
    #[cfg(test)]
    fn get_layout(&self) -> LayoutConfig {
        self.layout.clone().unwrap_or_default()
    }

    /// Returns theme preset selection config, or defaults.
    pub fn get_theme_selection(&self) -> ThemeSelectionPreferences {
        self.theme.clone().unwrap_or_default()
    }

    /// Returns dictation preferences, or defaults.
    pub fn get_dictation_preferences(&self) -> DictationPreferences {
        self.dictation.clone().unwrap_or_default()
    }

    /// Returns Agent Chat preferences, or defaults.
    pub fn get_ai_preferences(&self) -> AiPreferences {
        self.ai.clone().unwrap_or_default()
    }

    /// Returns window-management preferences, or defaults.
    pub fn get_window_management_preferences(&self) -> WindowManagementPreferences {
        self.window_management.clone().unwrap_or_default()
    }

    /// Returns command configuration for a specific command ID, or None if not configured.
    #[allow(dead_code)]
    pub fn get_command_config(&self, command_id: &str) -> Option<&CommandConfig> {
        self.commands.as_ref().and_then(|cmds| cmds.get(command_id))
    }

    /// Check if a command should be hidden from the main menu.
    ///
    /// Looks at both the per-command `commands.*.hidden` override and the
    /// top-level `hiddenCommands` array. Hidden commands stay resolvable via
    /// programmatic trigger (hotkeys, stdin) — this only filters visible lists.
    pub fn is_command_hidden(&self, command_id: &str) -> bool {
        if self
            .get_command_config(command_id)
            .and_then(|c| c.hidden)
            .unwrap_or(false)
        {
            return true;
        }
        self.hidden_commands
            .as_ref()
            .map(|hidden| hidden.iter().any(|id| id == command_id))
            .unwrap_or(false)
    }

    /// Get the shortcut for a command, if configured.
    #[allow(dead_code)]
    pub fn get_command_shortcut(&self, command_id: &str) -> Option<&HotkeyConfig> {
        self.get_command_config(command_id)
            .and_then(|c| c.shortcut.as_ref())
    }

    /// Check if a command requires confirmation before execution.
    ///
    /// Returns true if:
    /// - Command is in DEFAULT_CONFIRMATION_COMMANDS AND not explicitly disabled in config
    /// - OR command has confirmationRequired: true in config
    #[allow(dead_code)]
    pub fn requires_confirmation(&self, command_id: &str) -> bool {
        // Check if user has explicitly configured this command
        if let Some(cmd_config) = self.get_command_config(command_id) {
            if let Some(requires) = cmd_config.confirmation_required {
                return requires;
            }
        }
        // Fall back to defaults
        DEFAULT_CONFIRMATION_COMMANDS.contains(&command_id)
    }

    /// Returns the Claude Code CLI configuration, or defaults if not configured.
    ///
    /// Use this to check if Claude Code is enabled and get its settings:
    /// ```ignore
    /// let claude_config = config.get_claude_code();
    /// if claude_config.enabled {
    ///     // Register Claude Code provider
    /// }
    /// ```
    pub fn get_claude_code(&self) -> ClaudeCodeConfig {
        self.claude_code.clone().unwrap_or_default()
    }

    /// Returns the Script Kit MCP configuration, or defaults if not configured.
    pub fn get_mcp(&self) -> McpConfig {
        self.mcp.clone().unwrap_or_default()
    }
}

// --- merged from part_04.rs ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotkey_config_to_shortcut_string_basic() {
        let config = HotkeyConfig {
            modifiers: vec!["meta".to_string()],
            key: "KeyK".to_string(),
        };
        assert_eq!(config.to_shortcut_string(), "cmd+k");
    }

    #[test]
    fn hotkey_config_to_shortcut_string_multiple_modifiers() {
        let config = HotkeyConfig {
            modifiers: vec!["meta".to_string(), "shift".to_string()],
            key: "KeyV".to_string(),
        };
        assert_eq!(config.to_shortcut_string(), "cmd+shift+v");
    }

    #[test]
    fn hotkey_config_to_shortcut_string_all_modifiers() {
        let config = HotkeyConfig {
            modifiers: vec![
                "alt".to_string(),
                "meta".to_string(),
                "ctrl".to_string(),
                "shift".to_string(),
            ],
            key: "KeyA".to_string(),
        };
        assert_eq!(config.to_shortcut_string(), "alt+cmd+ctrl+shift+a");
    }

    #[test]
    fn hotkey_config_to_shortcut_string_digit_key() {
        let config = HotkeyConfig {
            modifiers: vec!["meta".to_string()],
            key: "Digit0".to_string(),
        };
        assert_eq!(config.to_shortcut_string(), "cmd+0");
    }

    #[test]
    fn hotkey_config_to_shortcut_string_special_key() {
        let config = HotkeyConfig {
            modifiers: vec!["meta".to_string(), "shift".to_string()],
            key: "Space".to_string(),
        };
        assert_eq!(config.to_shortcut_string(), "cmd+shift+space");
    }

    #[test]
    fn hotkey_config_to_shortcut_string_semicolon() {
        let config = HotkeyConfig {
            modifiers: vec!["meta".to_string()],
            key: "Semicolon".to_string(),
        };
        assert_eq!(config.to_shortcut_string(), "cmd+semicolon");
    }

    #[test]
    fn hotkey_config_to_shortcut_string_ctrl_modifier() {
        let config = HotkeyConfig {
            modifiers: vec!["meta".to_string(), "ctrl".to_string()],
            key: "KeyI".to_string(),
        };
        assert_eq!(config.to_shortcut_string(), "cmd+ctrl+i");
    }

    #[test]
    fn hotkey_config_to_shortcut_string_option_alias() {
        // "option" should be treated as "alt"
        let config = HotkeyConfig {
            modifiers: vec!["option".to_string()],
            key: "KeyN".to_string(),
        };
        assert_eq!(config.to_shortcut_string(), "alt+n");
    }

    #[test]
    fn hotkey_config_to_shortcut_string_cmd_alias() {
        // "cmd" should work as well as "meta"
        let config = HotkeyConfig {
            modifiers: vec!["cmd".to_string()],
            key: "KeyJ".to_string(),
        };
        assert_eq!(config.to_shortcut_string(), "cmd+j");
    }

    // Command ID validation and deeplink tests have moved to config_tests/mod.rs
    // and now use the public crate::config::command_ids module.

    #[test]
    fn test_get_ui_scale_returns_default_when_unset() {
        let config = Config::default();
        assert_eq!(config.get_ui_scale(), DEFAULT_UI_SCALE);
    }

    #[test]
    fn test_get_ui_scale_returns_configured_value_when_positive() {
        let config = Config {
            ui_scale: Some(1.5),
            ..Config::default()
        };
        assert_eq!(config.get_ui_scale(), 1.5);
    }

    #[test]
    fn test_get_ui_scale_returns_default_for_invalid_values() {
        for invalid in [0.0, -0.5, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let config = Config {
                ui_scale: Some(invalid),
                ..Config::default()
            };
            assert_eq!(config.get_ui_scale(), DEFAULT_UI_SCALE);
        }
    }

    #[test]
    fn test_get_layout_returns_default_when_unset() {
        let config = Config::default();
        assert_eq!(
            config.get_layout().standard_height,
            DEFAULT_LAYOUT_STANDARD_HEIGHT
        );
        assert_eq!(config.get_layout().max_height, DEFAULT_LAYOUT_MAX_HEIGHT);
    }

    #[test]
    fn test_get_layout_returns_configured_layout() {
        let config = Config {
            layout: Some(LayoutConfig {
                standard_height: 420.0,
                max_height: 840.0,
            }),
            ..Config::default()
        };

        let layout = config.get_layout();
        assert_eq!(layout.standard_height, 420.0);
        assert_eq!(layout.max_height, 840.0);
    }

    #[test]
    fn test_is_command_hidden_returns_false_when_missing() {
        let config = Config::default();
        assert!(!config.is_command_hidden("script/missing"));
    }

    #[test]
    fn test_is_command_hidden_returns_configured_hidden_value() {
        let mut commands = HashMap::new();
        commands.insert(
            "script/hidden".to_string(),
            CommandConfig {
                shortcut: None,
                hidden: Some(true),
                confirmation_required: None,
            },
        );
        commands.insert(
            "script/visible".to_string(),
            CommandConfig {
                shortcut: None,
                hidden: Some(false),
                confirmation_required: None,
            },
        );

        let config = Config {
            commands: Some(commands),
            ..Config::default()
        };
        assert!(config.is_command_hidden("script/hidden"));
        assert!(!config.is_command_hidden("script/visible"));
    }

    #[test]
    fn test_get_dictation_hotkey_returns_default_when_unset() {
        let config = Config::default();
        assert_eq!(
            config.get_dictation_hotkey(),
            Some(HotkeyConfig::default_dictation_hotkey())
        );
    }

    #[test]
    fn test_get_dictation_hotkey_returns_configured_value_when_enabled() {
        let hotkey = HotkeyConfig {
            modifiers: vec!["meta".to_string(), "shift".to_string()],
            key: "KeyD".to_string(),
        };
        let config = Config {
            dictation_hotkey: Some(hotkey.clone()),
            dictation_hotkey_enabled: Some(true),
            ..Config::default()
        };
        assert_eq!(config.get_dictation_hotkey(), Some(hotkey));
    }

    #[test]
    fn test_get_dictation_hotkey_returns_none_when_disabled() {
        let config = Config {
            dictation_hotkey: Some(HotkeyConfig {
                modifiers: vec!["meta".to_string()],
                key: "KeyD".to_string(),
            }),
            dictation_hotkey_enabled: Some(false),
            ..Config::default()
        };
        assert_eq!(config.get_dictation_hotkey(), None);
    }

    #[test]
    fn test_get_inline_ai_hotkey_returns_default_when_unset() {
        let config = Config::default();
        assert_eq!(
            config.get_inline_ai_hotkey(),
            Some(HotkeyConfig::default_inline_ai_hotkey())
        );
    }

    #[test]
    fn test_get_inline_ai_hotkey_returns_configured_value_when_enabled() {
        let hotkey = HotkeyConfig {
            modifiers: vec!["meta".to_string(), "alt".to_string()],
            key: "KeyI".to_string(),
        };
        let config = Config {
            inline_ai_hotkey: Some(hotkey.clone()),
            inline_ai_hotkey_enabled: Some(true),
            ..Config::default()
        };
        assert_eq!(config.get_inline_ai_hotkey(), Some(hotkey));
    }

    #[test]
    fn test_get_inline_ai_hotkey_returns_none_when_disabled() {
        let config = Config {
            inline_ai_hotkey: Some(HotkeyConfig {
                modifiers: vec!["meta".to_string(), "alt".to_string()],
                key: "KeyI".to_string(),
            }),
            inline_ai_hotkey_enabled: Some(false),
            ..Config::default()
        };
        assert_eq!(config.get_inline_ai_hotkey(), None);
    }

    #[test]
    fn mcp_config_defaults_to_enabled_with_no_servers() {
        let config = McpConfig::default();
        assert!(config.enabled);
        assert!(config.servers.is_empty());
    }

    #[test]
    fn mcp_server_config_round_trips_stdio_variant() {
        let json = r#"{
            "transport": "stdio",
            "command": "npx",
            "args": ["-y", "@modelcontextprotocol/server-memory"]
        }"#;

        let config: McpServerConfig = serde_json::from_str(json).expect("stdio MCP config");
        match config {
            McpServerConfig::Stdio(config) => {
                assert_eq!(config.command, "npx");
                assert_eq!(
                    config.args,
                    vec!["-y", "@modelcontextprotocol/server-memory"]
                );
                assert!(config.enabled);
            }
            McpServerConfig::Http(_) => panic!("expected stdio config"),
        }
    }

    #[test]
    fn config_get_mcp_returns_default_when_missing() {
        let config = Config::default();
        let mcp = config.get_mcp();
        assert!(mcp.enabled);
        assert!(mcp.servers.is_empty());
    }
}
