use super::super::AppView;

/// Resolved Tab AI context payload ready for harness submission.
#[derive(Debug, Clone)]
pub(crate) struct TabAiResolvedContext {
    pub(crate) context: crate::ai::TabAiContextBlob,
    pub(crate) invocation_receipt: crate::ai::TabAiInvocationReceipt,
    pub(crate) suggested_intents: Vec<crate::ai::TabAiSuggestedIntentSpec>,
}

/// Typed launch choices selected before the source surface is snapshotted.
#[derive(Debug, Clone)]
pub(crate) struct TabAiHarnessEntryOptions {
    /// Words for the harness or the chat.
    pub(crate) text: Option<String>,
    /// The words go now; otherwise they wait to be sent.
    pub(crate) submit: bool,
    /// The source surface's focused item goes along.
    pub(crate) stage_focused: bool,
    pub(crate) quick_submit_plan: Option<crate::ai::TabAiQuickSubmitPlan>,
    pub(crate) capture_kind: crate::ai::TabAiCaptureKind,
}

/// Pre-switch snapshot of the UI state captured at the Tab interception
/// boundary, before the view flips to `QuickTerminalView`.
///
/// The deferred capture pipeline uses this to assemble context in the
/// background while the harness terminal is already visible.
#[derive(Debug, Clone)]
pub(crate) struct TabAiLaunchRequest {
    /// The `AppView` that was active when Tab was pressed.
    pub(crate) source_view: AppView,
    /// Words for the harness; they go now when `submit` is set.
    pub(crate) text: Option<String>,
    pub(crate) submit: bool,
    /// Quick-submit plan from the deterministic planner (fallback / dictation).
    pub(crate) quick_submit_plan: Option<crate::ai::TabAiQuickSubmitPlan>,
    /// UI snapshot taken synchronously before the view switch.
    pub(crate) ui_snapshot: crate::ai::TabAiUiSnapshot,
    /// Invocation receipt for logging and downstream consumption.
    pub(crate) invocation_receipt: crate::ai::TabAiInvocationReceipt,
    /// What kind of capture to perform (focused window, full screen, etc.).
    pub(crate) capture_kind: crate::ai::TabAiCaptureKind,
    /// Monotonic generation counter, used to drop stale capture results.
    pub(crate) capture_generation: u64,
}

impl TabAiLaunchRequest {
    pub(crate) fn entry_text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    pub(crate) fn requests_submission(&self) -> bool {
        self.quick_submit_plan
            .as_ref()
            .map(|plan| plan.submit)
            .unwrap_or(self.submit)
    }
}

/// Artifacts produced by the deferred background capture task.
#[derive(Debug, Clone, Default)]
pub(crate) struct TabAiDeferredCaptureArtifacts {
    /// Desktop context snapshot (frontmost app, selected text, browser URL).
    pub(crate) desktop: crate::context_snapshot::AiContextSnapshot,
    /// Absolute path to the focused window screenshot file, if captured.
    pub(crate) screenshot_path: Option<String>,
}

/// Channel receiver for deferred capture results.
pub(crate) type TabAiDeferredCaptureRx =
    async_channel::Receiver<Result<TabAiDeferredCaptureArtifacts, String>>;

/// Maximum visible elements captured per UI snapshot for Tab AI context.
pub(crate) const TAB_AI_VISIBLE_ELEMENT_LIMIT: usize = 24;

/// Maximum visible targets resolved per surface for Tab AI context.
pub(crate) const TAB_AI_VISIBLE_TARGET_LIMIT: usize = 10;

/// Maximum clipboard history entries included in the Tab AI context blob.
pub(crate) const TAB_AI_CLIPBOARD_HISTORY_LIMIT: usize = 8;

/// Maximum character length for hydrated clipboard text entries.
pub(crate) const TAB_AI_CLIPBOARD_TEXT_LIMIT: usize = 1000;
