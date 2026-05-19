# Verification Recipes

Named patterns agents select based on what they changed.

---

### Recipe: verify-main-menu

**When:** Changes to script list rendering, footer, search, or built-in entries.

**Fast path:** `make smoke-main-menu`

**Steps:**
1. Build: `cargo build`
2. Start session: `bash scripts/agentic/session.sh start default`
3. Show: `bash scripts/agentic/session.sh send default '{"type":"show"}'`
4. Capture screenshot of main menu
5. Read PNG — verify: "Script Kit" header, list items, footer shows "Run ⌘K Actions ACP Chat"
6. Set filter: send `{"type":"setFilter","text":"clip"}` → capture → verify filtered results
7. Clear filter: send `{"type":"setFilter","text":""}` → verify list restores

**Pass:** Main menu renders with correct items, filter narrows list, footer intact.
**Fail:** Missing items, wrong footer, filter doesn't work. Check render_impl.rs and render_script_list.

---

### Recipe: verify-acp-chat-open

**When:** Changes to ACP chat view, ACP Chat entry, context bootstrap.

**Surface rule:** Verify the real ACP chat opened through the product entry path. Do not prove ACP UI changes by instantiating `AcpChatView` in an isolated temporary window.

**Steps:**
1. Build: `cargo build`
2. Start session, show window
3. Send `{"type":"triggerBuiltin","name":"tab-ai"}` → sleep 3
4. Capture → verify "Preparing context" or "Ask Claude Code anything"
5. Sleep 5 more → capture → verify "Context attached" and "Enter to send"

**Pass:** ACP chat opens, context attaches, input ready.
**Fail:** Toast error ("Failed to start ACP connection"), empty view, stuck on "Preparing".

---

### Recipe: verify-acp-chat-send

**When:** Changes to ACP thread, message rendering, streaming, submit logic.

**Steps:**
1. Build, start session, show, send triggerBuiltin tab-ai, wait 8s for context
2. Type "hi": simulateKey h, simulateKey i
3. Capture → verify text "hi" visible in input area
4. simulateKey enter → sleep 5
5. Capture → verify: user message card ("You / hi"), streaming indicator, footer "Streaming"
6. Sleep 15 → capture → verify assistant response text rendered
7. Check logs: `grep "acp_initialized\|acp_session_created" /tmp/sk-agentic-sessions/default/app.log`

**Pass:** Message sent, user card shown, ACP initialized, response streams.
**Fail:** No ACP logs = wrong agent binary. No response text = event stream not wired. Check config.rs (agent command) and handlers.rs (event dispatch).

---

### Recipe: verify-acp-close

**When:** Changes to close semantics, view restore, Cmd+W handling.

**Steps:**
1. Open ACP chat (use verify-acp-chat-open steps)
2. simulateKey w with cmd modifier
3. Sleep 1 → capture
4. Verify main menu restored (Script Kit header, list items)

**Pass:** Cmd+W returns to main menu.
**Fail:** View stuck, wrong view restored. Check `close_tab_ai_harness_terminal` in tab_ai_mode.rs.

---

### Recipe: verify-actions-dialog

**When:** Changes to actions dialog (⌘K), action entries, dialog rendering.

**Steps:**
1. Build, start session, show
2. simulateKey k with cmd modifier → sleep 1
3. Capture → verify actions dialog rendered (action list visible)
4. simulateKey escape → sleep 0.5
5. Capture → verify actions dialog closed, main menu restored

**Pass:** ⌘K opens dialog, Escape closes it.
**Fail:** Dialog doesn't appear or doesn't close. Check toggle_actions and actions dialog code.

---

### Recipe: verify-builtin-view

**When:** Changes to clipboard history, emoji picker, app launcher, file search, etc.

**Steps:**
1. Build, start session, show
2. triggerBuiltin with the view name (clipboard, emoji, apps, file-search)
3. Sleep 2 → capture
4. Verify the correct view rendered (check header, list content, footer)
5. simulateKey escape → sleep 1 → capture → verify return to main menu

**Pass:** View opens with correct content, escape returns to main.
**Fail:** Wrong view, empty content, stuck. Check the TriggerBuiltin handler and AppView variant.

---

### Recipe: verify-keyboard-input

**When:** Changes to key handlers, SimulateKey dispatch, input fields.

**Steps:**
1. Open the target view (main menu, ACP chat, arg prompt, etc.)
2. simulateKey with test characters → capture
3. Verify characters appear in the correct input field
4. Check logs for `SimulateKey: Dispatching` to confirm dispatch reached the right view

**Pass:** Keys dispatched to correct view, text appears in input.
**Fail:** Keys hit `_ =>` fallback ("View not supported"). Add the view's match arm to SimulateKey in app_run_setup.rs.

---

### Recipe: verify-dynamic-element-selection

**When:** Changes to element introspection, batch commands, or semantic ID generation.

**Steps:**
1. Build, start session, show
2. Send `getElements` request: `{"type":"getElements","requestId":"e1"}`
3. Read logs for `elementsResult` — verify semantic IDs returned (e.g., `choice:0:...`, `input:filter`)
4. Use a returned semantic ID in a batch: `{"type":"batch","requestId":"b1","commands":[{"type":"selectBySemanticId","semanticId":"<id-from-step-3>","submit":false}]}`
5. Read logs for `batchResult` — verify `success: true` and `value` field populated
6. Capture screenshot → verify the element is now selected/focused

**Pass:** getElements returns semantic IDs, selectBySemanticId resolves and selects the correct element.
**Fail:** Empty elements list (check collect_elements.rs), selectBySemanticId returns SelectionNotFound (check semantic ID format matches).

---

### Recipe: verify-regression

**When:** Any change — run as a smoke test.

**Steps:**
1. `make smoke-main-menu`
2. If the change is not covered by the launcher smoke path, add the smallest targeted check for the touched area
3. Escalate to repo-wide test gates only when the smoke proof is inconclusive or the risk warrants it

**Pass:** Smoke verification proves the real runtime surface still works.
**Fail:** Build failure, session failure, screenshot failure, log mismatch, or obvious visual regression.

---

### Recipe: verify-session-management

**When:** Changes to session.sh, session-state.ts, or the agentic testing infrastructure itself.

**Steps:**
1. Start session: `bash scripts/agentic/session.sh start test-session`
2. Verify JSON envelope has `status: "ok"`, `resumed: false`, valid `pid`, `pipe`, `log`
3. Re-run start: `bash scripts/agentic/session.sh start test-session` → verify `resumed: true`
4. Check status: `bash scripts/agentic/session.sh status test-session` → verify `alive: true`
5. Send show: `bash scripts/agentic/session.sh send test-session '{"type":"show"}'` → verify `sent: true`
6. From a fresh shell, send another command → verify it reaches the same app process
7. Check state: `bun scripts/agentic/session-state.ts --session test-session` → verify all fields
8. Stop: `bash scripts/agentic/session.sh stop test-session` → verify `wasRunning: true`
9. Re-check status → verify `alive: false` or `not_found`

**Pass:** Session creates, resumes, sends from multiple shells, reports state, and cleans up.
**Fail:** Stale PID, broken pipe, forwarder not running. Check session.sh forwarder loop.

---

### Recipe: verify-acp-golden-path

**When:** Changes to ACP picker, context mentions, Enter/Tab acceptance, caret placement, or input layout stability.

**This is the definitive ACP interaction verification recipe.** Use it whenever ACP behavior needs proving. Future agents should default to this recipe for any ACP change.

**Preferred: Use the canonical CLI command (fully non-interactive):**
```bash
bash scripts/agentic/session.sh start default
bun scripts/agentic/index.ts acp-accept --session default --key enter --vision
# Parse the JSON receipt — proofBundle contains state, probe, screenshot, visionCrops.
# Exit code 0 + proofBundle.state confirming expected ACP fields = PASS.
# No manual PNG reading required.
bash scripts/agentic/session.sh stop default
```

**With target threading (detached/popup ACP) — resolve exact identity first:**
```bash
bash scripts/agentic/session.sh start default

# Resolve exact target and surface identity once
RESOLVED="$(bun scripts/agentic/automation-window.ts resolve --session default --kind acpDetached --index 0)"
TARGET="$(printf '%s' "$RESOLVED" | jq -c '.targetJson')"
SURFACE_ID="$(printf '%s' "$RESOLVED" | jq -r '.surfaceId')"
WINDOW_ID="$(printf '%s' "$RESOLVED" | jq -r '.automationWindowId')"

bun scripts/agentic/index.ts acp-accept --session default --key enter \
  --target-json "$TARGET" --surface "$SURFACE_ID" --vision
# Confirm proofBundle.state.resolvedTarget.windowKind == "acpDetached"
# Confirm proofBundle.captureTarget.requestedWindowId == proofBundle.captureTarget.actualWindowId
# and no target warnings in proofBundle.state.warnings
bash scripts/agentic/session.sh stop default
```

**Target threading rule:** Resolve one exact ACP target once via
`automation-window.ts resolve` and reuse both the `targetJson` and `surfaceId`
for every RPC and native input step. Never use loose family-level `--surface acp`
when multiple ACP windows may exist — always derive the exact surface identity
from the resolver output.

The `acp-accept --vision` command encodes the full golden path:
- Resets ACP test probe before native interaction
- Uses `macos-input.ts --ensure-focus` for native typing and acceptance
- State-only intermediate checkpoints (no wasted screenshots)
- Waits for `acpAcceptedViaKey` (key-specific, not generic)
- One final screenshot + probe assertion as visual proof
- Returns a `proofBundle` with `state`, `probe`, `screenshot`, `visionCrops`

**Identity invariant for detached ACP runs:** A run is invalid unless these three
identities agree in the final receipt:
- ACP state `resolvedTarget` (from `proofBundle.state.resolvedTarget`)
- native input resolved `surfaceId` (from macos-input.ts `session_focus_resolved` log)
- screenshot `captureTarget` (from `proofBundle.captureTarget.requestedWindowId == actualWindowId`)

**Surface rule:** This recipe verifies the real ACP runtime surface only. Screenshots from synthetic `AcpChatView` wrappers, debug-only windows, or component harnesses do not count.

**Manual steps (when finer control is needed):**

```bash
S="bash scripts/agentic/session.sh send default"
R="bash scripts/agentic/session.sh rpc default"

# 1. Start session
bash scripts/agentic/session.sh start default

# 2. Show window
$S '{"type":"show"}'
sleep 0.3  # macOS focus-settling delay, not ACP proof

# 3. Open ACP
$S '{"type":"triggerBuiltin","name":"tab-ai"}'

# 4. Wait for ACP ready (deterministic, replaces sleep 5)
$R '{"type":"waitFor","requestId":"w-ready","condition":{"type":"acpReady"},"timeout":8000,"pollInterval":25,"trace":"onFailure"}' \
  --expect waitForResult --timeout 10000

# 5. State-only checkpoint: no screenshot needed
bun scripts/agentic/verify-shot.ts --session default \
  --label acp-ready \
  --skip-screenshot --skip-probe \
  --acp-context-ready

# 6. Reset probe before native interaction
$S '{"type":"resetAcpTestProbe","requestId":"reset-enter-1"}'

# 7. Type @ to open picker (NATIVE input with focus enforcement)
bun scripts/agentic/macos-input.ts type "@" --ensure-focus

# 8. Wait for picker to open (deterministic, replaces sleep 1)
$R '{"type":"waitFor","requestId":"w-picker","condition":{"type":"acpPickerOpen"},"timeout":3000,"pollInterval":25,"trace":"onFailure"}' \
  --expect waitForResult --timeout 5000

# 9. State-only checkpoint: no screenshot needed
bun scripts/agentic/verify-shot.ts --session default \
  --label picker-open \
  --skip-screenshot --skip-probe \
  --acp-picker-open

# 10. Accept with native Enter (or Tab) with focus enforcement
bun scripts/agentic/macos-input.ts key enter --ensure-focus

# 11. Wait for key-specific acceptance (not generic acpItemAccepted)
$R '{"type":"waitFor","requestId":"w-accepted","condition":{"type":"acpAcceptedViaKey","key":"enter"},"timeout":3000,"pollInterval":25,"trace":"onFailure"}' \
  --expect waitForResult --timeout 5000

# 12. Final proof: screenshot + probe (the only screenshot in the recipe)
bun scripts/agentic/verify-shot.ts --session default \
  --label enter-accepted \
  --acp-picker-closed \
  --acp-item-accepted \
  --acp-accepted-via enter

# 13. Cleanup
bash scripts/agentic/session.sh stop default
bash scripts/agentic/session.sh status default
```

**Critical invariants:**
- `waitFor` conditions are the primary proof of ACP state transitions — not fixed sleeps
- `getAcpState` **must** be queried before screenshot capture at every verification point
- The test MUST FAIL if `getAcpState` says picker is still open, even if the screenshot looks correct
- Native macOS input (`macos-input.ts --ensure-focus`) is required for picker acceptance testing
- Intermediate checkpoints (ACP ready, picker open) use `--skip-screenshot --skip-probe`
- Only the final acceptance checkpoint takes a screenshot and queries the probe
- Window focus must be verified before sending native input
- Cleanup is part of the recipe: do not report completion until the started session is stopped

**Pass:** All waitFor conditions resolve, verify-shot assertions pass, cursor lands after inserted text.
**Fail:** `waitForResult` returns `success: false` with trace receipt. Common causes:
- `acpReady` timeout → ACP context bootstrap failed or took too long
- `acpPickerOpen` timeout → `@` input not received or picker not triggered
- `acpAcceptedViaKey` timeout → Enter/Tab routed to composer submit instead of picker accept
- Picker still open after Enter → Enter routed to composer submit instead of picker accept
- No `lastAcceptedItem` → picker was never open when Enter was pressed
- Cursor at wrong index → text insertion logic off

---

### Recipe: verify-acp-picker-tab-vs-enter

**When:** Specifically testing that both Tab and Enter accept picker rows correctly.

**Preferred: Use the canonical CLI for both keys:**
```bash
bun scripts/agentic/index.ts acp-accept --session default --key enter --vision
bun scripts/agentic/index.ts acp-accept --session default --key tab --vision
```

**Pass:** Both Tab and Enter produce the same accepted state and cursor position.
**Fail:** One key accepts but the other doesn't. Check ACP key_down handler for picker-open branching.

---

### Recipe: verify-acp-detached-target-threading-stress

**When:** Testing multi-window detached ACP flows where target identity can drift between native input, ACP state, probe, waitFor, and screenshot capture.

**Command:**
```bash
bun scripts/agentic/index.ts acp-detached-target-threading-stress \
  --session default \
  --kind acpDetached \
  --index 0 \
  --min-targets 2 \
  --key enter \
  --vision \
  --json
```

**Pass:** Receipt reports `status:"pass"`, `proofBundle.targetThread.stable:true`, `proofBundle.usage.usedNativeInput:true`, and `proofBundle.captureTarget.requestedWindowId == proofBundle.captureTarget.actualWindowId`.
**Fail:** Any target drift, missing `surfaceId`, missing `osWindowId`, insufficient peer windows, or capture identity mismatch.

---

### Recipe: verify-acp-prompt-popup-parity

**When:** Testing ACP PromptPopup families such as mention, model selector, and local history without relying on pixels.

**Command:**
```bash
bun scripts/agentic/index.ts acp-prompt-popup-parity \
  --session default \
  --families mention,model-selector,local-history \
  --json
```

**Pass:** Every popup case has a stable `targetThread`, expected popup id, row-aware `getElements(target)` receipt, and non-empty row count.
**Fail:** Wrong popup family/id, missing rows, generic PromptPopup ambiguity, or close/visibility wait failure.

---

### Recipe: verify-notes-acp-delayed-action-origin-stress

**When:** Testing Notes-hosted ACP delayed actions where origin/generation drift can silently retarget a later action.

**Command:**
```bash
bun scripts/agentic/index.ts notes-acp-delayed-action-origin-stress \
  --session default \
  --drift generation \
  --json
```

**Pass:** A future app-side receipt proves the delayed action dispatched to its captured Notes ACP origin, or intentionally reports `originDriftDetected` for a drift injection.
**Current fail-closed behavior:** Until Notes ACP exposes origin/generation receipts, this recipe returns a machine-readable `missingOriginGeneration` failure instead of inferring safety from unrelated state.

---

### Recipe: verify-file-portal-origin-roundtrip

**When:** Testing ACP or Notes-hosted ACP attachment portal flows where portal return must preserve the original ACP host, generation, portal session, and accepted context-part URI.

**Command:**
```bash
bun scripts/agentic/index.ts file-portal-origin-roundtrip \
  --session default \
  --origin acp \
  --portal file-search \
  --selection file \
  --query AGENTS.md \
  --json
```

**Pass:** A future app-side receipt proves origin host/generation, portal session id, return target, and accepted context-part URI all match.
**Current fail-closed behavior:** Until those receipts exist, this recipe returns `missing_portal_round_trip_origin_receipt` and does not infer safety from generic ACP state.

---

### Recipe: verify-permission-privacy-preflight

**When:** Testing permission or screenshot/native-input prerequisites without opening System Settings, prompting macOS, or mutating TCC state.

**Command:**
```bash
bun scripts/agentic/index.ts permission-privacy-preflight \
  --session default \
  --kinds accessibility,screen-recording,microphone \
  --json
```

**Pass:** Receipt reports read-only prerequisite checks and `openedSystemSettings:false`, `mutatedTcc:false`, and `clickedSettings:false`.
**Fail:** Any prerequisite check fails; the receipt must still show that the harness did not attempt remediation.

---

### Recipe: verify-shortcut-recorder-focus-capture

**When:** Testing native shortcut recorder focus/capture behavior without writing `config.ts` or registering a global hotkey.

**Command:**
```bash
bun scripts/agentic/index.ts shortcut-recorder-focus-capture \
  --session default \
  --surface shortcuts \
  --action test-agentic-shortcut \
  --chord cmd+shift+7 \
  --sandbox-config \
  --json
```

**Pass:** A future recorder receipt proves the exact recorder surface retained focus, the native chord was captured, and no unrelated global hotkey fired.
**Current fail-closed behavior:** Until recorder focus/capture receipts exist, this recipe returns `missing_shortcut_recorder_capture_receipt` and does not write user config.

---

### Recipe: verify-template-prompt-automation-parity-stress

**When:** Testing TemplatePrompt protocol parity for state, elements, field navigation, actions, submit/cancel, and ForceSubmit without using screenshots.

**Command:**
```bash
bun scripts/agentic/index.ts template-prompt-automation-parity-stress \
  --session default \
  --template 'Hello {{name}}' \
  --field name \
  --value Ada \
  --forced-value forced-template-result \
  --json
```

**Pass:** The receipt proves `getState.promptType:"template"`, `getElements` template rows, TemplatePrompt actions host, Escape cancel, and batch ForceSubmit explicit value without screenshots or native input.
**Fail:** Missing template state/elements/actions/ForceSubmit receipts, or any protocol step returning an error.

---

### Recipe: verify-current-app-commands-frontmost-stress

**When:** Testing Do in Current App aliases, frontmost-app snapshot identity, shared CurrentAppCommands filtering, and stale-app execution guards.

**Command:**
```bash
bun scripts/agentic/index.ts current-app-commands-frontmost-stress \
  --session default \
  --alias 'Do in Current Command' \
  --query 'close tab' \
  --json
```

**Pass:** A future receipt proves the stable `builtin/do-in-current-app` entry opened `CurrentAppCommandsView` from the captured frontmost app, and state/elements/renderer counts use shared filtering semantics.
**Current fail-closed behavior:** Until those receipts exist, this recipe returns `missing_current_app_commands_frontmost_receipt` and never executes a menu action against a guessed app.

---

### Recipe: verify-actions-captured-subject-frame-stress

**When:** Testing root unified Cmd+K actions where execution must use the subject captured when the dialog opened, even after filter, selection, cache, or source-frame drift.

**Command:**
```bash
bun scripts/agentic/index.ts actions-captured-subject-frame-stress \
  --session default \
  --source root-file \
  --action quick-look \
  --mutation filter-selection-cache-frame \
  --json
```

**Pass:** A future receipt proves the MainList actions dialog retained the captured subject stable key, executed against that key, and restored focus without re-reading the current selection.
**Current fail-closed behavior:** Until those receipts exist, this recipe returns `missing_actions_captured_subject_receipt`.

---

### Recipe: verify-drop-prompt-native-drop-privacy-stress

**When:** Testing native file-drop behavior where automation receipts must expose file count/name/size without leaking full paths or file contents.

**Command:**
```bash
bun scripts/agentic/index.ts drop-prompt-native-drop-privacy-stress \
  --session default \
  --file-name agentic-drop.txt \
  --size 12 \
  --json
```

**Pass:** A future native-drop injection receipt proves `stateResult.drop` and `getElements` dropped-file rows are redacted to index/name/size.
**Current fail-closed behavior:** Until native file-drop injection is deterministic from scripts/agentic, this recipe returns `missing_drop_prompt_native_drop_receipt`.

---

### Recipe: verify-path-prompt-filesystem-edge-stress

**When:** Testing PathPrompt missing, empty, file-start, and permission-denied filesystem edge receipts.

**Command:**
```bash
bun scripts/agentic/index.ts path-prompt-filesystem-edge-stress \
  --session default \
  --json
```

**Pass:** The tracked `scripts/agentic/path-prompt-fs-edges.ts` helper proves `stateResult.path` and `getElements` status rows match the expected edge kind for every fixture case.
**Fail:** The helper exits non-zero or any state/elements status kind diverges.

---

### Recipe: verify-screenshot-identity-acp-context-stress

**When:** Testing screenshot identity threading from Tab AI capture through `stateResult.screenshotIdentity` and ACP context attachment.

**Command:**
```bash
bun scripts/agentic/index.ts screenshot-identity-acp-context-stress \
  --session default \
  --source tab-ai-screenshot \
  --json
```

**Pass:** A future receipt proves capture identity, state identity, and ACP context part identity match without filesystem greps.
**Current fail-closed behavior:** Until ACP context exposes that identity bridge, this recipe returns `missing_screenshot_identity_context_receipt`.

---

### Recipe: verify-clipboard-history-portal-range-stress

**When:** Testing Clipboard History portal host refusal, `kit://clipboard-history?id=...` round-trip, and exact range replacement on accept.

**Command:**
```bash
bun scripts/agentic/index.ts clipboard-history-portal-range-stress \
  --session default \
  --portal-id 'kit://clipboard-history?id=agentic' \
  --range composer:0..0 \
  --json
```

**Current fail-closed behavior:** Until clipboard portal receipts expose host refusal, round-trip URI, and exact replacement range, this recipe returns `missing_clipboard_portal_range_receipt`.

---

### Recipe: verify-browser-tabs-cache-identity-stress

**When:** Testing browser tabs/history cache identity, dedupe keys, and stale-cache rejection without activating the browser.

**Command:**
```bash
bun scripts/agentic/index.ts browser-tabs-cache-identity-stress \
  --session default \
  --source browser-tabs \
  --json
```

**Current fail-closed behavior:** Until browser cache receipts expose source identity, dedupe key, cache-only status, and browser-activation guard, this recipe returns `missing_browser_cache_identity_receipt`.

---

### Recipe: verify-scroll-selection-reanchor-stress

**When:** Testing cross-surface wheel/drag/native-scroll selection reanchor behavior against visible rows and footer-safe viewports.

**Command:**
```bash
bun scripts/agentic/index.ts scroll-selection-reanchor-stress \
  --session default \
  --kinds clipboard,browser-history,current-app-commands,file-search \
  --json
```

**Current fail-closed behavior:** Until surfaces expose one state-first reanchor receipt across wheel/drag transitions, this recipe returns `missing_scroll_selection_reanchor_receipt`.

---

### Recipe: verify-acp-input-stability

**When:** Changes to single-line input rendering, cursor movement, or scroll behavior in ACP.

**Steps:**
1. Open ACP, wait for ready
2. Type a long string (30+ characters) via native input
3. Query `getAcpState` → check `inputLayout` metrics: `charCount`, `visibleStart`, `visibleEnd`, `cursorInWindow`
4. Capture screenshot
5. Move cursor with native left-arrow 5 times
6. Query `getAcpState` again → verify `cursorIndex` decreased by 5, `visibleStart`/`visibleEnd` adjusted if needed
7. Capture second screenshot
8. Compare: the input container should not have shifted height or layout

**Canonical input-stability assertion (no screenshot needed):**
```bash
bun scripts/agentic/verify-shot.ts --session default \
  --label input-stability \
  --skip-screenshot \
  --acp-visible-start 12 \
  --acp-visible-end 52 \
  --acp-cursor-in-window 39
```

This verifies cursor position within the visible text window and catches scroll jumps,
layout shifts, and cursor-out-of-view regressions without requiring a screenshot.

**Pass:** Cursor moves correctly, layout metrics track, no visual jump between captures.
**Fail:** `cursorInWindow` doesn't match expected position, or `visibleStart` jumps unexpectedly. Check ACP input scroll/viewport logic.
