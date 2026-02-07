# Container/Layout Improvements Audit

Date: 2026-02-07  
Agent: `codex-container-layout`  
Scope: `src/components/prompt_container.rs`, `src/components/prompt_layout_shell.rs`

## Executive Summary

The current container/layout system has two parallel abstractions with different behavior guarantees:

1. `PromptContainer` is slot-rich (`header/content/footer`) but leaves core flex/overflow behavior to callers.
2. `prompt_shell_container` / `prompt_shell_content` enforce safer flex/overflow defaults but expose only minimal configuration.

This split is the main source of layout inconsistency. The highest-value improvement is to converge both paths on one shared frame contract so all prompts get the same resize/overflow semantics and spacing defaults.

## Current State

### `PromptContainer` (`src/components/prompt_container.rs`)

1. Provides `header`, `content`, `footer`, and optional hint/divider slots (`src/components/prompt_container.rs:145`, `src/components/prompt_container.rs:171`, `src/components/prompt_container.rs:177`, `src/components/prompt_container.rs:183`).
2. Root container sets `flex_col`, `w_full`, `h_full`, rounded background, and text/font styling (`src/components/prompt_container.rs:240`-`src/components/prompt_container.rs:248`).
3. Content is passed through directly; caller must provide flex sizing (`src/components/prompt_container.rs:268`-`src/components/prompt_container.rs:273`).
4. Divider/hint rendering is duplicated inline in `render()` while helper methods exist but are unused (`src/components/prompt_container.rs:195`-`src/components/prompt_container.rs:227`, `src/components/prompt_container.rs:255`-`src/components/prompt_container.rs:295`).
5. No direct tests in this module (`src/components/prompt_container.rs:302`-`src/components/prompt_container.rs:309`).

### `prompt_layout_shell` (`src/components/prompt_layout_shell.rs`)

1. Shared shell enforces `min_h(0)` + `overflow_hidden` on both root and content wrappers (`src/components/prompt_layout_shell.rs:17`-`src/components/prompt_layout_shell.rs:18`, `src/components/prompt_layout_shell.rs:30`-`src/components/prompt_layout_shell.rs:31`).
2. API is intentionally small: radius + optional vibrancy background for container; a single content slot (`src/components/prompt_layout_shell.rs:10`, `src/components/prompt_layout_shell.rs:26`).
3. Test coverage only verifies string usage in `render_prompts/other.rs`, not layout behavior contracts (`src/components/prompt_layout_shell.rs:37`-`src/components/prompt_layout_shell.rs:67`).

### Adoption Pattern

1. `PromptContainer` is currently used in path prompt flow (`src/prompts/path.rs:668`-`src/prompts/path.rs:671`).
2. Most simple prompt wrappers use `prompt_shell_container` + `prompt_shell_content` (`src/render_prompts/other.rs:51`-`src/render_prompts/other.rs:54`, `src/render_prompts/other.rs:102`-`src/render_prompts/other.rs:105`, `src/render_prompts/other.rs:153`-`src/render_prompts/other.rs:156`).
3. Div/Webcam paths add extra wrapper layers and custom header/footer structure on top of shell patterns (`src/render_prompts/div.rs:123`-`src/render_prompts/div.rs:167`, `src/render_prompts/other.rs:341`-`src/render_prompts/other.rs:359`).

## Findings (Ranked)

### P1: Two container systems encode different layout guarantees

Evidence:

1. `PromptContainer` root lacks explicit `min_h(0)` and `overflow_hidden` (`src/components/prompt_container.rs:240`-`src/components/prompt_container.rs:248`).
2. `prompt_shell_container` always applies both (`src/components/prompt_layout_shell.rs:17`-`src/components/prompt_layout_shell.rs:18`).

Impact:

1. Prompts using different containers can diverge on overflow/shrinking behavior.
2. Fixes for one path do not automatically apply to the other.

Recommendation:

1. Introduce a single shared frame primitive (for example `PromptFrame`) used by both `PromptContainer` and `prompt_shell_container`.
2. Make `min_h(0)` + overflow policy part of the common baseline contract.

### P1: Content slot sizing is implicit and fragile

Evidence:

1. `PromptContainer` explicitly relies on callers to provide proper flex sizing (`src/components/prompt_container.rs:269`-`src/components/prompt_container.rs:273`).
2. Call sites must remember to add `flex_1` themselves (`src/prompts/path.rs:594`-`src/prompts/path.rs:595`).

Impact:

1. Easy to regress by forgetting a single `flex_1` at call sites.
2. Makes container behavior harder to reason about during refactors.

Recommendation:

1. Add explicit content policy in config, e.g. `content_layout: Fill | Intrinsic`.
2. Default to `Fill` so container owns common behavior and call sites opt out only when needed.

### P1: Spacing/margin behavior is duplicated and not token-driven

Evidence:

1. `PromptContainer` uses mixed units and manual rem conversion (`src/components/prompt_container.rs:205`-`src/components/prompt_container.rs:206`, `src/components/prompt_container.rs:262`-`src/components/prompt_container.rs:263`).
2. Divider and hint markup is duplicated inline despite helper methods (`src/components/prompt_container.rs:195`-`src/components/prompt_container.rs:227`, `src/components/prompt_container.rs:255`-`src/components/prompt_container.rs:295`).

Impact:

1. Padding/margin consistency drifts over time.
2. Any style tweak requires editing multiple blocks.

Recommendation:

1. Replace manual `margin / 16.0` conversions with shared spacing tokens.
2. Render divider/hint through one helper path and remove duplicated inline blocks.

### P2: Responsive behavior is under-configured

Evidence:

1. Shell API only accepts radius + background (`src/components/prompt_layout_shell.rs:10`).
2. Callers force fixed heights in some prompt flows (`src/render_prompts/div.rs:107`, `src/render_prompts/div.rs:124`, `src/render_prompts/other.rs:322`, `src/render_prompts/other.rs:346`).
3. Hint text in path flow can be long and is rendered as a full string footer with no truncation policy (`src/prompts/path.rs:648`-`src/prompts/path.rs:650`, `src/components/prompt_container.rs:281`-`src/components/prompt_container.rs:294`).

Impact:

1. Small window/display scenarios risk clipping or density collapse.
2. Layout behavior depends heavily on each prompt’s local implementation.

Recommendation:

1. Add shell config for min/max height strategy and optional density mode.
2. Add footer hint overflow policy (truncate, collapse, or hide metadata on narrow widths).

### P2: Container nesting depth is higher than needed in richer prompts

Evidence:

1. Div prompt nests shell root -> custom flex wrapper -> shell content wrapper -> prompt entity (`src/render_prompts/div.rs:123`-`src/render_prompts/div.rs:167`).
2. Webcam prompt duplicates shell-like root manually rather than using shared helper (`src/render_prompts/other.rs:341`-`src/render_prompts/other.rs:349`).

Impact:

1. More places for overflow/focus bugs.
2. Harder to enforce consistent padding and section boundaries.

Recommendation:

1. Introduce shared composition helpers for `header + divider + body + footer` to remove duplicated wrapper layers.
2. Migrate webcam/div wrappers to the same shared shell+slots pipeline.

### P3: Tests validate usage strings, not layout contracts

Evidence:

1. Shell tests rely on source-string checks (`src/components/prompt_layout_shell.rs:37`-`src/components/prompt_layout_shell.rs:67`).
2. `PromptContainer` explicitly has no tests (`src/components/prompt_container.rs:302`-`src/components/prompt_container.rs:309`).

Impact:

1. Behavior can regress while string-based tests still pass.
2. Refactors are constrained by brittle source checks.

Recommendation:

1. Move critical layout decisions into pure helper functions and unit-test them.
2. Keep one lightweight usage smoke test, but add behavior tests for slot composition policies.

## Proposed Refactor Plan

### Phase 1 (Low Risk, High Return)

1. Add shared frame config type (`PromptFrameConfig`) used by both container APIs.
2. Normalize root/content defaults: `flex_col`, `w_full`, `h_full`, `min_h(0)`, overflow behavior.
3. Remove duplicated divider/hint rendering in `PromptContainer`.

### Phase 2 (Behavior Hardening)

1. Add explicit content sizing mode (`Fill` vs `Intrinsic`).
2. Add hint/footer truncation policy for narrow widths.
3. Replace ad-hoc fixed-height call-site patterns with a common shell sizing policy.

### Phase 3 (Adoption + Cleanup)

1. Migrate div/webcam wrappers to shared slot helpers.
2. Reduce nested container depth where wrappers redundantly apply the same flex/overflow constraints.
3. Remove dead comments/allowances once tests cover new contracts.

## Suggested Tests (TDD Targets)

1. `test_prompt_frame_defaults_apply_min_h_and_overflow_hidden`
2. `test_prompt_container_content_fill_mode_wraps_child_with_flex_1`
3. `test_prompt_container_intrinsic_mode_leaves_content_unwrapped`
4. `test_prompt_container_hint_respects_truncation_policy_when_narrow`
5. `test_prompt_shell_and_prompt_container_share_same_root_contract`

## Quick Wins

1. Unify divider/hint rendering in `PromptContainer` to eliminate duplicate blocks.
2. Add `min_h(px(0.))` to `PromptContainer` root for parity with shell behavior.
3. Replace source-string-only shell tests with one behavior-oriented helper test.
