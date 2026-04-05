# Verification Recipes

Named patterns agents select based on what they changed.

---

### Recipe: verify-main-menu

**When:** Changes to script list rendering, footer, search, or built-in entries.

**Steps:**
1. Build: `cargo build`
2. Start session: `bash scripts/agentic/session.sh start default`
3. Show: `bash scripts/agentic/session.sh send default '{"type":"show"}'`
4. Capture screenshot of main menu
5. Read PNG — verify: "Script Kit" header, list items, footer shows "Run ⌘K Actions Tab AI"
6. Set filter: send `{"type":"setFilter","text":"clip"}` → capture → verify filtered results
7. Clear filter: send `{"type":"setFilter","text":""}` → verify list restores

**Pass:** Main menu renders with correct items, filter narrows list, footer intact.
**Fail:** Missing items, wrong footer, filter doesn't work. Check render_impl.rs and render_script_list.

---

### Recipe: verify-acp-chat-open

**When:** Changes to ACP chat view, Tab AI entry, context bootstrap.

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
1. `cargo check && cargo clippy --lib -- -D warnings`
2. `cargo nextest run --lib` (with 30s timeout)
3. Build, start session, show, capture main menu
4. Read PNG → verify basic rendering intact

**Pass:** All gates pass, main menu renders.
**Fail:** Compilation error, clippy warning, test failure, or visual regression.

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

**Steps:**

```bash
# 1. Start session
bash scripts/agentic/session.sh start default
sleep 3

# 2. Show window
bash scripts/agentic/session.sh send default '{"type":"show"}'
sleep 1.5

# 3. Open ACP
bash scripts/agentic/session.sh send default '{"type":"triggerBuiltin","name":"tab-ai"}'
sleep 5

# 4. Verify ACP ready (state receipt BEFORE screenshot)
bun scripts/agentic/verify-shot.ts --session default \
  --label acp-ready \
  --acp-context-ready

# 5. Focus window for native input
bun scripts/agentic/window.ts focus
sleep 0.3

# 6. Type @ to open picker (NATIVE input, not simulateKey)
bun scripts/agentic/macos-input.ts type "@"
sleep 1

# 7. Verify picker opened
bun scripts/agentic/verify-shot.ts --session default \
  --label picker-open \
  --acp-picker-open

# 8. Accept with native Enter (or Tab)
bun scripts/agentic/macos-input.ts key enter
sleep 0.5

# 9. Verify accepted: picker closed + item recorded + cursor moved
bun scripts/agentic/verify-shot.ts --session default \
  --label item-accepted \
  --acp-picker-closed \
  --acp-item-accepted

# 10. Check telemetry logs
grep -i "acp_picker_item_accepted\|acp_picker_tab_accept\|picker.*accept\|route.*picker" \
  /tmp/sk-agentic-sessions/default/app.log | tail -5

# 11. Cleanup
bash scripts/agentic/session.sh stop default
```

**Or use the orchestrator (after Phase 1 is proven stable):**
```bash
bun scripts/agentic/index.ts acp-enter-accept --session default
```

**Critical invariants:**
- `getAcpState` **must** be queried before screenshot capture at every verification point
- The test MUST FAIL if `getAcpState` says picker is still open, even if the screenshot looks correct
- Native macOS input (`macos-input.ts`) is required for picker acceptance testing — `simulateKey` bypasses native key routing
- Window focus must be verified before sending native input
- Telemetry logs must show `acp_picker_item_accepted`, `acp_picker_tab_accept`, or equivalent route confirmation

**Pass:** All verify-shot assertions pass, telemetry confirms picker acceptance, cursor lands after inserted text.
**Fail:** State receipt contradicts expected outcome. Common causes:
- Picker still open after Enter → Enter routed to composer submit instead of picker accept
- No `lastAcceptedItem` → picker was never open when Enter was pressed
- Cursor at wrong index → text insertion logic off
- No telemetry → ACP key routing not instrumented

---

### Recipe: verify-acp-picker-tab-vs-enter

**When:** Specifically testing that both Tab and Enter accept picker rows correctly.

**Steps:**
1. Run `verify-acp-golden-path` with Enter (steps above)
2. Re-open ACP, re-trigger picker with `@`
3. Accept with native Tab instead of Enter
4. Verify same outcome: picker closed, item accepted, cursor correct
5. Compare cursor positions — both should produce identical caret placement

**Pass:** Both Tab and Enter produce the same accepted state and cursor position.
**Fail:** One key accepts but the other doesn't. Check ACP key_down handler for picker-open branching.

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

**Pass:** Cursor moves correctly, layout metrics track, no visual jump between captures.
**Fail:** `cursorInWindow` doesn't match expected position, or `visibleStart` jumps unexpectedly. Check ACP input scroll/viewport logic.
