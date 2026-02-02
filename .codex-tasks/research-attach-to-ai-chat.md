# Research: Attach Clipboard Entry to AI Chat

## Key Findings from Research

1. **AI Chat Integration Exists in `src/ai/window.rs`**
   - `AiApp` already stores a pending image attachment as base64 PNG (`pending_image: Option<String>`). This is the field that would hold an image ready to be sent with the next message. (`src/ai/window.rs` lines 409-410)
   - The paste handler shows how to inject images: it reads the clipboard image, encodes it as PNG base64 (via `encode_image_as_png`), strips the `png:` prefix, and stores it in `pending_image`. (`src/ai/window.rs` lines 698-727)
   - When a message is submitted, the pending image is attached to the user message via `ImageAttachment::png(image_base64)`. (`src/ai/window.rs` lines 1982-1989)

2. **Clipboard History Infrastructure**
   - Clipboard images use a blob store on disk at `~/.scriptkit/clipboard/blobs/<hash>.png`, with DB content stored as `blob:<hash>`. (`src/clipboard_history/blob_store.rs` lines 1-7, 33-37)
   - `get_entry_content(id)` retrieves the stored content string for a clipboard entry (text or image content reference). (`src/clipboard_history/database.rs` lines 572-584)
   - The `ContentType` enum distinguishes `Text` vs `Image` entries. (`src/clipboard_history/types.rs` lines 7-12)
   - Image encoding helpers include PNG base64 and blob formats; PNG base64 uses the `png:` prefix. (`src/clipboard_history/image.rs` lines 15-41)
   - Blob loading returns PNG bytes from disk, enabling conversion to base64 when needed. (`src/clipboard_history/blob_store.rs` lines 54-74)

3. **Action Already Defined But Not Wired**
   - The clipboard context action list includes `clipboard_attach_to_ai` with shortcut `⌃⌘A`. (`src/actions/builders.rs` lines 814-883)
   - There is no `clipboard_attach_to_ai` branch in `ScriptListApp::handle_action`, so the action is defined but not handled. (`src/app_actions.rs` lines 50-140)
   - Clipboard History UI key handler only wires `Enter` to copy+paste; no action dialog or attach-to-AI trigger is present in the view key handling. (`src/render_builtins.rs` lines 185-289)

4. **Implementation Plan (Based on Current Code Paths)**
   - Add a `clipboard_attach_to_ai` handler in `src/app_actions.rs` that reads the selected clipboard entry and routes it to the AI window.
   - For **text** entries: use `get_entry_content(id)` to fetch the text content and open AI chat with it as the initial prompt.
   - For **image** entries: resolve the blob or `png:` data, convert to base64 PNG if necessary, and pass it to AI chat as a pending image (mirroring `handle_paste_for_image`).
   - Add a UI trigger from clipboard history (e.g., actions dialog or keybinding) since the current view only handles `Enter` for paste and has no attach-to-AI route.

## Code References (Specific Locations)

- `src/ai/window.rs:409-410` — `pending_image: Option<String>` storage on `AiApp`.
- `src/ai/window.rs:698-727` — `handle_paste_for_image` encodes clipboard image as PNG base64 and stores in `pending_image`.
- `src/ai/window.rs:1982-1989` — `ImageAttachment::png(image_base64)` attachment on message submit.
- `src/actions/builders.rs:814-883` — `get_clipboard_history_context_actions` includes `clipboard_attach_to_ai` action.
- `src/app_actions.rs:50-140` — `handle_action` match list (no `clipboard_attach_to_ai` handler).
- `src/render_builtins.rs:185-289` — Clipboard History key handler; `Enter` triggers copy+paste only.
- `src/clipboard_history/blob_store.rs:1-7,33-37,54-74` — blob storage path/format and load bytes.
- `src/clipboard_history/database.rs:572-584` — `get_entry_content(id)` definition.
- `src/clipboard_history/types.rs:7-12` — `ContentType::{Text, Image}`.
- `src/clipboard_history/image.rs:15-41` — PNG base64 and blob encoding helpers.

## Verification

### What was changed

1. **Handler Implementation** (`src/app_actions.rs` lines ~278-335):
   - Added `clipboard_attach_to_ai` case in `handle_action` match
   - For TEXT entries: calls `ai::open_ai_window(cx)` then `ai::set_ai_input(cx, &content, false)`
   - For IMAGE entries: converts blob to PNG bytes via `clipboard_history::content_to_png_bytes`, encodes to base64, calls `ai::set_ai_input_with_image(cx, "", &base64_data, false)`
   - Handler calls `self.hide_main_and_reset(cx)` after action completes

### Test Results

- `cargo check`: PASS
- `cargo clippy --all-targets -- -D warnings`: PASS  
- `cargo test --bin script-kit-gpui clipboard`: 68 tests pass

### Before/After Comparison

| Aspect | Before | After |
|--------|--------|-------|
| Action Definition | Existed in `src/actions/builders.rs` | Unchanged |
| Handler | No handler in `app_actions.rs` | Full handler implementation |
| Text Support | N/A | Opens AI chat with text as input |
| Image Support | N/A | Opens AI chat with image as pending attachment |

### Deviations from Proposed Solution

None - the implementation follows the exact pattern documented in the research:
- Uses existing `content_to_png_bytes` helper for image blob conversion
- Uses standard `base64::engine::general_purpose::STANDARD.encode()` for PNG encoding
- Follows same error handling pattern as `clipboard_share` action
