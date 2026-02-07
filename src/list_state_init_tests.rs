//! Regression tests for list state initialization.
//!
//! These tests ensure that the main list state is properly initialized
//! during ScriptListApp::new() so that scripts are visible on first render.
//!
//! ## Background
//! A bug was introduced when state mutations were moved from render() to event handlers.
//! The fix moved selection validation and list sync to event handlers, but forgot to
//! call these methods during initialization. This caused the first open of the main menu
//! to show "No scripts or snippets found" because main_list_state started with 0 items.
//!
//! The fix is to call sync_list_state() and validate_selection_bounds() at the end
//! of ScriptListApp::new() to ensure the list state is properly initialized.
//!
//! ## Related Commits
//! - "fix: move state mutations from render to event handlers" (introduced bug)
//! - "fix: sync list state on initialization" (fixed bug)

#[cfg(test)]
mod tests {
    use std::fs;

    /// Read startup fragments used by ScriptListApp::new() in execution order.
    fn read_new_startup_sequence() -> String {
        let files = [
            "src/app_impl/startup_new_prelude.rs",
            "src/app_impl/startup_new_state.rs",
            "src/app_impl/startup_new_tab.rs",
            "src/app_impl/startup_new_arrow.rs",
            "src/app_impl/startup_new_actions.rs",
        ];

        let mut content = String::new();
        for file in files {
            content.push_str(
                &fs::read_to_string(file).unwrap_or_else(|_| panic!("Failed to read {}", file)),
            );
            content.push('\n');
        }
        content
    }

    /// Verify that sync_list_state() is called during initialization.
    ///
    /// This is critical: without this call, the first render shows an empty list
    /// because main_list_state is initialized with 0 items but scripts have been loaded.
    #[test]
    fn test_new_calls_sync_list_state() {
        let new_body = read_new_startup_sequence();

        // sync_list_state must be called somewhere in new()
        assert!(
            new_body.contains("sync_list_state"),
            "ScriptListApp::new() must call sync_list_state() to initialize the list state. \
             Without this, the first render shows 'No scripts or snippets found' because \
             main_list_state starts with 0 items.\n\n\
             This was a regression when state mutations were moved from render to event handlers."
        );
    }

    /// Verify that validate_selection_bounds() is called during initialization.
    ///
    /// This ensures the initial selection is valid (not pointing at a section header).
    #[test]
    fn test_new_calls_validate_selection_bounds() {
        let new_body = read_new_startup_sequence();

        // validate_selection_bounds must be called somewhere in new()
        assert!(
            new_body.contains("validate_selection_bounds"),
            "ScriptListApp::new() must call validate_selection_bounds() to ensure \
             the initial selection is valid. Without this, the selection might point \
             at a section header instead of a selectable item."
        );
    }

    /// Verify the order: sync_list_state must come before validate_selection_bounds.
    ///
    /// sync_list_state updates the list item count, and validate_selection_bounds
    /// depends on that count being correct.
    #[test]
    fn test_sync_before_validate() {
        let new_body = read_new_startup_sequence();

        let sync_pos = new_body
            .find("sync_list_state")
            .expect("sync_list_state not found");
        let validate_pos = new_body
            .find("validate_selection_bounds")
            .expect("validate_selection_bounds not found");

        assert!(
            sync_pos < validate_pos,
            "sync_list_state() must be called before validate_selection_bounds() in new(). \
             sync_list_state updates the item count that validate_selection_bounds depends on."
        );
    }

    /// Verify that main_list_state is initialized with 0 items (this is correct).
    ///
    /// The list starts with 0 items because the grouped results need to be computed
    /// before we know the count. sync_list_state() then updates this count.
    #[test]
    fn test_list_state_initial_count_is_zero() {
        let new_body = read_new_startup_sequence();

        // main_list_state should be initialized with 0 items
        assert!(
            new_body.contains("ListState::new(0,"),
            "main_list_state should be initialized with 0 items. \
             The actual count is set by sync_list_state() after grouped results are computed."
        );
    }

    /// Verify that scripts are loaded before sync_list_state is called.
    ///
    /// This ensures there are scripts available when we sync the list state.
    #[test]
    fn test_scripts_loaded_before_sync() {
        let new_body = read_new_startup_sequence();

        let read_scripts_pos = new_body
            .find("scripts::read_scripts()")
            .expect("scripts::read_scripts() not found in new()");
        let sync_pos = new_body
            .find("sync_list_state")
            .expect("sync_list_state not found");

        assert!(
            read_scripts_pos < sync_pos,
            "scripts::read_scripts() must be called before sync_list_state() in new(). \
             The list can't be synced until scripts are loaded."
        );
    }
}
