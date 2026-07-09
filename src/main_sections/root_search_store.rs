/// Root-launcher file-search state owned as one coherent async cohort.
pub(crate) struct RootSearchStore {
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
    fn default_preserves_root_file_startup_contract() {
        let store = RootSearchStore::default();

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
