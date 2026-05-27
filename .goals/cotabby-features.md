# Cotabby Feature Analysis: Top 5 Implementation Plans

> **Date:** 2026-05-27
> **Source:** Full Cotabby codebase analysis (~132 Swift files) + 6 Oracle sessions
> **Oracle Sessions:** `cotabby-top5-for-scriptkit`, `ghost-text-impl-plan`, `permission-onboarding-plan`, `context-aware-prediction-plan`, `staleproof-prediction-plan`, `visual-context-ocr-plan`

## Executive Summary

Cotabby is a macOS menu-bar AI autocomplete app. We analyzed its full codebase and ran 6 Oracle/GPT-5.5 sessions to produce implementation-ready plans for the top 5 features Script Kit GPUI should borrow. Each plan includes exact file targets, data models, rendering approaches, and verification checklists.

**Do not port Cotabby's AI generation pipeline.** Borrow its polished **interaction patterns**: inline predictability, guided permission recovery, explicit context, and stale-proof state.

| Rank | Feature | Impact / Effort | Status | Oracle Session |
|------|---------|-----------------|--------|----------------|
| 1 | Ghost text in main launcher input | Very high / medium | **IMPLEMENTED** (12 tests) | `ghost-text-impl-plan` |
| 2 | Permission onboarding wizard + drag-to-Settings | Very high / medium-high | **MODEL LAYER** (9 tests) | `permission-onboarding-plan` |
| 3 | Clipboard/context-aware command prediction | High / low-medium | **IMPLEMENTED** (9 tests) | `context-aware-prediction-plan` |
| 4 | Stale-proof prediction/session controller | Medium-high / low-medium | **IMPLEMENTED** (2 tests) | `staleproof-prediction-plan` |
| 5 | Visual context capture/OCR as @ context source | High / high | **PLANNED** | `visual-context-ocr-plan` |

---

## Implementation Status

### What's Done (39 tests pass)
- **Feature 1:** Ghost text prediction engine, overlay rendering, Tab acceptance, typed-through reconciliation, getState protocol field
- **Feature 2:** Unified 5-kind permission model (Accessibility, ScreenRecording, EventSynthesizing, InputMonitoring, Microphone), detection functions, startup intent, Settings deep-links, persistence
- **Feature 3:** Launcher context snapshot with clipboard content detection, freshness TTLs, sensitive content filtering, context boost wired into search sort
- **Feature 4:** PredictionRevision model, monotonic ghost_id for acceptance validation, revision-aware prediction computation

### What's Remaining
- **Feature 2 UI:** GPUI onboarding wizard surface, permission card rendering, drag-to-Settings NSPanel overlay, post-onboarding reminders
- **Feature 3 UI:** Context chips in launcher header, capture-on-open wiring, browser URL and selected text context sources
- **Feature 5:** ScreenCaptureKit capture, Vision OCR Swift helper, `@ Visible Text` context menu entry, MCP resource

---

## Manual Testing User Stories

### Feature 1: Ghost Text
1. Open Script Kit, type `cl` — expect dim ghost suffix `ipboard History` after cursor
2. Type `clip` — ghost suffix shrinks to `board History` without flicker (typed-through)
3. Press Tab — input becomes `Clipboard History`, nothing executes, results update
4. Press Enter — Clipboard History command executes
5. Type `se` — ghost shows `ttings` or `arch Files` depending on dominant result
6. Type a fuzzy non-prefix query like `cbh` — no ghost text appears (only prefix matches ghost)
7. Open actions popup (Cmd+K) — ghost disappears
8. Type `@` or `;` — ghost disappears (menu syntax takes over)
9. Clear input — ghost disappears

### Feature 2: Permission Model
1. Run `check_all_permissions()` — returns status for Accessibility, ScreenRecording, etc.
2. Run `PermissionSnapshot::current()` — shows 5 permission cards with live status
3. Run `open_permission_settings(PermissionKind::Accessibility)` — opens System Settings to correct pane
4. Check `startup_intent(true)` — returns `OpenFullWizard` when required permissions missing
5. Check `startup_intent(false)` after `mark_onboarding_completed()` — returns `ShowReminder` if permissions revoked

### Feature 3: Context-Aware Prediction
1. Copy a URL (`https://example.com`), open Script Kit — URL-related commands boost higher
2. Copy a file path (`/Users/john/README.md`), open Script Kit — file-related commands boost
3. Copy code snippet, open Script Kit — code-related commands boost
4. Copy sensitive content (`sk-proj-xxx`), open Script Kit — no context boost applied (filtered)
5. Wait 5+ minutes after copying — context boost expires (300s TTL)

### Feature 4: Stale-Proof Predictions
1. Type fast through a ghost suggestion — suffix advances smoothly, no flicker
2. Tab always accepts the currently visible ghost text, not a stale prediction
3. Script catalog reload doesn't show an old ghost for a different query

### Feature 5: Visual OCR (Planned)
1. Open a dense app window (e.g., error dialog), invoke Script Kit
2. Select `@ Visible Text` in context menu
3. Script Kit hides, captures frontmost window screenshot
4. OCR extracts text, shows as context chip: `Visible Text · Safari`
5. Submit to Agent Chat — model receives extracted text
6. `kit://context/visible-text` MCP resource returns cached OCR text

---

## Feature 1: Ghost Text in Main Launcher Input

**Oracle Session:** `~/.oracle/sessions/ghost-text-impl-plan/output.log` (36k chars)

### Core Approach
Implement as a **GPUI-native input decoration** (overlay div), not an external NSPanel. Script Kit owns the text field rendering.

### Key Data Model
```rust
struct GhostPrediction {
    query: SharedString,
    candidate_id: CommandId,
    full_label: SharedString,
    ghost_suffix: SharedString,
    source: GhostSource, // Prefix, Fuzzy, Recent, Contextual, Fallback
    confidence: f32,
}
```

### Data Flow
1. Keystroke → `handle_filter_input_change()` → `queue_filter_compute()`
2. → `get_grouped_results_cached()` → nucleo search + frecency + grouping
3. → **NEW:** `compute_ghost_prediction()` from final grouped results
4. → Render ghost suffix as dim overlay div after typed text in input

### Confidence Gate (Conservative)
- Top result label must **case-insensitively start** with typed fragment (prefix only)
- Match tier >= 950 (prefix or better)
- Score gap > 200 from second result
- No ghost for: empty query, 1 char, trailing space, fuzzy-only, fallback rows, actions popup, IME composition, @ prefix, sigil syntax

### Acceptance
- **Tab** accepts ghost into query (after menu-syntax checks, before ACP capture)
- **Right Arrow** accepts when caret is at end
- **Enter** still executes selected result (unchanged)

### File Changes
| File | Change |
|------|--------|
| `src/scripts/search/ghost.rs` | **NEW** — GhostPrediction, compute_ghost_prediction(), confidence logic |
| `src/scripts/search.rs` | Re-export ghost module |
| `src/main_sections/app_state.rs` | Add `ghost_prediction: Option<GhostPrediction>` field |
| `src/app_impl/filter_input_change.rs` | Reconcile/clear ghost on input change, typed-through detection |
| `src/app_impl/filtering_cache.rs` | Compute ghost after grouped results, before cache store |
| `src/app_impl/startup.rs` | Tab/Right Arrow acceptance in key interceptors |
| `src/render_script_list/mod.rs` | `render_search_input_with_ghost()` overlay rendering |

### Verification
- Type `se` → ghost shows `arch Files` (suffix only)
- Tab → input becomes full label, nothing executes
- Enter → executes selected command
- Type through ghost char by char → suffix shrinks without flicker
- Fuzzy non-prefix query → no ghost
- Actions popup / @ syntax / IME → ghost hidden

---

## Feature 2: Permission Onboarding Wizard

**Oracle Session:** `~/.oracle/sessions/permission-onboarding-plan/output.log` (83k chars)

### Core Approach
Unify 3 existing partial permission systems (`permiso_detect.rs`, `permissions_wizard.rs`, `platform/permiso/`) into a real first-run GPUI onboarding surface with Cotabby-inspired drag-to-Settings overlay.

### Permission Model
```rust
enum PermissionKind {
    Accessibility,      // Required — selected text, window control
    ScreenRecording,    // Required — screenshots, visual context
    EventSynthesizing,  // Required — synthetic paste, keypresses
    InputMonitoring,    // Recommended — global key interception
    Microphone,         // Optional — dictation
}

enum PermissionRequirement { Required, Recommended, Optional }
```

### Onboarding Steps
1. **Welcome** — logo, tagline, "Get Started"
2. **Permissions** — glass-material cards with live status, Allow buttons, disabled Continue until all required granted
3. **Done** — success, context features unlocked

### The Drag-to-Settings Overlay (Killer Feature)
1. Click "Allow" → native TCC prompt via `AXIsProcessTrustedWithOptions`
2. If not granted → open System Settings via `x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility`
3. Create non-activating NSPanel overlay (via objc2/cocoa) at statusBar level
4. Locate System Settings window via `CGWindowListCopyWindowInfo`
5. Spring-animate overlay from button position to Settings window
6. Show draggable app icon row (`NSDraggingSource` with app bundle URL)
7. Auto-dismiss when permission poll detects grant

### Settings Deep-Link URLs
| Permission | URL |
|-----------|-----|
| Accessibility | `x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility` |
| Screen Recording | `x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture` |
| Input Monitoring | `x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent` |
| Microphone | `x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone` |

### File Changes
| File | Change |
|------|--------|
| `src/platform/permiso_detect.rs` | Add `input_monitoring_authorized()`, `event_synthesizing_authorized()` |
| `src/permissions_wizard.rs` | Replace single-permission model with full `PermissionKind` enum, cards, snapshot, startup intent |
| `src/platform/permiso/panel.rs` | Extend `PermisoPanel` with all permission types and Settings URLs |
| `src/platform/permiso/host_app.rs` | Resolve app bundle URL + icon for drag source |
| `src/platform/permiso/locator.rs` | Find System Settings window via `CGWindowListCopyWindowInfo` |
| `src/platform/permiso/drag_source.rs` | `NSDraggingSource` AppKit view with app bundle URL payload |
| `src/platform/permiso/overlay_window.rs` | Non-activating NSPanel with spring animation |
| `src/main_entry/app_run_setup.rs` | Wire startup intent after ScriptListApp creation |
| `src/render_builtins/permission_onboarding.rs` | **NEW** — GPUI onboarding renderer |
| `src/builtins/mod.rs` | Add `OpenPermissionOnboarding` builtin entry |

### Post-Onboarding
- Persist `~/.scriptkit/permission-onboarding.json` with completion timestamp
- On subsequent launches: check for revoked permissions → show compact reminder
- Rate-limit reminders to once per day

---

## Feature 3: Context-Aware Command Prediction

**Oracle Session:** `~/.oracle/sessions/context-aware-prediction-plan/output.log` (66k chars)

### Core Approach
Capture context (clipboard, selected text, browser URL, frontmost app) when launcher opens. Use it to **boost commands** that can use that context. Show visible **context chips** in the header.

### ContextSnapshot Model
```rust
struct ContextSnapshot {
    generation: u64,
    items: Vec<ContextItem>,
    primary_item_id: Option<String>,
}

struct ContextItem {
    source: ContextSource,     // Clipboard, SelectedText, BrowserTab, etc.
    kind: ContextKind,         // Url, File, Image, Code, PlainText, etc.
    label: String,             // "Clipboard: URL"
    preview: String,           // redacted, capped
    captured_at_ms: i64,
    ttl_ms: u64,
    sensitive: bool,
}
```

### Context → Command Boosting
Integrate into existing `build_search_mode_results()` sort alongside frecency and preferred-result bonuses:
```rust
final_context_bonus = clamp((base + overlap) * freshness * source_weight, 0, 450)
// Kept below preferred-result bonus (500) so explicit selection still wins
```

### Freshness Windows
| Source | TTL |
|--------|-----|
| Clipboard | 300s |
| Selected text | 30s |
| Browser URL | 60s |
| Frontmost app | 15s |

### Privacy
- Never log raw preview content
- Detect API keys, private keys, JWTs, passwords → mark sensitive, hide preview, skip boost
- Cap previews at 160 chars, tokens at 256

### File Changes
| File | Change |
|------|--------|
| `src/context_snapshot/types.rs` | ContextSnapshot, ContextItem, ContextKind, ContextSource |
| `src/context_snapshot/detect.rs` | **NEW** — content type detection |
| `src/context_snapshot/privacy.rs` | **NEW** — redaction, sensitive detection |
| `src/context_snapshot/boost.rs` | **NEW** — context_boost_for_result() |
| `src/scripts/grouping/search_mode.rs` | Add context_snapshot param, context bonus in sort |
| `src/render_script_list/mod.rs` | Render context chips in header |
| `src/main_sections/app_state.rs` | Add context_snapshot fields |

---

## Feature 4: Stale-Proof Prediction/Session Controller

**Oracle Session:** `~/.oracle/sessions/staleproof-prediction-plan/output.log` (55k chars)

### Core Approach
Build a revision-gated state machine that prevents ghost text flicker, stale predictions, and wrong-query acceptance.

### Revision Model
```rust
#[derive(Clone, Copy, PartialEq, Eq)]
struct PredictionRevision {
    query_rev: u64,     // incremented on every filter text change
    catalog_rev: u64,   // incremented on script/builtin/source refresh
    context_rev: u64,   // incremented on context snapshot change
}
```

### State Machine
```
Idle → Computing(work_id, revision) → Active(ghost, revision) → Stale
                                    ↗ Failed
```

### Key Invariants
- **Latest wins:** monotonic `work_id` rejects results from older computations
- **Typed-through:** if user types chars matching ghost suffix, advance without recomputing
- **Acceptance guard:** Tab only accepts if `last_rendered_ghost_id` matches current prediction
- **Invalidation:** query change, catalog refresh, context change, view change, arrow navigation, actions popup

### File Changes
| File | Change |
|------|--------|
| `src/scripts/search/ghost.rs` | Add PredictionRevision, PredictionSessionController, state machine |
| `src/app_impl/filter_input_change.rs` | Call controller.on_input_change() for reconciliation |
| `src/app_impl/filtering_cache.rs` | Refresh revision signatures after grouped results |
| `src/app_impl/startup.rs` | Guard Tab acceptance with rendered ghost ID validation |

---

## Feature 5: Visual Context Capture/OCR

**Oracle Session:** `~/.oracle/sessions/visual-context-ocr-plan/output.log` (53k chars)

### Core Approach
Add `@ Visible Text` and `@ Screenshot` to the existing @ context menu. On-demand capture of frontmost window → OCR via Apple Vision → attach text as context chip.

### State Machine
```rust
enum VisualContextState {
    Idle,
    Capturing,
    ExtractingText,
    Ready { excerpt: String, image_id: Option<ImageId> },
    Unavailable(String),
    Failed(String),
}
```

### OCR Pipeline
1. User selects `@ Visible Text` in context menu
2. Hide/defer Script Kit window
3. Capture frontmost window via ScreenCaptureKit (by PID)
4. Run Apple Vision OCR via Swift helper (`VNRecognizeTextRequest`)
5. Sanitize, cap, redact OCR text
6. Show as context chip: `Visible Text · Safari`
7. Attach to ACP Agent Chat or script input

### File Changes
| File | Change |
|------|--------|
| `src/platform/visual_context_capture.rs` | **NEW** — screenshot capture facade |
| `src/platform/vision_ocr.rs` | **NEW** — Rust OCR wrapper |
| `src/platform/macos_vision_ocr.swift` | **NEW** — Swift Vision OCR helper |
| `src/context_snapshot/types.rs` | Add VisibleTextContext struct |
| `src/mcp_resources/mod.rs` | Add `kit://context/visible-text` resource |
| `src/ai/acp/context_parts.rs` | Add VisibleText/Screenshot attachment kinds |

---

## Recommended Build Order

1. **Ghost text** (Feature 1) — deterministic, prefix-based, no async complexity
2. **Permission onboarding** (Feature 2) — unlocks selected text + screenshots
3. **Context-aware boosting** (Feature 3) — reuses existing surfaces
4. **Prediction controller** (Feature 4) — hardens ghost text for async + context
5. **Visual OCR context** (Feature 5) — highest effort, biggest AI differentiator

## Full Oracle Output Locations

Each Oracle session output contains the complete implementation plan with exact line numbers, code sketches, edge cases, and test plans:

```
~/.oracle/sessions/cotabby-top5-for-scriptkit/output.log    — Feature ranking & rationale
~/.oracle/sessions/ghost-text-impl-plan/output.log          — Ghost text (36k chars)
~/.oracle/sessions/permission-onboarding-plan/output.log    — Permissions (83k chars)
~/.oracle/sessions/context-aware-prediction-plan/output.log — Context prediction (66k chars)
~/.oracle/sessions/staleproof-prediction-plan/output.log    — Session controller (55k chars)
~/.oracle/sessions/visual-context-ocr-plan/output.log       — Visual OCR (53k chars)
```
