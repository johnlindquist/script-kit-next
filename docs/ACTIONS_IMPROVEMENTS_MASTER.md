# Actions Improvements Master Document

**Generated**: 2026-01-31
**Research Sources**: 15 parallel codex agents analyzing codebase + online research

---

## Executive Summary

This document consolidates findings from comprehensive research on action handling improvements for Script Kit GPUI. The research covered codebase analysis, Raycast/Alfred/VS Code patterns, accessibility, performance, theming, and UX best practices.

---

## Top 3 Priority Implementations (Recommended)

### 1. Fix Confirm Dialog Button Click Not Closing Popup (BUG)
**Impact**: High | **Effort**: Low | **Files**: `src/confirm/dialog.rs`

**Problem**: Button clicks call `on_choice` but don't close the confirm popup. Only `dispatch_confirm_key()` closes the window, which depends on main-window key routing.

**Fix**: Call `close_confirm_window()` in button click callbacks or inside `ConfirmDialog::submit/cancel`.

**Reference**: codex-11-confirm-actions.md

---

### 2. Hidden Actions Can Still Trigger via Shortcuts (BUG/SECURITY)
**Impact**: High | **Effort**: Medium | **Files**: `src/prompt_handler.rs`, `src/app_actions.rs`

**Problem**: `action_shortcuts` is built from ALL SDK actions including `visible: false`. Hidden actions can still be executed via keyboard shortcuts.

**Fix**: Filter by `visible` (and `enabled` if added) when building shortcut maps and when triggering actions.

**Reference**: codex-4-action-state.md

---

### 3. Add Debounce to Action Search Input (PERFORMANCE)
**Impact**: High | **Effort**: Low | **Files**: `src/actions/dialog.rs`, `src/app_impl.rs`

**Problem**: Every keystroke triggers full re-filter + sort + window resize + NSWindow enumeration. Causes jank on rapid typing.

**Fix**:
- Add 16-30ms debounce for action search input
- Cache lowercase title/description/shortcut in `Action` struct
- Only call resize when result count changes

**Reference**: codex-15-action-performance.md

---

## All Findings by Category

### Bugs & Issues

| Issue | Severity | File | Description |
|-------|----------|------|-------------|
| Button click doesn't close confirm | High | confirm/dialog.rs | Clicks call callback but don't close popup |
| Hidden actions trigger via shortcuts | High | prompt_handler.rs | visible:false not respected in shortcuts |
| Multi-char keys become multiple keycaps | Medium | actions/dialog.rs | "Delete", "PageUp" split into chars |
| ActionsWindow ignores close:false on Enter | Medium | actions/window.rs | Conflicts with ProtocolAction.close |
| Duplicate key handling paths in confirm | Medium | confirm/window.rs | Both ConfirmWindow and dispatch_confirm_key handle keys |
| ShowDiv/ShowForm don't rebuild action_shortcuts | Medium | prompt_handler.rs | Shortcuts inconsistent across prompt types |

### Performance Bottlenecks

| Bottleneck | Impact | Fix |
|------------|--------|-----|
| Per-keystroke full re-filter + sort | High | Debounce + cache lowercase strings |
| NSWindow enumeration on every keystroke | High | Cache window handle |
| Scriptlet actions re-parsed from disk | Medium | Cache by path + mtime |
| Per-render list setup clones grouped_items | Low | Precompute after refilter |

### Theming Gaps

| Gap | Files | Fix |
|-----|-------|-----|
| Hardcoded light-mode colors (0xE8E8E8..) | actions/dialog.rs | Use theme tokens |
| Fixed dialog opacity (0.50/0.95) | actions/dialog.rs | Use theme.opacity.dialog |
| Mixed theme sources in Notes | notes/actions_panel.rs | Unify to single theme |
| Focus tied to text content, not state | actions/dialog.rs | Use focus_handle.is_focused() |

### Accessibility Gaps (WCAG)

| Gap | WCAG | Fix |
|-----|------|-----|
| No programmatic roles/names | 1.3.1, 4.1.2 | Add accessibility metadata |
| Popup doesn't take focus | 2.4.3, 2.4.7 | Reconsider focus strategy |
| No screen-reader announcements | 4.1.3 | Add live region announcements |
| Shortcut hints visual-only | 1.3.1, 3.3.2 | Expose to screen readers |
| UnifiedListItem a11y fields unused | 4.1.2 | Wire into rendering |

### Missing Features

| Feature | Priority | Reference |
|---------|----------|-----------|
| Right-click context menus | High | Raycast/Alfred pattern |
| Ctrl+N/Ctrl+P list navigation | Medium | Raycast keyboard UX |
| Cmd+Enter secondary action | Medium | Raycast action semantics |
| Shortcut customization UI | Medium | Raycast settings pattern |
| Motion tokens for animations | Low | Design system research |

### Architecture Improvements

| Improvement | Impact | Description |
|-------------|--------|-------------|
| Unify action bars into shared component | High | Reduce duplication in header/footer/modals |
| Single "context actions" API | High | Remove duplicated setup in render_* files |
| Consolidate key routing | Medium | Migrate to route_key_to_actions_dialog() |
| Unify window/overlay behavior | Medium | Consistent focus and sizing |
| Platform-aware shortcut display | Low | Windows/Linux support |

---

## Raycast Patterns Worth Adopting

1. **Cmd+K as universal command palette** - Already implemented
2. **Enter = primary, Cmd+Enter = secondary** - Partially implemented
3. **Action Panel with sections** - Already implemented via ActionCategory
4. **Shortcut hints in action list** - Already implemented
5. **Ctrl+N/Ctrl+P navigation** - Not implemented
6. **Primary action always safe/obvious** - Design consideration

---

## Implementation Recommendations

### Immediate (This Sprint)
1. Fix confirm button click closing
2. Fix hidden actions triggering via shortcuts
3. Add action search debounce

### Short Term (Next 2 Weeks)
4. Replace hardcoded light-mode colors
5. Fix multi-character key parsing
6. Add right-click context menus
7. Unify action bar component

### Medium Term (Next Month)
8. Add Ctrl+N/Ctrl+P navigation
9. Improve accessibility (roles, announcements)
10. Cache scriptlet actions
11. Platform-aware shortcuts

---

## File Quick Reference

| Area | Key Files |
|------|-----------|
| Actions Core | `src/actions/{dialog,window,builders,types}.rs` |
| Confirm Dialog | `src/confirm/{dialog,window}.rs` |
| Prompt Handling | `src/prompt_handler.rs`, `src/app_impl.rs` |
| Action Execution | `src/app_actions.rs` |
| UI Components | `src/components/{button,prompt_footer,prompt_header}.rs` |
| Notes Actions | `src/notes/{window,actions_panel}.rs` |
| Theming | `src/theme/types.rs` |
| Shortcuts | `src/shortcuts/*` |

---

## Research Sources

1. codex-1-actions-prompts.md - Codebase action flow analysis
2. codex-2-keyboard-shortcuts.md - Shortcut handling patterns
3. codex-3-action-ui.md - UI component audit
4. codex-4-action-state.md - State management analysis
5. codex-5-action-accessibility.md - WCAG accessibility review
6. codex-6-raycast-actions.md - Raycast action patterns
7. codex-7-raycast-keyboard.md - Raycast keyboard UX
8. codex-8-alfred-actions.md - Alfred comparison
9. codex-9-launcher-ux.md - Launcher UX patterns
10. codex-10-action-animations.md - Animation best practices
11. codex-11-confirm-actions.md - Confirm dialog analysis
12. codex-12-context-menus.md - Context menu patterns
13. codex-13-action-theming.md - Theming analysis
14. codex-14-discoverability.md - UX discoverability research
15. codex-15-action-performance.md - Performance bottlenecks
