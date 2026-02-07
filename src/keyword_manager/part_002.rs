impl Default for KeywordManager {
    fn default() -> Self {
        Self::new()
    }
}
impl Drop for KeywordManager {
    fn drop(&mut self) {
        self.disable();
    }
}
// ============================================================================
// Global Singleton
// ============================================================================

use std::sync::OnceLock;
/// Global singleton for the keyword manager
/// This allows the manager to be accessed from anywhere in the application,
/// including the file watcher callback that needs to update triggers.
static KEYWORD_MANAGER: OnceLock<Mutex<KeywordManager>> = OnceLock::new();
/// Initialize the global keyword manager singleton
///
/// This should be called once at startup. Returns the number of triggers loaded,
/// or an error if initialization fails.
///
/// # Returns
/// - `Ok(Some(count))` if initialization succeeded and keyboard monitoring is enabled
/// - `Ok(None)` if accessibility permissions are not granted (manager not initialized)
/// - `Err(e)` if there was an error during initialization
#[cfg(target_os = "macos")]
pub fn init_keyword_manager() -> Result<Option<usize>> {
    // Check accessibility permissions first
    if !KeywordManager::has_accessibility_permission() {
        info!("Accessibility permissions not granted - text expansion disabled");
        return Ok(None);
    }

    let manager = KEYWORD_MANAGER.get_or_init(|| Mutex::new(KeywordManager::new()));

    let mut guard = manager
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

    // Load scriptlets with keyword triggers
    let count = guard.load_scriptlets()?;
    if count == 0 {
        info!("No keyword triggers found in scriptlets");
        return Ok(Some(0));
    }

    // Enable keyboard monitoring
    guard.enable()?;
    info!(count, "Keyword manager initialized with triggers");

    // List registered triggers for debugging
    for (trigger, name) in guard.list_triggers() {
        debug!(trigger = %trigger, name = %name, "Registered trigger");
    }

    Ok(Some(count))
}
#[cfg(not(target_os = "macos"))]
pub fn init_keyword_manager() -> Result<Option<usize>> {
    // Text expansion is macOS-only for now
    Ok(None)
}
/// Update keyword triggers when a scriptlet file changes
///
/// This is called by the file watcher when a scriptlet file is modified.
/// It updates the keyword matcher with any added/removed/changed triggers.
///
/// # Arguments
/// * `path` - Path to the changed scriptlet file
/// * `new_scriptlets` - The newly parsed scriptlets from the file (empty if file was deleted)
#[cfg(target_os = "macos")]
pub fn update_keyword_triggers_for_file(
    path: &Path,
    new_scriptlets: &[std::sync::Arc<crate::scripts::Scriptlet>],
) -> (usize, usize, usize) {
    info!(
        path = %path.display(),
        scriptlet_count = new_scriptlets.len(),
        "update_keyword_triggers_for_file called"
    );

    let Some(manager) = KEYWORD_MANAGER.get() else {
        warn!("Keyword manager not initialized yet, skipping trigger update");
        return (0, 0, 0);
    };

    let Ok(mut guard) = manager.lock() else {
        warn!("Failed to lock keyword manager");
        return (0, 0, 0);
    };

    // Log existing file_triggers for debugging
    {
        let file_triggers_guard = guard
            .file_triggers
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let existing_keys: Vec<_> = file_triggers_guard.keys().collect();
        info!(
            path = %path.display(),
            existing_paths = ?existing_keys,
            "Checking file_triggers for match"
        );
    }

    // Collect the new triggers from the scriptlets
    let new_triggers: Vec<(String, String, String, String)> = new_scriptlets
        .iter()
        .filter_map(|s| {
            s.keyword.as_ref().map(|kw| {
                info!(
                    keyword = %kw,
                    name = %s.name,
                    content_len = s.code.len(),
                    "Found scriptlet with keyword trigger"
                );
                (kw.clone(), s.name.clone(), s.code.clone(), s.tool.clone())
            })
        })
        .collect();

    info!(
        trigger_count = new_triggers.len(),
        "Calling update_triggers_for_file"
    );

    guard.update_triggers_for_file(path, &new_triggers)
}
#[cfg(not(target_os = "macos"))]
pub fn update_keyword_triggers_for_file(
    _path: &Path,
    _new_scriptlets: &[std::sync::Arc<crate::scripts::Scriptlet>],
) -> (usize, usize, usize) {
    (0, 0, 0)
}
/// Check if the keyword manager is currently enabled
#[allow(dead_code)]
pub fn is_keyword_manager_enabled() -> bool {
    KEYWORD_MANAGER
        .get()
        .and_then(|m| m.lock().ok())
        .map(|g| g.is_enabled())
        .unwrap_or(false)
}
