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
