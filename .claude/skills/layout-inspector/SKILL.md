---
name: layout-inspector
description: Layout debugging for GPUI windows. Use when diagnosing layout bugs — missing elements, clipped content, flex sizing issues, overflow problems. Covers debug div probing, layout dump JSON protocol, computed bounds assertions, and the agentic fix-verify loop.
---

# Layout Inspector

Debug GPUI layout issues by combining visual probes, computed layout dumps, and automated invariant checks. Use this when elements disappear, get clipped, or have wrong sizes.

## Quick Diagnosis Flow

When an element is missing or not rendering:

1. **Add a debug probe** — bright-colored div to test if the container is working
2. **Build and capture** — screenshot to check if probe is visible
3. **Read the screenshot** — visually confirm what rendered
4. **Interpret results** — probe visible = container works, element is the problem; probe invisible = container/parent is broken

## Debug Probe Pattern

Add a brightly colored div near the suspected problem area:

```rust
// Add AFTER the suspected missing element
.child(div().h(px(50.)).w_full().bg(gpui::red()))

// Add BEFORE to test if parent is rendering children at all
.child(div().h(px(50.)).w_full().bg(gpui::blue()))
```

**Remove debug probes after diagnosis.** They are not meant to be committed.

### Interpreting Probe Results

| Probe visible? | Element visible? | Diagnosis |
|---------------|-----------------|-----------|
| Yes | No | Element itself has 0 height, display:none, or is transparent |
| No | No | Parent container is broken — flex sizing, overflow clip, or display:none |
| Yes | Yes | Element is fine — problem was elsewhere |
| No | Yes | Impossible — re-check test setup |

## Common Flex Layout Failures

### "Content eats the column" (most common)

In a flex column with `overflow_hidden()`:
- Content area refuses to shrink (min-height auto / min-content semantics)
- It expands to fill the column
- Fixed-height elements below get pushed past the bottom
- `overflow_hidden()` clips them — they "disappear"

**Fix:** Ensure the expanding content area has `.min_h_0()` or `.min_h(px(0.))` so it can shrink below its content size:

```rust
// CORRECT: content can shrink, fixed footer stays visible
div().flex().flex_col().h_full().overflow_hidden()
    .child(
        div().flex_1().min_h_0().overflow_hidden()  // content area shrinks
            .child(scrollable_content)
    )
    .child(fixed_footer)  // always visible at bottom
```

### Canonical "scrollable + fixed" pattern

From `render_impl.rs:385`:
```rust
.child(div().flex_1().w_full().min_h(px(0.)).child(main_content))
```

Key: `flex_1()` + `min_h(px(0.))` + parent has `overflow_hidden()`.

### Common mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Missing `min_h_0()` on flex child | Child never shrinks, pushes siblings out | Add `.min_h_0()` or `.min_h(px(0.))` |
| Missing `overflow_hidden()` on parent | Content overflows parent bounds, no clipping | Add `.overflow_hidden()` |
| Adding `flex()` + `flex_col()` to wrapper that only needs `flex_1()` | Wrapper becomes a new flex context, children collapse | Use only `.flex_1()` for size-taking wrappers |
| `h_full()` on nested flex child | Can force parent to grow, pushing siblings out | Use `.flex_1()` instead of `.h_full()` for flexible sizing |

## Layout Dump System (stdin JSONL protocol)

> **Status:** This is the design spec. Implementation requires patching the GPUI crate's TaffyLayoutEngine.

### Triggering a dump

Send via stdin:
```json
{"type":"debug.layout_dump","window":"ai","path":"./target/layout_ai.json"}
```

Optional fields:
- `ids`: array of element ID strings to focus on in `by_id` output
- `include_tree`: boolean (default true) — full tree traversal
- `include_by_id`: boolean (default true) — flat index by element ID
- `pretty`: boolean (default false) — pretty-print JSON

### Acknowledgment

Stdout JSONL response:
```json
{"type":"debug.layout_dumped","window":"ai","path":"./target/layout_ai.json","ok":true}
```

### Dump JSON Schema (v1)

```json
{
  "schema_version": 1,
  "window": "ai",
  "scale_factor": 2.0,
  "tree": {
    "id": "root",
    "node": 1,
    "bounds": { "x": 0, "y": 0, "w": 1200, "h": 800 },
    "children": [
      {
        "id": "ai-main-panel",
        "node": 42,
        "bounds": { "x": 0, "y": 52, "w": 1200, "h": 748 },
        "children": []
      }
    ]
  },
  "by_id": {
    "ai-input-area": [
      { "node": 99, "bounds": { "x": 0, "y": 700, "w": 1200, "h": 48 } }
    ]
  }
}
```

**Node ID format:**
- Elements with `.id("name")` → `"name"`
- Unlabeled nodes → `"@node:123"` (Taffy node ID)

### Agent Assertions

Minimum invariants for the AI window input area:

```
by_id["ai-input-area"][0].bounds.h > 0          # not collapsed
by_id["ai-input-area"][0].bounds.w > 0          # not zero-width
ai-input-area.y + h <= ai-main-panel.y + main_h # not clipped
ai-composer.h == COMPOSER_H (36px)               # correct height
```

### Failure Signatures

| Signature | Condition | Likely Causes |
|-----------|-----------|---------------|
| **Collapsed** | `target.h == 0` | Style size/min_size regression, display:none, flex basis/shrink change |
| **Clipped** | `target.h > 0` but `target.y` outside ancestor bounds | min-height auto prevents flex shrink, overflow hidden + wrong min_h_0, flex column sizing bug |
| **Overflowed** | `target.y + target.h > parent.y + parent.h` and parent has overflow:hidden | Content area not shrinking, missing min_h_0 on sibling |

## Implementation Hook Points (for patching GPUI)

### Where identity mapping goes

During layout node creation, register `LayoutId -> DebugNodeInfo` in debug builds:

```rust
#[cfg(debug_assertions)]
pub struct DebugNodeInfo {
    pub element_id: Option<String>,     // from .id("ai-input-area")
    pub source_file: Option<&'static str>,
    pub source_line: Option<u32>,
    pub style: DebugStyleInfo,          // overflow, display, flex_grow, etc.
}
```

### Where dump happens

**After** `TaffyLayoutEngine::compute_layout()` — layout is not computed during `Render`, so dumping from render is wrong.

Sequence:
1. stdin thread parses JSON → enqueues `LayoutDumpRequest`
2. Window render loop processes request → calls `window.request_layout_dump(spec)`
3. Next frame, after Taffy compute_layout, GPUI writes dump atomically
4. GPUI emits stdout JSONL ack

### Atomic file writing

```rust
#[cfg(debug_assertions)]
fn write_atomic_json(path: &Path, value: &impl Serialize) -> io::Result<()> {
    let tmp = path.with_extension("tmp");
    let mut f = fs::File::create(&tmp)?;
    f.write_all(&serde_json::to_vec(value).expect("serialize"))?;
    f.write_all(b"\n")?;
    f.sync_all()?;
    fs::rename(tmp, path)?;
    Ok(())
}
```

## Agentic Fix-Verify Loop

### Without layout dump (current state)

1. **Probe:** Add debug colored div near suspected issue
2. **Build:** `cargo build`
3. **Capture:** Use named-pipe test script (see `visual-test` skill)
4. **Read:** Read the PNG screenshot to visually verify
5. **Diagnose:** Interpret probe visibility per table above
6. **Fix:** Apply the fix based on diagnosis
7. **Verify:** Rebuild, recapture, confirm fix
8. **Clean:** Remove debug probes

### With layout dump (when implemented)

1. **Build:** `cargo build`
2. **Start app:** Named pipe pattern from `visual-test` skill
3. **Show window:** `echo '{"type":"show"}' >&3`
4. **Trigger dump:** `echo '{"type":"debug.layout_dump","window":"ai","path":"./target/layout_ai.json"}' >&3`
5. **Wait for ack:** Check stdout log for `debug.layout_dumped`
6. **Read dump:** Parse JSON, run invariant assertions
7. **Diagnose:** Match failure signature (collapsed vs clipped)
8. **Fix:** Apply targeted fix based on computed bounds data
9. **Verify:** Retrigger dump + screenshot, confirm all invariants pass

## Key Element IDs (AI Window)

| Element ID | Description | Expected |
|-----------|-------------|----------|
| `ai-main-panel` | Root flex column for content + input | h > 0, fills window minus titlebar |
| `ai-input-area` | Bottom input area with composer + controls | h > 0, visible at bottom |
| `ai-composer` | Text input row with plus button + input field | h == COMPOSER_H (36px) |
| `attachments-btn` | Plus button for image attachments | size == S6 |
| `submit-btn` / `stop-btn` | Submit message / Stop streaming | visible in bottom-right |
| `ai-actions-btn` | Actions button (Cmd+K) | visible in bottom-right |

## Window Layout Constants

| Constant | Value | Location |
|----------|-------|----------|
| `COMPOSER_H` | `px(36.)` | `src/ai/window/types.rs` |
| `TITLEBAR_H` | `px(48.)` | `src/ai/window/types.rs` |
| `SIDEBAR_W` | `px(220.)` | `src/ai/window/types.rs` |
| `PANEL_INSET_X` | `S4` | `src/ai/window/types.rs` |

## Related Skills

- **visual-test** — Named-pipe test scripts, captureWindow, screenshot workflow
- **gpui-patterns** — Layout chains, flex patterns, theme system
- **dev-loop** — Background dev server, log monitoring
