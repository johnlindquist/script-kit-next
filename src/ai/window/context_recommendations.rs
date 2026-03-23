use crate::ai::context_contract::ContextAttachmentKind;
use crate::ai::message_parts::AiContextPart;
use crate::context_snapshot::AiContextSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextRecommendationPriority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextRecommendation {
    pub kind: ContextAttachmentKind,
    pub reason: String,
    pub priority: ContextRecommendationPriority,
}

impl ContextRecommendation {
    pub fn label(&self) -> &'static str {
        self.kind.spec().label
    }

    pub fn action_id(&self) -> &'static str {
        self.kind.spec().action_id
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContextRecommendationReceipt {
    pub authored_content: String,
    pub recommendations: Vec<ContextRecommendation>,
}

/// Deterministic, pure-function recommendation engine.
///
/// Inspects the user's draft text and the live desktop snapshot to suggest
/// context attachments the user likely forgot to add. Returns stable
/// recommendations ordered by priority then label. Never makes network
/// calls or uses model-based scoring.
pub fn recommend_context_parts(
    authored_content: &str,
    snapshot: &AiContextSnapshot,
    attached: &[AiContextPart],
) -> ContextRecommendationReceipt {
    let text = normalize(authored_content);
    let mut recommendations = Vec::new();

    if text.trim().is_empty() {
        return ContextRecommendationReceipt {
            authored_content: authored_content.to_string(),
            recommendations,
        };
    }

    let has_selection = snapshot
        .selected_text
        .as_ref()
        .map(|value: &String| !value.trim().is_empty())
        .unwrap_or(false);
    let has_browser = snapshot
        .browser
        .as_ref()
        .map(|browser: &crate::context_snapshot::BrowserContext| !browser.url.trim().is_empty())
        .unwrap_or(false);
    let has_window = snapshot.focused_window.is_some();
    let has_current = snapshot.frontmost_app.is_some() || has_browser || has_window;

    let wants_full = contains_any(
        &text,
        &[
            "full context",
            "all context",
            "entire context",
            "everything you can see",
            "everything on screen",
        ],
    );

    let wants_diagnostics = contains_any(
        &text,
        &[
            "diagnostics",
            "debug context",
            "why didn't context work",
            "why did context fail",
            "permissions issue",
            "permission issue",
            "what context do you have",
        ],
    ) || !snapshot.warnings.is_empty();

    let deictic = contains_any(
        &text,
        &[
            "this",
            "here",
            "current",
            "currently",
            "right now",
            "looking at",
            "on screen",
        ],
    );

    let wants_selection = has_selection
        && (contains_any(
            &text,
            &[
                "selection",
                "selected",
                "highlighted",
                "this text",
                "this code",
                "this paragraph",
                "highlight",
            ],
        ) || (deictic
            && contains_any(
                &text,
                &[
                    "rewrite",
                    "summarize",
                    "explain",
                    "translate",
                    "fix",
                    "edit",
                    "improve",
                    "refactor",
                    "review",
                ],
            )));

    let wants_browser = has_browser
        && contains_any(
            &text,
            &[
                "page",
                "tab",
                "browser",
                "website",
                "site",
                "url",
                "link",
                "article",
                "open page",
            ],
        );

    let wants_window = has_window
        && contains_any(
            &text,
            &[
                "window", "screen", "dialog", "modal", "ui", "app", "toolbar", "menu",
            ],
        );

    // --- Full context (explicit request) ---
    if wants_full && !has_attached_kind(attached, ContextAttachmentKind::Full) {
        push_unique(
            &mut recommendations,
            ContextAttachmentKind::Full,
            "You explicitly asked for everything, so attach the full desktop context.",
            ContextRecommendationPriority::High,
        );
    }

    // --- Selection ---
    if wants_selection && !has_attached_kind(attached, ContextAttachmentKind::Selection) {
        push_unique(
            &mut recommendations,
            ContextAttachmentKind::Selection,
            "You referenced selected/highlighted content.",
            ContextRecommendationPriority::High,
        );
    }

    // --- Current Context when both browser and window are relevant ---
    if !wants_selection
        && wants_browser
        && wants_window
        && has_current
        && !has_attached_kind(attached, ContextAttachmentKind::Current)
        && !has_attached_kind(attached, ContextAttachmentKind::Full)
    {
        push_unique(
            &mut recommendations,
            ContextAttachmentKind::Current,
            "You referenced the current page/window, so the minimal ambient context is the best fit.",
            ContextRecommendationPriority::High,
        );
    } else {
        if wants_browser && !has_attached_kind(attached, ContextAttachmentKind::Browser) {
            push_unique(
                &mut recommendations,
                ContextAttachmentKind::Browser,
                "You referenced a page/tab/URL.",
                ContextRecommendationPriority::Medium,
            );
        }

        if wants_window && !has_attached_kind(attached, ContextAttachmentKind::Window) {
            push_unique(
                &mut recommendations,
                ContextAttachmentKind::Window,
                "You referenced the current app/window/UI.",
                ContextRecommendationPriority::Medium,
            );
        }
    }

    // --- Deictic fallback to Current Context ---
    if recommendations.is_empty()
        && deictic
        && has_current
        && !has_attached_kind(attached, ContextAttachmentKind::Current)
        && !has_attached_kind(attached, ContextAttachmentKind::Full)
    {
        push_unique(
            &mut recommendations,
            ContextAttachmentKind::Current,
            "Your draft refers to the current situation without naming a source.",
            ContextRecommendationPriority::Medium,
        );
    }

    // --- Diagnostics ---
    if wants_diagnostics && !has_attached_kind(attached, ContextAttachmentKind::Diagnostics) {
        push_unique(
            &mut recommendations,
            ContextAttachmentKind::Diagnostics,
            "You appear to be debugging context capture itself.",
            ContextRecommendationPriority::Low,
        );
    }

    // Stable sort: priority descending, then label ascending
    recommendations.sort_by(|left, right| {
        priority_rank(right.priority)
            .cmp(&priority_rank(left.priority))
            .then_with(|| left.label().cmp(right.label()))
    });

    tracing::info!(
        target: "ai",
        recommendation_count = recommendations.len(),
        authored_content_len = authored_content.len(),
        has_selection,
        has_browser,
        has_window,
        has_current,
        "ai_context_recommendations_computed"
    );

    ContextRecommendationReceipt {
        authored_content: authored_content.to_string(),
        recommendations,
    }
}

fn has_attached_kind(attached: &[AiContextPart], kind: ContextAttachmentKind) -> bool {
    let part = kind.part();
    attached.iter().any(|existing| existing == &part)
}

fn push_unique(
    recommendations: &mut Vec<ContextRecommendation>,
    kind: ContextAttachmentKind,
    reason: &str,
    priority: ContextRecommendationPriority,
) {
    if recommendations.iter().any(|existing| existing.kind == kind) {
        return;
    }

    recommendations.push(ContextRecommendation {
        kind,
        reason: reason.to_string(),
        priority,
    });
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn normalize(value: &str) -> String {
    value.to_lowercase()
}

fn priority_rank(priority: ContextRecommendationPriority) -> u8 {
    match priority {
        ContextRecommendationPriority::High => 3,
        ContextRecommendationPriority::Medium => 2,
        ContextRecommendationPriority::Low => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_snapshot::{
        AiContextSnapshot, BrowserContext, FocusedWindowContext, FrontmostAppContext,
    };

    fn full_snapshot() -> AiContextSnapshot {
        AiContextSnapshot {
            selected_text: Some("let answer = 42;".to_string()),
            frontmost_app: Some(FrontmostAppContext {
                pid: 123,
                bundle_id: "com.apple.Safari".to_string(),
                name: "Safari".to_string(),
            }),
            browser: Some(BrowserContext {
                url: "https://example.com".to_string(),
            }),
            focused_window: Some(FocusedWindowContext {
                title: "Example".to_string(),
                width: 1440,
                height: 900,
                used_fallback: false,
            }),
            ..Default::default()
        }
    }

    #[test]
    fn recommends_selection_for_selected_text_intent() {
        let receipt = recommend_context_parts(
            "Rewrite this selected text in a friendlier tone",
            &full_snapshot(),
            &[],
        );

        assert_eq!(receipt.recommendations.len(), 1);
        assert_eq!(
            receipt.recommendations[0].kind,
            ContextAttachmentKind::Selection
        );
    }

    #[test]
    fn recommends_current_context_for_deictic_reference() {
        // "this" → deictic, but no specific browser/window/selection keyword
        // → falls back to Current Context via deictic fallback
        let mut snapshot = full_snapshot();
        snapshot.selected_text = None; // Remove selection so it doesn't trigger

        let receipt =
            recommend_context_parts("Can you help me understand this right now?", &snapshot, &[]);

        assert!(
            receipt
                .recommendations
                .iter()
                .any(|item| item.kind == ContextAttachmentKind::Current),
            "deictic language should recommend Current Context"
        );
    }

    #[test]
    fn recommends_current_context_for_combined_page_and_window() {
        // Text that references both page and window/app keywords
        // → combo path fires Current Context
        let mut snapshot = full_snapshot();
        snapshot.selected_text = None; // Remove selection so wants_selection doesn't block combo

        let receipt =
            recommend_context_parts("Summarize the page and app window for me", &snapshot, &[]);

        assert!(
            receipt
                .recommendations
                .iter()
                .any(|item| item.kind == ContextAttachmentKind::Current),
            "combined page + window reference should recommend Current Context"
        );
    }

    #[test]
    fn recommends_diagnostics_when_snapshot_has_warnings() {
        let mut snapshot = full_snapshot();
        snapshot
            .warnings
            .push("browserUrl: permission denied".to_string());

        let receipt = recommend_context_parts("Why didn't context work?", &snapshot, &[]);

        assert!(
            receipt
                .recommendations
                .iter()
                .any(|item| item.kind == ContextAttachmentKind::Diagnostics),
            "should recommend Diagnostics when snapshot has warnings"
        );
    }

    #[test]
    fn skips_already_attached_parts() {
        let receipt = recommend_context_parts(
            "Rewrite this selected text",
            &full_snapshot(),
            &[ContextAttachmentKind::Selection.part()],
        );

        // Selection is already attached, so it should NOT appear in recommendations
        assert!(
            !receipt
                .recommendations
                .iter()
                .any(|item| item.kind == ContextAttachmentKind::Selection),
            "should not recommend Selection when already attached"
        );
    }

    #[test]
    fn empty_draft_returns_no_recommendations() {
        let receipt = recommend_context_parts("", &full_snapshot(), &[]);
        assert!(receipt.recommendations.is_empty());
    }

    #[test]
    fn whitespace_draft_returns_no_recommendations() {
        let receipt = recommend_context_parts("   ", &full_snapshot(), &[]);
        assert!(receipt.recommendations.is_empty());
    }

    #[test]
    fn recommends_full_context_for_explicit_request() {
        let receipt = recommend_context_parts(
            "Give me the full context of what I'm doing",
            &full_snapshot(),
            &[],
        );

        assert!(
            receipt
                .recommendations
                .iter()
                .any(|item| item.kind == ContextAttachmentKind::Full),
            "should recommend Full when user explicitly asks for full context"
        );
    }

    #[test]
    fn recommends_browser_for_page_reference() {
        // Remove focused_window so the combo path doesn't fire
        let mut snapshot = full_snapshot();
        snapshot.focused_window = None;

        let receipt = recommend_context_parts("Summarize this page for me", &snapshot, &[]);

        assert!(
            receipt
                .recommendations
                .iter()
                .any(|item| item.kind == ContextAttachmentKind::Browser),
            "should recommend Browser when user references a page"
        );
    }

    #[test]
    fn deictic_fallback_to_current_context() {
        // No specific signal, just deictic "this" with no other matches
        let mut snapshot = full_snapshot();
        snapshot.selected_text = None; // No selection to trigger Selection
        snapshot.browser = None; // No browser to trigger Browser

        let receipt =
            recommend_context_parts("Can you help me with this right now?", &snapshot, &[]);

        assert!(
            receipt
                .recommendations
                .iter()
                .any(|item| item.kind == ContextAttachmentKind::Current),
            "deictic language without specific signal should fall back to Current Context"
        );
    }

    #[test]
    fn recommendations_are_sorted_by_priority_then_label() {
        let mut snapshot = full_snapshot();
        snapshot.warnings.push("test warning".to_string());

        // "full context" + "diagnostics" → should get Full (High) before Diagnostics (Low)
        let receipt =
            recommend_context_parts("Show me the full context and diagnostics", &snapshot, &[]);

        assert!(receipt.recommendations.len() >= 2);
        // High priority should come first
        assert_eq!(
            receipt.recommendations[0].priority,
            ContextRecommendationPriority::High
        );
    }
}
