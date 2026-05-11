//! Run 11 Pass 32 — attacker probe of [[src/menu_syntax/action_effects.rs#apply_safe_effect]]
//! (added Pass 29). Pure-function attacker — no UI surface.
//!
//! Categories: Boundary, Composition, Resurrection. Actions: 22.
//!
//! Probes the Pass-29 safe-effects allowlist, the wildcard `Unsupported`
//! fallthrough, and the per-state dispatch logic for CopyFilterExpression /
//! DefaultTime / EditCommandArgv.

use script_kit_gpui::menu_syntax::actions::{MenuSyntaxActionKind, MenuSyntaxActionState};
use script_kit_gpui::menu_syntax::payload::{
    AdvancedQuery, CaptureAlias, CaptureInvocation, Predicate,
};
use script_kit_gpui::menu_syntax::{apply_safe_effect, ActionEffect};

fn capture(raw: &str, target: &str) -> CaptureInvocation {
    CaptureInvocation {
        target: target.to_string(),
        alias_form: CaptureAlias::CapturePrefix,
        body: String::new(),
        tags: vec![],
        priority: None,
        url: None,
        duration: None,
        kv: vec![],
        date_phrases: vec![],
        raw: raw.to_string(),
    }
}

fn refine(raw: &str) -> AdvancedQuery {
    AdvancedQuery {
        free_text: String::new(),
        predicates: Vec::<Predicate>::new(),
        source_filters: Default::default(),
        raw: raw.to_string(),
    }
}

// ============================================================================
// BOUNDARY (8 actions) — empty / single-char / unicode / extreme inputs
// ============================================================================

#[test]
fn boundary_01_empty_raw_capture_copy_returns_empty_clipboard() {
    let inv = capture("", "cal");
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &inv,
        schema: None,
    };
    assert_eq!(
        apply_safe_effect(&state, &MenuSyntaxActionKind::CopyFilterExpression),
        ActionEffect::WriteClipboard {
            content: String::new()
        }
    );
}

#[test]
fn boundary_02_default_time_with_empty_raw_still_appends() {
    // Per Pass 29 contract, DefaultTime trims trailing whitespace then appends.
    // Empty raw → trim_end is "" → format produces ` start:"today 9am"` with a
    // leading space that's actually in the output. Document current behavior.
    let inv = capture("", "cal");
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &inv,
        schema: None,
    };
    let kind = MenuSyntaxActionKind::DefaultTime {
        phrase: "today 9am".to_string(),
    };
    match apply_safe_effect(&state, &kind) {
        ActionEffect::SetFilterText { new_text } => {
            // Empty trimmed + " start:..." → " start:..." (leading space).
            assert_eq!(new_text, " start:\"today 9am\"");
        }
        other => panic!("expected SetFilterText, got {other:?}"),
    }
}

#[test]
fn boundary_03_default_time_with_empty_phrase_serializes_empty_quotes() {
    let inv = capture(";cal task", "cal");
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &inv,
        schema: None,
    };
    let kind = MenuSyntaxActionKind::DefaultTime {
        phrase: String::new(),
    };
    match apply_safe_effect(&state, &kind) {
        ActionEffect::SetFilterText { new_text } => {
            assert_eq!(new_text, ";cal task start:\"\"");
        }
        other => panic!("expected SetFilterText, got {other:?}"),
    }
}

#[test]
fn boundary_04_default_time_with_quote_in_phrase_is_backslash_escaped_PINNED() {
    // Run 11 Pass #41 (Fix): the original Pass-32 [?] is closed. The
    // DefaultTime arm now routes the phrase through `quote_for_filter_value`
    // which backslash-escapes `"` and `\`. A phrase containing `"` produces
    // a properly-escaped, balanced output instead of unbalanced quotes.
    let inv = capture(";cal task", "cal");
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &inv,
        schema: None,
    };
    let kind = MenuSyntaxActionKind::DefaultTime {
        phrase: r#"today "9am""#.to_string(),
    };
    match apply_safe_effect(&state, &kind) {
        ActionEffect::SetFilterText { new_text } => {
            assert_eq!(new_text, r#";cal task start:"today \"9am\"""#);
            // Falsifier guard: count UNESCAPED `"` chars — must be exactly 2.
            let mut unescaped = 0;
            let mut prev_backslash = false;
            for c in new_text.chars() {
                if c == '"' && !prev_backslash {
                    unescaped += 1;
                }
                prev_backslash = c == '\\' && !prev_backslash;
            }
            assert_eq!(unescaped, 2, "balanced wrappers expected; got {new_text:?}");
        }
        other => panic!("expected SetFilterText, got {other:?}"),
    }
}

#[test]
fn boundary_05_edit_command_argv_with_empty_head_yields_just_bang_space() {
    let argv: Vec<String> = vec![];
    let state = MenuSyntaxActionState::CommandComposer {
        head: "",
        argv: &argv,
    };
    assert_eq!(
        apply_safe_effect(&state, &MenuSyntaxActionKind::EditCommandArgv),
        ActionEffect::SetFilterText {
            new_text: "! ".to_string(),
        }
    );
}

#[test]
fn boundary_06_copy_filter_command_with_empty_argv_yields_bang_head() {
    let argv: Vec<String> = vec![];
    let state = MenuSyntaxActionState::CommandComposer {
        head: "deploy",
        argv: &argv,
    };
    assert_eq!(
        apply_safe_effect(&state, &MenuSyntaxActionKind::CopyFilterExpression),
        ActionEffect::WriteClipboard {
            content: ">deploy".to_string(),
        }
    );
}

#[test]
fn boundary_07_copy_filter_command_with_empty_argv_entry_keeps_double_space() {
    // An empty string in argv produces back-to-back spaces. Current behavior;
    // documenting since the formatter is intentionally simple.
    let argv: Vec<String> = vec!["".into(), "real".into()];
    let state = MenuSyntaxActionState::CommandComposer {
        head: "deploy",
        argv: &argv,
    };
    assert_eq!(
        apply_safe_effect(&state, &MenuSyntaxActionKind::CopyFilterExpression),
        ActionEffect::WriteClipboard {
            content: ">deploy  real".to_string(),
        }
    );
}

#[test]
fn boundary_08_unicode_in_payload_raw_passes_through() {
    let inv = capture(";cal 設計レビュー \u{1F4D8}", "cal");
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &inv,
        schema: None,
    };
    assert_eq!(
        apply_safe_effect(&state, &MenuSyntaxActionKind::CopyFilterExpression),
        ActionEffect::WriteClipboard {
            content: ";cal 設計レビュー \u{1F4D8}".to_string(),
        }
    );
}

// ============================================================================
// COMPOSITION (8 actions) — state × kind cross-products, fallthrough cases
// ============================================================================

#[test]
fn composition_09_default_time_in_command_state_unsupported() {
    // DefaultTime is capture-only per Pass 29. Other states fall through.
    let argv: Vec<String> = vec!["--prod".into()];
    let state = MenuSyntaxActionState::CommandComposer {
        head: "deploy",
        argv: &argv,
    };
    let kind = MenuSyntaxActionKind::DefaultTime {
        phrase: "today 9am".to_string(),
    };
    assert_eq!(apply_safe_effect(&state, &kind), ActionEffect::Unsupported);
}

#[test]
fn composition_10_edit_command_argv_in_refine_state_unsupported() {
    // EditCommandArgv is command-only per Pass 29.
    let q = refine(":foo");
    let state = MenuSyntaxActionState::RefineQuery { query: &q };
    assert_eq!(
        apply_safe_effect(&state, &MenuSyntaxActionKind::EditCommandArgv),
        ActionEffect::Unsupported
    );
}

#[test]
fn composition_11_copy_filter_in_capture_uses_payload_raw_not_target() {
    // The clipboard content comes from `payload.raw`, NOT from `target`.
    // A divergence between the two (e.g. parser bug) MUST surface payload.raw.
    let inv = capture("+CAL Design review", "cal"); // raw uppercase, target lowercase
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &inv,
        schema: None,
    };
    assert_eq!(
        apply_safe_effect(&state, &MenuSyntaxActionKind::CopyFilterExpression),
        ActionEffect::WriteClipboard {
            content: "+CAL Design review".to_string(),
        }
    );
}

#[test]
fn composition_12_copy_filter_refine_uses_query_raw_not_free_text() {
    // The clipboard content comes from `query.raw`, ignoring the parsed
    // `free_text` / `predicates` decomposition.
    let mut q = refine(":kit#tag1");
    q.free_text = "kit".to_string(); // would diverge if mapper used free_text
    let state = MenuSyntaxActionState::RefineQuery { query: &q };
    assert_eq!(
        apply_safe_effect(&state, &MenuSyntaxActionKind::CopyFilterExpression),
        ActionEffect::WriteClipboard {
            content: ":kit#tag1".to_string(),
        }
    );
}

#[test]
fn composition_13_cancel_dominates_every_state_kind_combo() {
    // Cancel is the only kind that matches before per-state dispatch. This
    // is the cross-product invariant: 3 states × Cancel → all Cancel.
    let inv = capture(";cal", "cal");
    let q = refine(":a");
    let argv: Vec<String> = vec![];
    let states: Vec<MenuSyntaxActionState<'_>> = vec![
        MenuSyntaxActionState::CaptureComposer {
            target: "cal",
            payload: &inv,
            schema: None,
        },
        MenuSyntaxActionState::RefineQuery { query: &q },
        MenuSyntaxActionState::CommandComposer {
            head: "deploy",
            argv: &argv,
        },
    ];
    for state in &states {
        assert_eq!(
            apply_safe_effect(state, &MenuSyntaxActionKind::Cancel),
            ActionEffect::Cancel,
            "Cancel must dominate state {state:?}"
        );
    }
}

#[test]
fn composition_14_open_captures_browser_unsupported_in_every_state() {
    // OpenCapturesBrowser is in the unsafe-kinds backlog. Verify it's
    // Unsupported in EVERY state, not just capture.
    let inv = capture(";cal", "cal");
    let q = refine(":a");
    let argv: Vec<String> = vec![];
    let states: Vec<MenuSyntaxActionState<'_>> = vec![
        MenuSyntaxActionState::CaptureComposer {
            target: "cal",
            payload: &inv,
            schema: None,
        },
        MenuSyntaxActionState::RefineQuery { query: &q },
        MenuSyntaxActionState::CommandComposer {
            head: "deploy",
            argv: &argv,
        },
    ];
    let kind = MenuSyntaxActionKind::OpenCapturesBrowser {
        target: "todo".to_string(),
    };
    for state in &states {
        assert_eq!(
            apply_safe_effect(state, &kind),
            ActionEffect::Unsupported,
            "OpenCapturesBrowser must be Unsupported in {state:?}"
        );
    }
}

#[test]
fn composition_15_default_time_in_refine_state_unsupported() {
    let q = refine(":foo");
    let state = MenuSyntaxActionState::RefineQuery { query: &q };
    let kind = MenuSyntaxActionKind::DefaultTime {
        phrase: "today 9am".to_string(),
    };
    assert_eq!(apply_safe_effect(&state, &kind), ActionEffect::Unsupported);
}

#[test]
fn composition_16_edit_command_argv_in_capture_state_unsupported() {
    let inv = capture(";cal", "cal");
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &inv,
        schema: None,
    };
    assert_eq!(
        apply_safe_effect(&state, &MenuSyntaxActionKind::EditCommandArgv),
        ActionEffect::Unsupported
    );
}

// ============================================================================
// RESURRECTION (6 actions) — idempotence, cloning, repeated calls
// ============================================================================

#[test]
fn resurrection_17_repeated_default_time_calls_yield_same_effect() {
    let inv = capture(";cal Standup", "cal");
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &inv,
        schema: None,
    };
    let kind = MenuSyntaxActionKind::DefaultTime {
        phrase: "today 9am".to_string(),
    };
    let e1 = apply_safe_effect(&state, &kind);
    let e2 = apply_safe_effect(&state, &kind);
    let e3 = apply_safe_effect(&state, &kind);
    assert_eq!(e1, e2);
    assert_eq!(e2, e3);
    assert_eq!(
        e1,
        ActionEffect::SetFilterText {
            new_text: ";cal Standup start:\"today 9am\"".to_string(),
        }
    );
}

#[test]
fn resurrection_18_clone_kind_yields_equal_effect() {
    let inv = capture(";cal", "cal");
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &inv,
        schema: None,
    };
    let kind = MenuSyntaxActionKind::DefaultTime {
        phrase: "noon".to_string(),
    };
    let cloned = kind.clone();
    assert_eq!(
        apply_safe_effect(&state, &kind),
        apply_safe_effect(&state, &cloned)
    );
}

#[test]
fn resurrection_19_clone_effect_equality_preserved() {
    let argv: Vec<String> = vec!["--foo".into()];
    let state = MenuSyntaxActionState::CommandComposer {
        head: "deploy",
        argv: &argv,
    };
    let e = apply_safe_effect(&state, &MenuSyntaxActionKind::Cancel);
    let cloned = e.clone();
    assert_eq!(e, cloned);
    assert_eq!(e, ActionEffect::Cancel);
}

#[test]
fn resurrection_20_payload_mutation_flows_into_repeat_call() {
    let mut inv = capture(";cal A", "cal");
    let kind = MenuSyntaxActionKind::CopyFilterExpression;
    {
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "cal",
            payload: &inv,
            schema: None,
        };
        assert_eq!(
            apply_safe_effect(&state, &kind),
            ActionEffect::WriteClipboard {
                content: ";cal A".to_string(),
            }
        );
    }
    inv.raw = ";cal B".to_string();
    {
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "cal",
            payload: &inv,
            schema: None,
        };
        assert_eq!(
            apply_safe_effect(&state, &kind),
            ActionEffect::WriteClipboard {
                content: ";cal B".to_string(),
            }
        );
    }
}

#[test]
fn resurrection_21_unsupported_is_idempotent_across_kinds() {
    // Multiple unsafe kinds in sequence each return Unsupported; the
    // function carries no state.
    let inv = capture(";cal", "cal");
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &inv,
        schema: None,
    };
    let kinds = [
        MenuSyntaxActionKind::SaveAndCopyId,
        MenuSyntaxActionKind::EditPayloadJson,
        MenuSyntaxActionKind::ChangeHandler,
        MenuSyntaxActionKind::SaveAndCopyId, // repeat
    ];
    for kind in &kinds {
        assert_eq!(apply_safe_effect(&state, kind), ActionEffect::Unsupported);
    }
}

#[test]
fn resurrection_22_state_tuple_dispatch_is_pure() {
    // The same (state, kind) tuple in two distinct match arms must produce
    // distinct branches. This exercises the wildcard `_ => Unsupported` arm
    // by alternating an Implemented and Unsupported kind in the same state.
    let inv = capture(";cal task", "cal");
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &inv,
        schema: None,
    };
    assert_eq!(
        apply_safe_effect(&state, &MenuSyntaxActionKind::CopyFilterExpression),
        ActionEffect::WriteClipboard {
            content: ";cal task".to_string(),
        }
    );
    assert_eq!(
        apply_safe_effect(&state, &MenuSyntaxActionKind::SaveAndCopyId),
        ActionEffect::Unsupported
    );
    assert_eq!(
        apply_safe_effect(&state, &MenuSyntaxActionKind::CopyFilterExpression),
        ActionEffect::WriteClipboard {
            content: ";cal task".to_string(),
        }
    );
}
