/// Get the list of enabled built-in entries based on configuration
///
/// # Arguments
/// * `config` - The built-in features configuration
///
/// # Returns
/// A vector of enabled built-in entries that should appear in the main search
///
/// Note: AppLauncher built-in is no longer used since apps now appear directly
/// in the main search results. The config option is retained for future use
/// (e.g., to control whether apps are included in search at all).
pub fn get_builtin_entries(config: &BuiltInConfig) -> Vec<BuiltInEntry> {
    let mut entries = Vec::new();

    include!("part_001_entries/entries_000.rs");
    include!("part_001_entries/entries_001.rs");
    include!("part_001_entries/entries_002.rs");
    include!("part_001_entries/entries_003.rs");

    debug!(count = entries.len(), "Built-in entries loaded");
    entries
}
