#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_request_defaults() {
        let req = FocusRequest::main_filter();
        assert_eq!(req.target, FocusTarget::MainFilter);
        assert_eq!(req.cursor, CursorOwner::MainFilter);

        let req = FocusRequest::div_prompt();
        assert_eq!(req.target, FocusTarget::DivPrompt);
        assert_eq!(req.cursor, CursorOwner::None);
    }

    #[test]
    fn test_coordinator_request() {
        let mut coord = FocusCoordinator::new();
        assert!(!coord.has_pending());

        coord.request(FocusRequest::main_filter());
        assert!(coord.has_pending());

        let req = coord.take_pending();
        assert!(req.is_some());
        assert!(!coord.has_pending());
        assert_eq!(coord.cursor_owner(), CursorOwner::MainFilter);
    }

    #[test]
    fn test_overlay_push_pop() {
        let mut coord = FocusCoordinator::with_main_filter_focus();

        // Apply initial focus
        coord.take_pending();
        assert_eq!(coord.cursor_owner(), CursorOwner::MainFilter);
        assert_eq!(coord.overlay_depth(), 0);

        // Push overlay
        coord.push_overlay(FocusRequest::actions_dialog());
        assert_eq!(coord.overlay_depth(), 1);

        // Apply overlay focus
        let req = coord.take_pending().unwrap();
        assert_eq!(req.target, FocusTarget::ActionsDialog);
        assert_eq!(coord.cursor_owner(), CursorOwner::ActionsSearch);

        // Pop overlay
        coord.pop_overlay();
        let req = coord.take_pending().unwrap();
        assert_eq!(req.target, FocusTarget::MainFilter);
        assert_eq!(coord.overlay_depth(), 0);
    }

    #[test]
    fn test_overlay_clear() {
        let mut coord = FocusCoordinator::with_main_filter_focus();
        coord.take_pending();

        // Push multiple overlays
        coord.push_overlay(FocusRequest::actions_dialog());
        coord.take_pending();
        coord.push_overlay(FocusRequest::arg_prompt());
        coord.take_pending();

        assert_eq!(coord.overlay_depth(), 2);

        // Clear all
        coord.clear_overlays();
        assert_eq!(coord.overlay_depth(), 0);

        let req = coord.take_pending().unwrap();
        assert_eq!(req.target, FocusTarget::MainFilter);
    }

    #[test]
    fn test_pop_empty_stack_fallback() {
        let mut coord = FocusCoordinator::new();
        coord.pop_overlay();

        let req = coord.take_pending().unwrap();
        assert_eq!(req.target, FocusTarget::MainFilter);
    }
}
