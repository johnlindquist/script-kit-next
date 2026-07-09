/// Root-launcher Windows, file, and Brain search state owned as one coherent async cohort.
pub(crate) struct RootSearchStore {
    /// App-layer enriched rows for root/unified `windows:` search.
    pub(crate) cached_root_windows: Vec<crate::scripts::RootWindowEntry>,
    /// Last provider state for root unified `windows:` search.
    pub(crate) root_windows_provider_status: crate::window_control::RootWindowsProviderStatus,
    /// Generation bumped when root unified search refreshes cached windows.
    pub(crate) root_windows_refresh_generation: u64,
    /// Token used to drop stale async root window refresh results.
    pub(crate) root_windows_refresh_token: u64,
    /// True while an async root window refresh is in flight.
    pub(crate) root_windows_refreshing: bool,
    /// Last successful root window refresh completion.
    pub(crate) root_windows_last_completed_at: Option<std::time::Instant>,
    /// In-memory local recency for windows focused through Script Kit.
    pub(crate) root_window_focus_recency: std::collections::HashMap<String, u64>,
    /// Sequence number for in-memory root window recency.
    pub(crate) root_window_focus_seq: u64,
    /// Async hybrid brain hits keyed by the trimmed root-launcher search text.
    pub(crate) root_brain_semantic_results: Option<(String, Vec<crate::brain::RootBrainSearchHit>)>,
    /// Generation counter used to ignore stale semantic brain batches.
    pub(crate) root_brain_search_generation: u64,
    /// Last requested semantic brain search, used to avoid duplicate work.
    pub(crate) root_brain_search_request: Option<(String, crate::brain::RootBrainSectionOptions)>,
    /// Revision folded into the passive-frame key when semantic results change.
    pub(crate) root_brain_semantic_epoch: u64,
    /// Open brain-inbox items pinned above the empty root-launcher query.
    pub(crate) root_brain_inbox_items: Vec<crate::brain::InboxItem>,
    /// When the root brain-inbox snapshot was last loaded.
    pub(crate) root_brain_inbox_loaded_at: Option<std::time::Instant>,
    /// Revision folded into grouped cache keys when inbox items change.
    pub(crate) root_brain_inbox_epoch: u64,
    /// Latest capped Spotlight results appended to eligible root launcher searches.
    pub(crate) root_file_results: Vec<crate::file_search::FileResult>,
    /// Bounded completed global root file batches, keyed by root search request.
    pub(crate) root_file_result_cache:
        std::collections::VecDeque<(String, Vec<crate::file_search::FileResult>)>,
    /// Source mode currently backing `root_file_results`.
    pub(crate) root_file_search_mode: Option<crate::file_search::RootFileSectionMode>,
    /// Frecency-backed file rows shown on the empty root launcher.
    pub(crate) root_recent_file_results: Vec<crate::file_search::FileResult>,
    /// Frecency revision currently backing `root_recent_file_results`.
    pub(crate) root_recent_file_revision: u64,
    /// Query currently backing `root_file_results`.
    pub(crate) root_file_search_query: String,
    /// Generation counter used to ignore stale root file search batches.
    pub(crate) root_file_search_generation: u64,
    /// Cancel token for in-flight root file search.
    pub(crate) root_file_search_cancel: Option<crate::file_search::CancelToken>,
    /// True while a root file search task is collecting its one stable batch.
    pub(crate) root_file_search_loading: bool,
    /// True while the root file provider is still collecting/cache-warming.
    pub(crate) root_file_provider_loading: bool,
    /// Frozen global root file rows for the current root-search query frame.
    pub(crate) root_file_frame: Option<crate::RootFileFrame>,
    /// Page key for the explicit Files source-chip visible-row budget.
    pub(crate) root_file_source_chip_page_key: Option<String>,
    /// Current visible-row budget for the explicit Files source-chip page.
    pub(crate) root_file_source_chip_visible_limit: usize,
}

impl Default for RootSearchStore {
    fn default() -> Self {
        Self {
            cached_root_windows: Vec::new(),
            root_windows_provider_status:
                crate::window_control::RootWindowsProviderStatus::Unknown,
            root_windows_refresh_generation: 0,
            root_windows_refresh_token: 0,
            root_windows_refreshing: false,
            root_windows_last_completed_at: None,
            root_window_focus_recency: std::collections::HashMap::new(),
            root_window_focus_seq: 0,
            root_brain_semantic_results: None,
            root_brain_search_generation: 0,
            root_brain_search_request: None,
            root_brain_semantic_epoch: 0,
            root_brain_inbox_items: Vec::new(),
            root_brain_inbox_loaded_at: None,
            root_brain_inbox_epoch: 0,
            root_file_results: Vec::new(),
            root_file_result_cache: std::collections::VecDeque::new(),
            root_file_search_mode: None,
            root_recent_file_results: Vec::new(),
            root_recent_file_revision: u64::MAX,
            root_file_search_query: String::new(),
            root_file_search_generation: 0,
            root_file_search_cancel: None,
            root_file_search_loading: false,
            root_file_provider_loading: false,
            root_file_frame: None,
            root_file_source_chip_page_key: None,
            root_file_source_chip_visible_limit:
                crate::file_search::ROOT_FILE_SOURCE_CHIP_INITIAL_VISIBLE_ROWS,
        }
    }
}

impl RootSearchStore {
    pub(crate) fn with_root_windows(
        cached_root_windows: Vec<crate::scripts::RootWindowEntry>,
        root_windows_provider_status: crate::window_control::RootWindowsProviderStatus,
    ) -> Self {
        Self {
            cached_root_windows,
            root_windows_provider_status,
            ..Self::default()
        }
    }

    pub(crate) fn install_root_windows(
        &mut self,
        cached_root_windows: Vec<crate::scripts::RootWindowEntry>,
    ) {
        let count = cached_root_windows.len();
        self.cached_root_windows = cached_root_windows;
        self.bump_root_windows_refresh_generation();
        self.root_windows_provider_status =
            crate::window_control::RootWindowsProviderStatus::Ready { count };
        self.root_windows_last_completed_at = Some(std::time::Instant::now());
    }

    pub(crate) fn rebuild_root_windows(
        &mut self,
        cached_root_windows: Vec<crate::scripts::RootWindowEntry>,
    ) {
        self.cached_root_windows = cached_root_windows;
        self.bump_root_windows_refresh_generation();
    }

    pub(crate) fn root_windows_refresh_needed(&self) -> bool {
        let stale = self
            .root_windows_last_completed_at
            .map(|completed_at| completed_at.elapsed() >= std::time::Duration::from_secs(3))
            .unwrap_or(true);
        !self.root_windows_refreshing && (self.cached_root_windows.is_empty() || stale)
    }

    pub(crate) fn begin_root_windows_refresh(&mut self) -> u64 {
        self.root_windows_refreshing = true;
        self.root_windows_refresh_token = self.root_windows_refresh_token.wrapping_add(1);
        self.root_windows_provider_status =
            crate::window_control::RootWindowsProviderStatus::Refreshing {
                count: self.cached_root_windows.len(),
            };
        self.bump_root_windows_refresh_generation();
        self.root_windows_refresh_token
    }

    pub(crate) fn root_windows_refresh_token_matches(&self, token: u64) -> bool {
        self.root_windows_refresh_token == token
    }

    pub(crate) fn finish_root_windows_refresh_request(&mut self) {
        self.root_windows_refreshing = false;
    }

    pub(crate) fn fail_root_windows_refresh(
        &mut self,
        status: crate::window_control::RootWindowsProviderStatus,
    ) {
        self.root_windows_provider_status = status;
        self.bump_root_windows_refresh_generation();
    }

    pub(crate) fn record_root_window_focus(&mut self, selection_key: String) {
        self.root_window_focus_seq = self.root_window_focus_seq.wrapping_add(1);
        self.root_window_focus_recency
            .insert(selection_key, self.root_window_focus_seq);
    }

    fn bump_root_windows_refresh_generation(&mut self) {
        self.root_windows_refresh_generation = self.root_windows_refresh_generation.wrapping_add(1);
    }
}

#[cfg(test)]
mod root_search_store_tests {
    use super::*;

    #[test]
    fn default_preserves_root_search_startup_contract() {
        let store = RootSearchStore::default();

        assert!(store.cached_root_windows.is_empty());
        assert!(matches!(
            store.root_windows_provider_status,
            crate::window_control::RootWindowsProviderStatus::Unknown
        ));
        assert_eq!(store.root_windows_refresh_generation, 0);
        assert_eq!(store.root_windows_refresh_token, 0);
        assert!(!store.root_windows_refreshing);
        assert!(store.root_windows_last_completed_at.is_none());
        assert!(store.root_window_focus_recency.is_empty());
        assert_eq!(store.root_window_focus_seq, 0);
        assert!(store.root_brain_semantic_results.is_none());
        assert_eq!(store.root_brain_search_generation, 0);
        assert!(store.root_brain_search_request.is_none());
        assert_eq!(store.root_brain_semantic_epoch, 0);
        assert!(store.root_brain_inbox_items.is_empty());
        assert!(store.root_brain_inbox_loaded_at.is_none());
        assert_eq!(store.root_brain_inbox_epoch, 0);
        assert!(store.root_file_results.is_empty());
        assert!(store.root_file_result_cache.is_empty());
        assert_eq!(store.root_file_search_mode, None);
        assert!(store.root_recent_file_results.is_empty());
        assert_eq!(store.root_recent_file_revision, u64::MAX);
        assert!(store.root_file_search_query.is_empty());
        assert_eq!(store.root_file_search_generation, 0);
        assert!(store.root_file_search_cancel.is_none());
        assert!(!store.root_file_search_loading);
        assert!(!store.root_file_provider_loading);
        assert!(store.root_file_frame.is_none());
        assert!(store.root_file_source_chip_page_key.is_none());
        assert_eq!(
            store.root_file_source_chip_visible_limit,
            crate::file_search::ROOT_FILE_SOURCE_CHIP_INITIAL_VISIBLE_ROWS
        );
    }

    #[test]
    fn root_windows_refresh_and_focus_lifecycle_stays_cohesive() {
        let mut store = RootSearchStore::with_root_windows(
            Vec::new(),
            crate::window_control::RootWindowsProviderStatus::Ready { count: 0 },
        );

        let token = store.begin_root_windows_refresh();
        assert!(store.root_windows_refresh_token_matches(token));
        assert!(store.root_windows_refreshing);
        assert_eq!(store.root_windows_refresh_generation, 1);
        assert!(matches!(
            store.root_windows_provider_status,
            crate::window_control::RootWindowsProviderStatus::Refreshing { count: 0 }
        ));

        store.finish_root_windows_refresh_request();
        store.fail_root_windows_refresh(
            crate::window_control::RootWindowsProviderStatus::PermissionRequired,
        );
        assert!(!store.root_windows_refreshing);
        assert_eq!(store.root_windows_refresh_generation, 2);

        store.install_root_windows(Vec::new());
        assert_eq!(store.root_windows_refresh_generation, 3);
        assert!(matches!(
            store.root_windows_provider_status,
            crate::window_control::RootWindowsProviderStatus::Ready { count: 0 }
        ));
        assert!(store.root_windows_last_completed_at.is_some());

        store.rebuild_root_windows(Vec::new());
        assert_eq!(store.root_windows_refresh_generation, 4);

        store.record_root_window_focus("window-key".to_string());
        assert_eq!(store.root_window_focus_seq, 1);
        assert_eq!(store.root_window_focus_recency.get("window-key"), Some(&1));
    }
}
