# Script Kit polish audit: what will my next keystroke do?

**Finish line:** In any feature, someone can tell where typing goes, what is selected, what Return will do to it, and how to get back. After acting, they can tell whether it worked. The layout should communicate those answers before they have to read a hint.

**Recommended order:** establish coverage → audit the shared keyboard and visual rules together → walk every feature family → revisit the native boundaries. Fix and verify one complete interaction at a time. This is an execution plan, not a claim that the app has passed.

**Audit log:** [01 · Invalid form → correction → script response](form-correction.md) ([screenshots](https://brave-essence-5njm.here.now/)); [02 · Native form foundation](native-form-foundation.md) ([screenshots](https://ivory-sierra-cm9e.here.now/)).

## 1. Build the list before grading it

Start with the read-only `bun scripts/devtools/surfaces.ts report`. At planning time it found **34 contract kinds, 49 mappings and 48 view variants**. The generated projection agreed with source, but the coverage binding was unusable: it expected 37 kinds / 54 mappings. The [atlas](../../feature-map/index.md) also lists 37 entries. Trace the differences to a live route, an alias, a retired route, or a genuine omission. Don’t change an expected count just to turn the report green. This blocks a claim of *complete coverage*, not useful work on known features.

Use [FEATURES.md](../../FEATURES.md) for user outcomes, the [surface glossary](../../GLOSSARY.md) for owners, and cross-check four lists:

- [App views](../../src/main_sections/app_view_state.rs) **and** [views inside the launcher](../../src/one_window/). A single `ScriptList` can contain chat, a form, a connection, a staged action or a voice session.
- [Registered built-ins](../../src/builtins/mod.rs), their subcommands, staged actions and configured shortcuts. Commands that act without opening a screen still count.
- SDK prompt families, tray/hotkey/deeplink entry points, and agent-facing operations. Record separate routes when they change the initial context, selected subject, permissions or return destination.
- [Owned fixtures](../../src/design_evaluation/fixture_ids.rs), existing chapters and runtime receipts. These tell us **what we can prove**, not everything the product does. Audit third-party scripts at their host and SDK boundaries, not the contents of every personal script.

Keep one working table beside the audit evidence; no new dashboard. One row is a **feature + meaningful route + outcome**, not a screenshot. Record: stable ID · entry and starting state · visible promise · key/click and expected target · observed effect and return state · owner · build/evidence · verdict. Use `not run`, `failed`, `verified (scope)`, `blocked (prerequisite)` or `intentional exception (reason)`. Link aliases to their canonical row; don’t count them twice. Diff the inventory at the end so features added during the audit aren’t silently omitted.

## 2. What to check on every applicable route

### Keyboard and action contract

| Gesture | Question the walkthrough must answer |
|---|---|
| **Return / primary capsule** | Do key and click perform the same **named action on the same item exactly once**? Try a valid target, zero matches, disabled/invalid input, and a target changed by delayed results. Observe the resulting file, view, reply, or dispatch receipt—not just a pressed button. During the 400 ms target-change guard, an early Return must not run the replacement. |
| **Tab / Shift-Tab** | Which control owns Tab here? A form traverses fields; a completion accepts the suggestion; some lists change ordering; an editor may indent. Test both directions where applicable, first/last field, and Tab with **no** completion. Focus, caret, active border and destination of the next character must agree. Don’t standardize away useful differences; make the active meaning apparent. |
| **Arrows / selection** | With text focused, do arrows edit the insertion point or navigate results as intended? Verify boundary rows, grids, selection after filtering, and reveal of the selected item past the footer. Hover must not silently change the keyboard target. |
| **⌘K / ⌘↵ / numbered shortcuts** | Stage the intended subject; distinguish a clean chat from an explicit handoff. Test empty input, typed text and an already-open view. Check whether a global, view or editor binding wins; one chord must not trigger two handlers. After remapping, displayed keys, help, recorder and actual dispatch must agree. |
| **Esc / ⌘Z** | Dismiss the innermost transient state first; then go back one step with the right words, selection, draft and scroll position. Don’t lose unsaved work. Editor undo must not become action undo. An irreversible action waits in the line; stopping it must leave the underlying state untouched. A reversible action offers Undo only while it can run. |

For routing defects, compare capture, view/bubble, and simulated/semantic dispatch—not just a direct function call. Pair the normal path with a fast path: hold a navigation key, type through a refresh, press Return at a result boundary, or leave a view before a reply arrives. A first press that vanishes without explanation is a finding even if a second press works.

### Visual and language contract

Use the current [one-window contract](../../crates/sk-one-window/src/contract.rs), [chrome tokens](../../src/ui/chrome/tokens.rs), [theme](../../src/theme/) and [shared components](../../src/components/) as references. Compare siblings at the **same scale, theme, content and selection**. Use [.impeccable.md](../../.impeccable.md) for visual intent; use the fixed-height contract for window sizing. Do not retune the calibrated glass motion.

| Inspect | Pass condition |
|---|---|
| **Geometry and rhythm** | The input baseline stays put through empty → loading → populated → filtered. The 34 pt line, 28 pt header, 44 pt rows and 24 pt focused tile line up across ordinary lists. Compare left edges, section gaps, preview gutters, first/last row and the capsule’s bottom clearance. Editors, grids and prompts have justified anatomy, not fake list rows. SDK prompt height is set on open, not on every result. |
| **Typography and copy** | Compare actual rendered family, size, weight, line height and baseline by role: title to title, hint to hint, keycap to keycap. Check long filenames, accents, emoji, descenders, multi-key glyphs, code and wrapped lines. No clipped text or double dimming. Use the same verb for the same effect: **Attach Note** is not **Open Note**; **Stop** is not **Send**. Errors say what the user can do next. |
| **Visual meaning** | In one frame, point separately to keyboard focus, selected item, selected text, hover, disabled action and persistent toggle. The focused list row has its soft fill and gold tile, not an invented accent bar; an editable field’s border identifies its focus. A selection fill shows what typing replaces; colored underlines identify meaning under white words. Attached context is a chip, not an underline under invisible text. Neither color nor hover alone carries essential information. |
| **Text and pointer precision** | Click both halves of a proportional glyph; insert in the middle, after a hard return, at a soft wrap and on a scrolled line. Select backward, replace, paste and undo. Check graphemes and IME composition: Return committing a candidate must not simultaneously submit the form. The highlight and caret use the positions of the drawn text. |
| **Readability and access** | Inspect dark and light themes, supported sizes/scales, narrow available width, long labels, keyboard-only navigation and visible focus. Compare the painted label, semantic role/name, enabled state and action ID. Accessibility-tree inspection is not a substitute for a native screen-reader check; record that boundary if untested. |

Then look at the interaction at normal speed: **Before**—what draws the eye, and can you predict Return? **During**—can you tell waiting from work already done? **After**—is success, refusal or recovery visible without opening logs? A frame can meet every spacing token and still fail this review.

## 3. Sweep the product in this order

| Pass | Surface families | High-value seam to exercise |
|---|---|---|
| 1. Decide and act | Summon/hide, launcher, Jev, symbols and completions, staged subjects, learned matches, all results, actions, line, holds and undo | Type forward through a late reading; navigate deliberately; compare Return, its capsule and the resulting target. Reopen and check what was remembered. |
| 2. Choose and manipulate | Files mini/full, path picker, clipboard, favorites, apps, windows, browser tabs/history, app menus, emoji, processes | Filter to none and back. Compare similar names, two windows or files with the same title, stale indexes, offscreen selection, previews and disabled actions. Verify effects against identity, not the row number. |
| 3. Enter and edit | All SDK prompt families: arg/mini/micro, select, div, form/fields, env, path/drop, template, naming, hotkey, editor, terminal, chat and webcam; scratch pad and quick terminal | Compare Tab/Return in single-line and multiline fields, submit versus newline, password masking, cancellation and the script’s actual response. Exercise non-list prompts via their real entry, not only a mounted fixture. |
| 4. Read, create and converse | Notes/day pages, links/history, Brain capture, model chat, attachments, answers, agents, Flow Desk/sessions, dictation and transcript history | Follow a note into chat and back. Try delayed first token, partial and complete Markdown, long tool output, approval/refusal, stop, failure and retry. Preserve the draft and keep the active decision visible. Distinguish app Flow features from repository automation. |
| 5. Configure, learn and recover | Profiles, presets, themes, settings, global shortcuts, permission/setup views, connections, kits, templates, migration, script issues, About, Tips, SDK reference, run history | Test first use, unavailable dependency, revoked permission, conflict, cancel, persistence after reopen and recovery. A setup or help row must lead somewhere useful. |
| 6. Cross the app boundary | Tray, global hotkeys, deeplinks, SDK/stdin/MCP routes, external activation/paste, native footer/glass and snap overlay | Compare initial context and destination with the equivalent launcher route. Separate the app’s dispatch from what macOS or another app actually did. Include actions with no dedicated view. |

Every inventoried feature gets a happy path and its likeliest misleading or unsafe state. Deepen coverage where ownership, renderer, lifecycle or consequence differs; don’t blindly multiply every theme × key × command. An exception needs an owner and rationale, not a local pixel override.

## 4. Run the fix loop

1. **First batch: invalid form → correction → successful script response.** Open a sandbox script that asks for an email and multiline notes. Submit an invalid email while notes has focus. Can you tell which field needs fixing, where typing will land, and that nothing was sent? Correct it, Tab back, insert a newline and submit with the advertised chord. In a fresh run, click Submit instead. Both must deliver the same two values exactly once; Esc in a third run must deliver cancellation, not the draft. Compare field labels, validation borders, caret, error clearance and focus return with SDK Fields. This builds on the caret repair rather than retesting only `a\nok`.
2. **Keep a small reference set.** Use the [form walkthroughs](native-form-foundation.md) and the `p3-form-edit`, `p3-chat-markdown` and `p3-staged-browser-tab` [chapters](../../scripts/agentic/one-window/chapters/) as controls, not proof for unrelated features. Use matched frames for layout, a short sequence for motion/async changes, and `getState`/`getElements` plus an observable effect for behavior.
3. **Triage by consequence.** Fix wrong-target execution, data loss or a misleading irreversible action first. Next: dead keys, focus traps, broken recovery and obscured decisions. Then align spacing, type and copy. A recurring 2 pt drift belongs in the shared renderer or token; a genuine editor exception belongs in the editor. Don’t solve either with ten local patches.
4. **Prove the change.** Save the failing interaction, change the smallest responsible owner, rerun the same interaction on the new build, then test a neighboring consumer and a negative case. Keep a durable behavior or pixel regression when it distinguishes the bug. Don’t turn a plausible screenshot or source-string match into a pass.
5. **Close each batch visibly.** Report features checked / in scope, failures fixed and still open, intentional exceptions, blocked native checks, and a few before/after pairs. Count *outcomes*, not frames. Completion means all reachable rows are accounted for, no unresolved wrong-target/data-loss case is signed off, and every remaining gap is named.

**Execution boundary:** Default to owned-hidden evaluation on an identified build, sandbox data, provider-free responses and the existing DevTools. Use the agent Cargo wrapper for Rust checks. Reuse evidence where source-current; keep bulk captures on the SSD and clean up owned sessions. The existing Chrome check proves dispatch, not a real tab switch; Copy → Keep Both → Undo still needs the macOS Trash integration check. Keep those open until a separately authorized native session. This plan does not authorize desktop control, live providers, publishing, or replacing the running app.
