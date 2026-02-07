# Animation and Transition Improvements Audit

## Scope
- Audited `src/**/*.rs` for transition usage, view switches, filtering flows, resize behavior, and element show/hide paths.
- Focus areas requested: view changes, list filtering, window resize, and element appearance/disappearance.

## Executive Summary
The codebase has a solid transition foundation (`src/transitions.rs`) but most runtime UI behavior still uses hard cuts. The largest sources of jank are:
1. Window height changes that are always non-animated.
2. Immediate view swaps and list reset/scroll jumps during filtering.
3. Overlay and toast paths that support animation in theory but are mostly wired as instant show/hide.

## Findings (Ordered by Impact)

### P0: Window resize is explicitly non-animated in primary and actions windows
**Evidence**
- Main window resize calls Cocoa `setFrame(... animate:false)` in `src/window_resize.rs:215`.
- Actions popup uses the same non-animated pattern in `src/actions/window.rs:585` and `src/actions/window.rs:742`.
- Filtering drives frequent resize updates (`update_window_size`) in `src/app_impl.rs:3217` and `src/app_impl.rs:3494-3498`.

**User impact**
- Height jumps feel abrupt during filter typing and view transitions.
- Abrupt geometry shifts are most noticeable on larger monitor distances and when result counts change rapidly.

**Recommendation**
- Add a resize animator that lerps height over 120-180ms (ease-out), rather than direct jumps.
- Keep coalescing (`src/window_ops.rs:75-209`) but animate toward the latest target.
- For fast typing bursts: retarget active animation rather than restarting from stale heights.

**Implementation sketch**
- Introduce `WindowResizeAnimationState { start_height, target_height, started_at, duration_ms, correlation_id }`.
- Tick via `Timer::after(16ms)` until complete; call existing resize primitive each frame.
- Log `ANIM_RESIZE_START`, `ANIM_RESIZE_RETARGET`, `ANIM_RESIZE_END` with `correlation_id`.

---

### P0: View transitions are hard swaps with no enter/exit animation
**Evidence**
- Render path directly matches `current_view` and returns a single view tree in `src/main.rs:1716-1804`.
- Reset path hard-switches view state (`self.current_view = AppView::ScriptList`) in `src/app_impl.rs:6952`.
- Reset also immediately clears selection/scroll state (`scroll_to` top) in `src/app_impl.rs:6989-6993`.

**User impact**
- Prompt-to-prompt and prompt-to-script-list navigation feels sudden.
- Context switch is visually harsh when content density differs (e.g., editor -> list).

**Recommendation**
- Implement a lightweight crossfade+slide transition for `AppView` changes.
- Keep business state changes immediate, but animate rendering between previous and next views for ~140ms.
- Prefer directional rules (e.g., prompt open = slide up/fade in, return to list = slide down/fade out).

**Implementation sketch**
- Track `last_view_snapshot` + `view_transition_progress` in app state.
- On view change, render both layers in a temporary stack until progress reaches 1.0.
- Reuse easing constants from `src/transitions.rs` instead of inventing ad-hoc durations.

---

### P1: List filtering resets selection/scroll instantly; no item enter/exit transitions
**Evidence**
- Multiple list views reset selection to 0 and scroll instantly to top:
  - `src/app_impl.rs:2735-2737`
  - `src/app_impl.rs:2761-2763`
  - `src/app_impl.rs:2774-2776`
  - `src/app_impl.rs:2787-2789`
  - `src/app_impl.rs:2800-2802`
- Main filter coalescer updates computed filter and triggers resize + notify immediately in `src/app_impl.rs:3183-3234`.
- File search avoids flashes with frozen filters but still does hard list transitions on first/done batches (`src/app_impl.rs:2930-2961`, `src/app_impl.rs:3091-3104`).

**User impact**
- Result sets "snap" rather than transition.
- Scroll and selection discontinuities are visually jarring when typing quickly.

**Recommendation**
- Add animated list-state transitions:
  - Fade/slide rows in on enter (80-120ms).
  - Fade rows out on exit when practical.
  - Smooth scroll-to-top over short duration rather than instant jump.
- Maintain current coalescing pipeline; add visual interpolation only in render layer.

**Implementation sketch**
- Keep previous filtered ids for one frame window.
- Build `ListDiffTransitionState` keyed by command id/script id.
- Apply per-item opacity/offset from transition progress during render.

---

### P1: Transition utility module is largely unused in runtime UI
**Evidence**
- `src/transitions.rs` is marked dead-code tolerant (`#![allow(dead_code)]`) at `src/transitions.rs:6`.
- `lib.rs` claims transitions are used broadly (`src/lib.rs:233-237`), but practical usage is sparse.
- `AppearTransition` is only directly wired in toast component (`src/components/toast.rs:204`, `src/components/toast.rs:286`, `src/components/toast.rs:363-365`).

**User impact**
- Inconsistent motion language: some components expose transition APIs while most UI remains static.

**Recommendation**
- Standardize on a single motion token system (durations/easing) from `src/transitions.rs`.
- Remove dead paths or integrate them into core flows (view switch, list filtering, overlays, toasts).
- Add a small `animation policy` doc/table mapping interaction -> duration/easing.

---

### P2: Overlay appearance/disappearance is mostly instantaneous
**Evidence**
- Overlays are toggled via `when_some` without transition wrappers in `src/main.rs:1918-1924`.
- Shortcut recorder overlay appears/disappears as an absolute full-screen child with no transition state in `src/app_impl.rs:5062-5069`.
- Alias input overlay returns directly from entity with no staged entry/exit in `src/app_impl.rs:5389-5390`.

**User impact**
- Modal overlays can pop in/out abruptly, especially after keyboard-triggered actions.

**Recommendation**
- Add shared overlay transition helper: backdrop fade (100ms) + panel scale/slide (120-160ms).
- Delay focus transfer until first frame after overlay enter starts, to avoid visible focus jumps.

---

### P2: Toast path has transition fields, but pipeline strips richer transition intent
**Evidence**
- Toast component supports transition state (`src/components/toast.rs:204`, `src/components/toast.rs:279-302`, `src/components/toast.rs:363-365`).
- Toast manager drains pending toasts into a simplified struct without transition metadata (`src/toast_manager.rs:332-342`, `src/toast_manager.rs:353-360`).
- App converts pending to gpui notifications in `src/app_impl.rs:6333-6348` and `src/main.rs:303-327`.

**User impact**
- Custom toast transition capabilities are not consistently preserved through the full delivery path.

**Recommendation**
- Either:
  - Commit to gpui-component notifications and remove unused custom transition APIs, or
  - Keep custom toasts end-to-end and preserve transition metadata through manager + renderer.
- Avoid maintaining both patterns unless both are actively needed.

---

### P2: Secondary window resize behavior is also abrupt
**Evidence**
- Notes auto-sizing and panel-related resizing use direct `window.resize(...)` with no interpolation in `src/notes/window.rs:689`, `src/notes/window.rs:2516`, and `src/notes/window.rs:2527`.

**User impact**
- Notes window can visibly jump as content grows/shrinks or panels open/close.

**Recommendation**
- Reuse the same shared resize animator approach for secondary windows where dynamic height changes are user-visible.
- Clamp animation duration for accessibility and keep it interruptible.

## Consistency Gaps
- Motion behavior is inconsistent across modules: immediate hover updates (`src/render_script_list.rs:283-311`) + static state color switches (`src/list_item.rs:1173-1190`) coexist with mostly non-animated view and layout transitions.
- Timing constants are spread across modules (`8ms`, `16ms`, `30ms`, `75ms`, etc. in `src/app_impl.rs`) with limited shared animation semantics.

## Proposed Rollout Plan

### Phase 1 (High ROI, low risk)
1. Add shared resize animator for main window + actions window.
2. Add motion telemetry with `correlation_id` for resize and view-switch events.
3. Define motion tokens from `src/transitions.rs` and consume them in resize/view code.

### Phase 2 (Core UX polish)
1. Add `AppView` crossfade/slide transition wrapper in main render path.
2. Add smooth list filter transitions for script list and file search results.
3. Add smooth scroll-to-top behavior where selection resets to item 0.

### Phase 3 (System consistency)
1. Unify overlay enter/exit transitions (alias, shortcut recorder, command palettes).
2. Resolve toast architecture split (custom transition path vs gpui notifications).
3. Extend shared motion policy to Notes and AI windows.

## Verification Plan (for future implementation)
- Build gate: `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`.
- Runtime checks with compact logs:
  - `echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
  - Verify `ANIM_*` logs include `correlation_id`, duration, and target state.
- UI checks via stdin scripts + screenshot capture:
  - Filter typing scenarios (script list + file search).
  - Prompt/view transitions (script list <-> editor/chat/arg).
  - Window resize transitions under rapid filter updates.

## Risks and Tradeoffs
- Over-animating list updates can increase frame cost; keep transitions short and diff-based.
- Animated resize must remain interruptible under rapid typing to avoid lag.
- Focus handling can regress if overlays animate without careful focus timing.
- Cross-platform behavior differs; Cocoa-specific animation hooks should be behind platform guards.
