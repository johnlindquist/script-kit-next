# Expert Bundle 41: Render Key Handler Consolidation

## Goal
Consolidate duplicated keyboard event handling code across render files into shared routing helpers.

## Current State

The render layer has **6 files** with nearly identical key handling blocks for the actions dialog, plus 29 locations with duplicated arrow key matching:
- `src/render_script_list.rs` (lines 519-615)
- `src/render_prompts/arg.rs` (lines 136-196)
- `src/render_prompts/form.rs` (lines 63-125)
- `src/render_prompts/div.rs` (lines 42-100)
- `src/render_prompts/editor.rs` (lines 58-120)
- `src/render_prompts/term.rs` (lines 57-115)

Each file has an ~80-line block handling actions dialog navigation that is copy-pasted.

## Specific Concerns

1. **Actions Dialog Handler (6 copies x 80 lines)**: Identical handling of up/down/enter/escape for the actions popup, duplicated in every render file.

2. **Arrow Key Variants (29 locations)**: Some match `"up" | "arrowup"`, others only `"up"` - inconsistent and platform-dependent.

3. **Key Event Extraction**: Each handler extracts `event.keystroke.key.to_lowercase()` independently; could be a helper.

4. **Modifier Key Checking**: Platform modifier checks (`modifiers.platform`) scattered throughout.

5. **Return Early Pattern**: Each handler manually returns after consuming a key; could be abstracted.

## Key Questions

1. Should `route_key_to_actions_dialog()` be a method on `ScriptListApp` or a standalone function?

2. Is a `KeyEventContext` helper struct useful for normalizing key + modifiers + consumed state?

3. Should we create a `KeyRouter` trait that prompts/components implement for consistent handling?

4. How do we handle prompt-specific key bindings that should be checked *after* actions dialog routing?

5. Should key handlers return `bool` (consumed) or use `event.stop_propagation()` pattern?

## Duplicated Code Pattern

```rust
// This 80-line block appears in 6 files
if this.show_actions_popup {
    if let Some(ref dialog) = this.actions_dialog {
        match key_str.as_str() {
            "up" | "arrowup" => {
                dialog.update(cx, |d, cx| d.move_up(cx));
                return;
            }
            "down" | "arrowdown" => {
                dialog.update(cx, |d, cx| d.move_down(cx));
                return;
            }
            "enter" => {
                let action_id = dialog.read(cx).get_selected_action_id();
                if let Some(id) = action_id {
                    this.execute_action(&id, window, cx);
                }
                return;
            }
            "escape" | "esc" | "backspace" => {
                this.show_actions_popup = false;
                cx.notify();
                return;
            }
            _ => {}
        }
    }
}
```

## Implementation Checklist

- [ ] Create `ScriptListApp::route_key_to_actions_dialog(&mut self, key: &str, window, cx) -> bool`
- [ ] Create `src/ui/keys.rs` with `is_up()`, `is_down()`, `is_enter()`, `is_escape()` helpers
- [ ] Update `render_script_list.rs` to use routing helper
- [ ] Update `render_prompts/arg.rs` to use routing helper
- [ ] Update `render_prompts/form.rs` to use routing helper
- [ ] Update `render_prompts/div.rs` to use routing helper
- [ ] Update `render_prompts/editor.rs` to use routing helper
- [ ] Update `render_prompts/term.rs` to use routing helper
- [ ] Standardize all arrow key matching to use `is_up()`/`is_down()` helpers
- [ ] Consider `KeyRouter` trait for future extensibility
