# Research: Clipboard OCR Action (Vision OCR + Clipboard History)

## 1) Files investigated
- `src/ocr.rs`
  - OCR module implemented with macOS Vision (`VNRecognizeTextRequest`) and CoreGraphics image conversion.
- `src/lib.rs`
  - OCR module behind the `ocr` Cargo feature gate.
- `src/actions/builders.rs`
  - Adds the `clipboard_ocr` action for image entries in clipboard history.
- `src/clipboard_history/blob_store.rs`
  - File-based blob storage for clipboard images (`blob:<hash>` → PNG on disk).
- `src/clipboard_history/image.rs`
  - Encodes clipboard images to blob format; decodes blob/PNG/RGBA formats.
- `src/clipboard_history/monitor.rs`
  - Clipboard image capture path that stores images as blobs and saves DB entries.
- `src/clipboard_history/database.rs`
  - SQLite schema includes `ocr_text` column + migration + `update_ocr_text` helper.
- `src/clipboard_history/db_worker/mod.rs`
  - DB worker schema includes `ocr_text` column and update request routing.
- `src/clipboard_history/db_worker/db_impl.rs`
  - DB worker implementation updates `ocr_text` via SQL update.
- `src/clipboard_history/types.rs`
  - `ClipboardEntry` + `ClipboardEntryMeta` include `ocr_text` fields.

## 2) Existing OCR module (Vision framework)
- `src/ocr.rs` already implements OCR using Apple Vision:
  - Builds a `CGImage` from RGBA bytes, then uses Objective‑C FFI to create `VNImageRequestHandler`.
  - Creates `VNRecognizeTextRequest`, sets recognition level to **accurate** and enables language correction.
  - Aggregates `topCandidates(1)` results into a newline‑joined string.
  - Offers both sync (`extract_text_from_rgba`) and background‑thread async (`extract_text_async`) variants.
- **Feature gate**: The module is only compiled when the `ocr` feature is enabled. (`src/lib.rs`)

## 3) Clipboard OCR action stub
- `src/actions/builders.rs` defines `clipboard_ocr`:
  - Added only for `ContentType::Image` entries.
  - Label: “Copy Text from Image”, description: “Extract text from image using OCR”.
  - Shortcut: `⇧⌘C`.
- Tests in the same file assert that `clipboard_ocr` exists for image entries and is not present for text entries.
- **No other references** to `clipboard_ocr` exist in the repo, so it is currently a **stub only**.

## 4) How clipboard images are stored as blobs
- **Blob storage abstraction**: `src/clipboard_history/blob_store.rs`
  - Stores PNG bytes at `~/.scriptkit/clipboard/blobs/<hash>.png`.
  - DB content stores a reference string: `"blob:<hash>"`.
  - `store_blob()` writes content‑addressed PNG files; `load_blob()` reads them.
- **Image encoding**: `src/clipboard_history/image.rs`
  - `encode_image_as_blob()` converts `arboard::ImageData` → PNG bytes → `blob:<hash>` reference.
  - This is the preferred format (no base64 overhead, less SQLite WAL churn).
  - `decode_to_render_image()` supports `blob:`, `png:`, and legacy `rgba:` formats.
- **Clipboard monitor**: `src/clipboard_history/monitor.rs`
  - When an image is detected, it is encoded with `encode_image_as_blob()` and inserted into history.
  - The `content` column in SQLite stores the `blob:<hash>` string (not raw PNG bytes).

## 5) `ocr_text` field in the database
- SQLite schema includes `ocr_text TEXT`:
  - `src/clipboard_history/database.rs`: creates column + migration if missing.
  - `src/clipboard_history/db_worker/mod.rs`: schema string also includes `ocr_text` + migration helper.
- `update_ocr_text(id, text)` exists in both DB layers:
  - `src/clipboard_history/database.rs` (direct connection) updates and refreshes cache.
  - `src/clipboard_history/db_worker/db_impl.rs` (worker thread) also updates the column.
- Types include OCR fields:
  - `ClipboardEntry.ocr_text` and `ClipboardEntryMeta.ocr_text` in `src/clipboard_history/types.rs`.

## 6) Gap analysis: `clipboard_ocr` is defined but not wired
- `clipboard_ocr` is created in the action builder but is **never handled** by any action dispatcher.
- There are **no call sites** for `ActionsDialog::with_clipboard_entry(...)` and no `clipboard_ocr` branch in action handlers.
- Result: users can see an OCR action in the UI (if the actions dialog is shown), but selecting it would **not run OCR** or copy anything.

## 7) Implementation work needed

### A) Wire the action into the action system
1. **Open the action dialog for clipboard entries** (if not already):
   - In clipboard history UI (likely `src/render_builtins.rs` or `src/app_impl.rs` key handler), add a shortcut (e.g., Cmd+K) to open `ActionsDialog::with_clipboard_entry(...)`.
2. **Route clipboard actions to a clipboard-specific handler**:
   - `handle_action()` in `src/app_actions.rs` currently resets to `ScriptList` and only handles global actions.
   - Add a clipboard‑aware execution path (e.g., `execute_clipboard_action(action_id, entry_id, cx)`) so OCR can run while in the clipboard view.

### B) Add OCR execution for `clipboard_ocr`
1. **Fetch the image content** for the selected entry:
   - Use `clipboard_history::get_entry_by_id()` (or the DB worker equivalent) to retrieve content.
   - For images, `content` will be `blob:<hash>` (or legacy `png:` / `rgba:`).
2. **Decode image to RGBA bytes** for OCR:
   - Existing decode helpers return `RenderImage` or `arboard::ImageData` but not a direct RGBA byte buffer for OCR.
   - Add a helper (likely in `src/clipboard_history/image.rs`) to decode `blob:` / `png:` / `rgba:` into RGBA byte arrays + width/height.
3. **Invoke OCR** using `src/ocr.rs`:
   - Call `ocr::extract_text_async(width, height, rgba_bytes, callback)`.
   - Ensure the `ocr` feature is enabled in Cargo for this build path.
4. **Persist OCR results**:
   - Call `clipboard_history::update_ocr_text(entry_id, text)` (or DB worker request) to cache OCR text.
5. **Copy OCR text to clipboard**:
   - Use the existing clipboard copy helper (e.g., `pbcopy` on macOS or a clipboard API helper) to write extracted text.
6. **UI feedback**:
   - Show a HUD notification (“Copied OCR text”) and handle error cases (empty OCR, OCR failure).

### C) Optional enhancements
- If OCR is slow, show a spinner or “Running OCR…” HUD while the background thread runs.
- Cache OCR results and skip re‑OCR if `ocr_text` is already present.
- Add tests: action routing for `clipboard_ocr`, and a unit test for the new decode‑to‑RGBA helper.

---

## Verification

### Changes Made

1. **`/Users/johnlindquist/dev/script-kit-gpui/src/clipboard_history/image.rs`**
   - Added `decode_to_rgba_bytes(content: &str) -> Option<(u32, u32, Vec<u8>)>` function
   - Supports blob, png, and legacy rgba formats
   - Returns raw RGBA bytes suitable for Vision OCR (not BGRA like render functions)
   - Added unit tests for the new function

2. **`/Users/johnlindquist/dev/script-kit-gpui/src/clipboard_history/mod.rs`**
   - Exported `decode_to_rgba_bytes` from the module

3. **`/Users/johnlindquist/dev/script-kit-gpui/src/app_actions.rs`**
   - Added `clipboard_ocr` action handler after `clipboard_upload_cleanshot`
   - Handler flow:
     1. Validates entry is an image
     2. Checks for cached OCR text in `entry.ocr_text` (skip OCR if available)
     3. Gets entry content from database
     4. Decodes to RGBA bytes using new `decode_to_rgba_bytes()`
     5. Calls `script_kit_gpui::ocr::extract_text_from_rgba()`
     6. Caches result via `clipboard_history::update_ocr_text()`
     7. Copies extracted text to clipboard
     8. Shows HUD notification with result

### Test Results

- `cargo check`: PASS
- `cargo clippy --all-targets -- -D warnings`: PASS
- `cargo test --lib --bins`: 2441 passed, 1 failed (pre-existing failure unrelated to OCR)

### Implementation Notes

- The OCR runs synchronously in the action handler
- Vision framework handles threading internally
- Results are cached in SQLite for future lookups
- Falls back to "No text found" message if OCR returns empty
- Error handling shows failure message in HUD

### Known Limitations

- Synchronous call may briefly block UI for large images
- No progress indicator during OCR (shows "Extracting text..." HUD)
- Could be improved with GPUI async integration in future
