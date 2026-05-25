use script_kit_gpui::platform::accessibility::double_modifier_trigger::{
    DoubleCommandOutcome, DoubleCommandState, ModifierEvent,
};
use script_kit_gpui::platform::accessibility::focused_text::classify_content_kind;
use script_kit_gpui::platform::accessibility::focused_text::FocusedTextContentKind;
use script_kit_gpui::platform::accessibility::geometry::{
    preferred_anchor_geometry, DisplayBounds, FocusedFieldGeometry, RectPx,
};
use script_kit_gpui::platform::accessibility::metrics::TextMetrics;
use script_kit_gpui::platform::accessibility::mutation::{
    plan_append_mutation, validate_mutation_session, AppendMutationPlan,
    FocusedTextMutationSession, TextMutationOptions,
};
use script_kit_gpui::platform::accessibility::{FocusedTextError, FocusedTextSessionId};

#[test]
fn utf16_metrics_count_emoji_without_confusing_chars_and_units() {
    let metrics = TextMetrics::from_text("A🧪\nB");

    assert_eq!(metrics.bytes, "A🧪\nB".len());
    assert_eq!(metrics.chars, 4);
    assert_eq!(metrics.utf16_units, 5);
    assert_eq!(metrics.lines, 2);
    assert_eq!(metrics.estimated_tokens, 1);
}

#[test]
fn anchor_fallback_order_prefers_caret_selection_field_window_display() {
    let display = DisplayBounds {
        visible: RectPx {
            x: 0.0,
            y: 0.0,
            width: 1000.0,
            height: 800.0,
        },
    };
    let window = RectPx {
        x: 10.0,
        y: 20.0,
        width: 300.0,
        height: 200.0,
    };
    let field = RectPx {
        x: 30.0,
        y: 40.0,
        width: 250.0,
        height: 120.0,
    };
    let selection = RectPx {
        x: 50.0,
        y: 60.0,
        width: 80.0,
        height: 20.0,
    };
    let caret = RectPx {
        x: 70.0,
        y: 80.0,
        width: 2.0,
        height: 18.0,
    };

    assert_eq!(
        preferred_anchor_geometry(&FocusedFieldGeometry {
            caret_bounds: Some(caret),
            selection_bounds: Some(selection),
            field_bounds: Some(field),
            window_bounds: Some(window),
            display_bounds: display,
        }),
        caret
    );
    assert_eq!(
        preferred_anchor_geometry(&FocusedFieldGeometry {
            caret_bounds: None,
            selection_bounds: Some(selection),
            field_bounds: Some(field),
            window_bounds: Some(window),
            display_bounds: display,
        }),
        selection
    );
    assert_eq!(
        preferred_anchor_geometry(&FocusedFieldGeometry {
            caret_bounds: None,
            selection_bounds: None,
            field_bounds: Some(field),
            window_bounds: Some(window),
            display_bounds: display,
        }),
        field
    );
    assert_eq!(
        preferred_anchor_geometry(&FocusedFieldGeometry {
            caret_bounds: None,
            selection_bounds: None,
            field_bounds: None,
            window_bounds: Some(window),
            display_bounds: display,
        }),
        window
    );
    assert_eq!(
        preferred_anchor_geometry(&FocusedFieldGeometry {
            caret_bounds: None,
            selection_bounds: None,
            field_bounds: None,
            window_bounds: None,
            display_bounds: display,
        }),
        display.visible
    );
}

#[test]
fn stale_mutation_sessions_reject_unless_explicitly_allowed() {
    let session = FocusedTextMutationSession {
        session_id: FocusedTextSessionId::new_for_tests("session-a"),
        captured_at_ms: 1_000,
        current_text: Some("draft".to_string()),
        ttl_ms: 500,
    };

    assert_eq!(
        validate_mutation_session(&session, TextMutationOptions::default(), 1_501),
        Err(FocusedTextError::StaleSession)
    );
    assert_eq!(
        validate_mutation_session(&session, TextMutationOptions { allow_stale: true }, 1_501),
        Ok(())
    );
}

#[test]
fn append_plan_uses_current_readable_value_when_direct_set_is_available() {
    let plan = plan_append_mutation(Some("Hello"), " world", true, false);

    assert_eq!(
        plan,
        AppendMutationPlan::DirectSet {
            text: "Hello world".to_string(),
            metrics: TextMetrics::from_text("Hello world"),
        }
    );
}

#[test]
fn append_plan_pastes_only_output_when_caret_can_move_to_end() {
    let plan = plan_append_mutation(Some("Hello"), " world", false, true);

    assert_eq!(
        plan,
        AppendMutationPlan::PasteOutputAtEnd {
            output: " world".to_string(),
            output_metrics: TextMetrics::from_text(" world"),
        }
    );
}

#[test]
fn double_command_triggers_but_combined_shortcut_resets_state() {
    let mut state = DoubleCommandState::default();
    assert_eq!(
        state.observe(ModifierEvent::CommandUp { at_ms: 1_000 }),
        DoubleCommandOutcome::Armed
    );
    assert_eq!(
        state.observe(ModifierEvent::CombinedShortcut),
        DoubleCommandOutcome::Idle
    );
    assert_eq!(
        state.observe(ModifierEvent::CommandUp { at_ms: 1_100 }),
        DoubleCommandOutcome::Armed
    );
    assert_eq!(
        state.observe(ModifierEvent::CommandUp { at_ms: 1_220 }),
        DoubleCommandOutcome::Trigger
    );
}

#[test]
fn focused_text_content_kind_rejects_secure_fields() {
    assert_eq!(
        classify_content_kind(Some("AXTextField"), Some("AXSecureTextField")),
        FocusedTextContentKind::Secure
    );
    assert_eq!(
        classify_content_kind(Some("AXTextArea"), None),
        FocusedTextContentKind::PlainText
    );
    assert_eq!(
        classify_content_kind(Some("AXStaticText"), None),
        FocusedTextContentKind::RichText
    );
    assert_eq!(
        classify_content_kind(Some("AXButton"), None),
        FocusedTextContentKind::Unsupported
    );
}
