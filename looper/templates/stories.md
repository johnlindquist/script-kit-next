# AFK Audit Stories — <run id>

Format: `- [<marker>] <slug>: <one-line verifiable behavior>`

Slugs are used in commit subjects' user-story lines. Keep them short, lowercase, hyphenated. Each description must be a falsifiable behavior — something the loop can verify with a state receipt or screenshot.

## Status markers

- `[ ]` open — next tick may pick it
- `[x]` closed (pass or fix-committed)
- `[!]` blocked (needs external dep or tool gap; inline comment names the dep)
- `[?]` skipped this run (known-flaky or out-of-scope; revisit later)
- `[-]` cancelled (no longer relevant)

## Story pick order

The tick prompt (Step 5) picks top-down first `[ ]` with surface satisfying Step 4's rotation constraint.

**Priority overrides** (applied before top-down):

1. **`tool-*` slugs** — auto-promoted by `promote-tool-gaps.sh` drain first. These unlock other stories.
2. **Deferred items** from the scope file's "defer and log" policy — pick when their precondition clears.
3. **Attacker passes** (every 4th) — pick an attacker-suitable surface regardless of top-down order.

## Seed stories

Adapt these to your app. Drop anything that doesn't apply. Keep each story to ONE verifiable behavior.

- [ ] main-menu-filter: Type query in main menu, see expected item rank top.
- [ ] emoji-picker: triggerBuiltin emoji, type "heart", see heart emoji filtered.
- [ ] app-launcher: triggerBuiltin apps, type known prefix, see app highlighted.
- [ ] file-search-render: triggerBuiltin file-search, type path segment, list renders without crash.
- [ ] clipboard-history: triggerBuiltin clipboard, see recent items render.
- [ ] acp-open: triggerBuiltin tab-ai, ACP view reaches acpReady within 8s.
- [ ] slash-picker: In ACP, native-type "/", slash picker opens within 500ms.
- [ ] at-picker: In ACP, native-type "@", context picker opens.
- [ ] actions-dialog: On a main-menu selection, Cmd+K, actions dialog popup resolves.
- [ ] global-hotkey: Trigger hotkey → window shows → escape → window hides.

## Generating new stories (when seeds drain)

Source new stories from, in order of priority:

1. **Tool-gap queue** — `promote-tool-gaps.sh` appends `tool-<slug>` stories below a `### Tool-gap backlog (promoted from log)` header. Drain before seed stories.
2. **Recent commits** — `git log --oneline -20`; for each commit that touched behavior (not just formatting), write a story that verifies the commit's observable effect.
3. **Untested surfaces** — cross-reference the story list against the app's built-in / view / route inventory. Surfaces with zero stories deserve one.
4. **Attacker mode** — adversarial categories from looper/rules/attacker-mode.md: invalid input, rapid-fire events, drag-interrupt, window-resize under load, focus-steal races, empty-state rendering, max-length input.

Append generated stories to this file in the SAME commit that marks them `[x]` (if the pass closed them) or in a preceding `looper:` commit (if just filing).

## Surface tagging (optional but recommended)

Prefix the slug with the surface, e.g.:

```
- [ ] menu-calc-filter: …
- [ ] acp-slash-script-authoring: …
- [ ] stdin-protocol-system-control-verbs: …
```

This makes surface breadth and rotation trivially greppable.

---

### Tool-gap backlog (promoted from log)

(populated by `promote-tool-gaps.sh` — don't hand-edit; the script is idempotent)
