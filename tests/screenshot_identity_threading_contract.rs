//! Source-level contract test for the `screenshot-identity-threading` user story.
//!
//! The story wants three live receipts: (a) a `screenshot` builtin trigger,
//! (b) `getState.promptType="imagePrompt"` after capture, and (c)
//! `imageIdentity="screenshot:<timestamp>"` with a matching identity when
//! the image is embedded into Agent Chat. Three structural gaps block the literal
//! assertions: (1) there is no `BuiltInFeature::Screenshot` — screenshots
//! are captured through `AiCommand::SendScreenToAi` (from the AI commands
//! submenu) and through the Tab AI deferred-capture pipeline, not via
//! `triggerBuiltin screenshot`; (2) `prompt_type` never becomes
//! `"imagePrompt"` — screenshots do not change the prompt type, they are
//! attached to the AI input; (3) `imageIdentity` is not a `State` field.
//!
//! However, the **behavioral** invariant the story cares about — identity
//! threading from capture through to the Agent Chat context — is real and
//! implemented, just via a different mechanism than the story text implies.
//! Identity is encoded in the screenshot filename (`tab-ai-screenshot-
//! <UTC-ISO-millis>Z-<pid>-<sequence>.png`) produced by
//! [`src/ai/harness/screenshot_files.rs`], carried through
//! `TabAiScreenshotFile.path` and then `TabAiContextBlob.screenshot_path`,
//! and finally emitted into the Agent Chat context as a `screenshot path: <path>`
//! line inside a single `ContentBlock::Text`.
//!
//! This test pins the identity-threading chain at source level so any
//! regression that breaks identity (non-unique filenames, dropped path,
//! silent image-block insertion, missing sequence counter) surfaces in CI.
//! It does not try to verify the missing state-field machinery — that is
//! carried forward as a separate tooling story.
//!
//! Invariants pinned:
//!
//! 1. `build_tab_ai_screenshot_filename(now, pid, sequence)` formats the
//!    filename with `%Y%m%dT%H%M%S%.3fZ` (millisecond precision), PID, and
//!    sequence — three independent axes of identity.
//!
//! 2. `TAB_AI_SCREENSHOT_SEQUENCE` is an `AtomicU64` incremented via
//!    `fetch_add(1, Ordering::Relaxed)` — guarantees per-process
//!    monotonicity so two captures in the same millisecond cannot alias.
//!
//! 3. Both capture paths (focused-window and full-screen) build their
//!    filenames through `build_tab_ai_screenshot_filename` with
//!    `chrono::Utc::now()`, `std::process::id()`, and
//!    `TAB_AI_SCREENSHOT_SEQUENCE.fetch_add(1, Ordering::Relaxed)` — they
//!    share the same identity generator, not two parallel ones.
//!
//! 4. `TabAiScreenshotFile` carries `path`, `width`, `height`, `title`,
//!    `used_fallback` — the tuple a consumer needs to thread identity.
//!
//! 5. `TabAiContextBlob.screenshot_path: Option<String>` is the field the
//!    capture wires into; `build_tab_ai_harness_context_block` emits a
//!    literal `screenshot path: <path>` line when that field is `Some`,
//!    so the captured path survives context construction.
//!
//! 6. `build_tab_ai_agent_chat_context_blocks` produces a SINGLE `ContentBlock`
//!    wrapping the harness text output — no image content block is
//!    silently inserted, which would break the story's "matching identity"
//!    clause (an image block has its own identity scheme separate from the
//!    text context path).

const SCREENSHOT_FILES_SOURCE: &str = include_str!("../src/ai/harness/screenshot_files.rs");
const HARNESS_CONTEXT_SOURCE: &str = include_str!("../src/ai/harness/mod.rs");
const AGENT_CHAT_CONTEXT_SOURCE: &str = include_str!("../src/ai/agent_chat/ui/context.rs");
const TAB_CONTEXT_SOURCE: &str = include_str!("../src/ai/tab_context.rs");

#[test]
fn screenshot_filename_format_encodes_three_independent_identity_axes() {
    assert!(
        SCREENSHOT_FILES_SOURCE.contains("fn build_tab_ai_screenshot_filename("),
        "build_tab_ai_screenshot_filename must remain the single filename \
         generator — if it moves or splits, identity threading forks"
    );
    assert!(
        SCREENSHOT_FILES_SOURCE.contains("now.format(\"%Y%m%dT%H%M%S%.3fZ\")"),
        "filename timestamp must retain millisecond precision \
         (%Y%m%dT%H%M%S%.3fZ) — dropping sub-second precision lets two \
         rapid captures alias on the same identity"
    );
    assert!(
        SCREENSHOT_FILES_SOURCE.contains("TAB_AI_SCREENSHOT_PREFIX")
            && SCREENSHOT_FILES_SOURCE
                .contains("const TAB_AI_SCREENSHOT_PREFIX: &str = \"tab-ai-screenshot-\";"),
        "filename prefix must remain the stable `tab-ai-screenshot-` so \
         cleanup and identity matching still work — renaming it silently \
         orphans any older PNGs left over after a restart"
    );
}

#[test]
fn sequence_counter_is_atomic_and_monotonic() {
    assert!(
        SCREENSHOT_FILES_SOURCE
            .contains("static TAB_AI_SCREENSHOT_SEQUENCE: AtomicU64 = AtomicU64::new(0);"),
        "TAB_AI_SCREENSHOT_SEQUENCE must remain an AtomicU64 starting at \
         0 — a non-atomic counter can race on rapid captures and produce \
         duplicate identities"
    );
    let fetch_add_count = SCREENSHOT_FILES_SOURCE
        .matches("TAB_AI_SCREENSHOT_SEQUENCE.fetch_add(1, Ordering::Relaxed)")
        .count();
    assert!(
        fetch_add_count >= 2,
        "TAB_AI_SCREENSHOT_SEQUENCE.fetch_add(1, Ordering::Relaxed) must \
         be called from BOTH capture helpers (focused-window and \
         full-screen). Found {fetch_add_count} calls, expected >=2. If \
         one helper stops bumping the counter, its captures alias on \
         each other within the same millisecond."
    );
}

#[test]
fn both_capture_helpers_route_through_the_same_filename_builder() {
    for helper in [
        "pub fn capture_tab_ai_focused_window_screenshot_file()",
        "pub fn capture_tab_ai_screen_screenshot_file()",
    ] {
        assert!(
            SCREENSHOT_FILES_SOURCE.contains(helper),
            "capture helper `{helper}` must remain as the public entry \
             point for the identity-bearing capture path — callers rely \
             on the returned TabAiScreenshotFile.path to thread identity"
        );
    }
    let builder_call_count = SCREENSHOT_FILES_SOURCE
        .matches("build_tab_ai_screenshot_filename(")
        .count();
    assert!(
        builder_call_count >= 3,
        "build_tab_ai_screenshot_filename must be invoked at least once in \
         each of the two capture helpers (plus the declaration site and \
         any tests). Found {builder_call_count}, expected >=3. If a \
         helper stops routing through the shared builder, identity \
         format diverges between focused-window and full-screen captures."
    );
}

#[test]
fn tab_ai_screenshot_file_has_identity_tuple_fields() {
    for field in [
        "pub path: String,",
        "pub width: u32,",
        "pub height: u32,",
        "pub title: String,",
        "pub used_fallback: bool,",
    ] {
        assert!(
            SCREENSHOT_FILES_SOURCE.contains(field),
            "TabAiScreenshotFile must retain field `{field}` — consumers \
             thread identity via this whole tuple (path is the primary \
             axis, width/height/title/used_fallback are the corroborating \
             metadata that lets Agent Chat render and caption the image \
             deterministically)"
        );
    }
}

#[test]
fn screenshot_path_is_threaded_into_context_text_block() {
    assert!(
        TAB_CONTEXT_SOURCE.contains("pub screenshot_path: Option<String>,"),
        "TabAiContextBlob must keep the `screenshot_path: Option<String>` \
         field — this is the slot the capture wires into and the context \
         builder reads from; dropping it severs identity threading at \
         the handoff between capture and context"
    );
    assert!(
        HARNESS_CONTEXT_SOURCE.contains("if let Some(path) = context.screenshot_path.as_deref() {")
            && HARNESS_CONTEXT_SOURCE.contains("push_line(&mut out, \"screenshot path\", path);"),
        "build_tab_ai_harness_context_block must emit a `screenshot path: \
         <path>` line when `context.screenshot_path` is Some — removing \
         the emission silently drops the path before Agent Chat sees it; \
         changing the label breaks match patterns elsewhere that rely on \
         the exact `screenshot path: ` prefix"
    );
}

#[test]
fn agent_chat_context_block_builder_is_text_only_preserving_path_identity() {
    assert!(
        AGENT_CHAT_CONTEXT_SOURCE.contains("pub(crate) fn build_tab_ai_agent_chat_context_blocks(")
            && AGENT_CHAT_CONTEXT_SOURCE
                .contains("ContentBlock::Text(TextContent::new(context_text))"),
        "build_tab_ai_agent_chat_context_blocks must wrap the harness text output \
         as a SINGLE `ContentBlock::Text` — if an image ContentBlock is \
         ever inserted here, Agent Chat would see two parallel identity channels \
         (text path + image bytes) that can drift apart, breaking the \
         story's \"matching identity\" clause"
    );
    assert!(
        AGENT_CHAT_CONTEXT_SOURCE
            .contains("screenshot_path_stays_in_text_context_without_image_block"),
        "the screenshot_path_stays_in_text_context_without_image_block \
         regression test in src/ai/agent_chat/ui/context.rs must remain — it is \
         the end-to-end pin that a Some(screenshot_path) surfaces in the \
         final Agent Chat context text with no image block"
    );
}
