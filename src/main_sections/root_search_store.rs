/// Root-launcher file and Brain search state owned as one coherent async cohort.
pub(crate) struct RootSearchStore {
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

#[cfg(test)]
mod root_search_store_tests {
    use super::*;

    #[test]
    fn default_preserves_root_search_startup_contract() {
        let store = RootSearchStore::default();

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
}
