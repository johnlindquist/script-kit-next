# Research: Webcam Built-in UX

## 1) Summary of existing prompt patterns

- **Arg prompt** (`src/prompts/mod.rs` + `docs/ux/PROMPT_TYPES.md`)
  - Fully implemented selection prompt with virtualized list (`uniform_list`), search input, and keyboard navigation (Enter submit, Escape cancel, arrow keys).
  - Uses `focus_handle` and shared prompt focus patterns; footer hints and header indicators mention `↑↓` navigation, `↵` submit, `Esc` cancel (see `docs/ux/PROMPT_TYPES.md`).

- **Div prompt** (`src/prompts/div.rs`)
  - Displays informational text; currently strips HTML tags and does not support Tailwind rendering. 
  - Uses Enter/Escape to submit/cancel; basic `PromptBase` styles.

- **Editor prompt** (`src/editor.rs`)
  - Full text editor, syntax highlighting, cursor navigation, clipboard support, undo/redo.
  - Uses design variant context for theme and `focus_handle`.

- **Term prompt** (`src/term_prompt.rs`)
  - Terminal emulator with PTY support and command submission via Enter.

- **Chat prompt** (`src/prompts/chat.rs`)
  - Protocol only; no UI currently.

- **Webcam/mic** (`src/protocol/message.rs`)
  - Protocol definition exists but no UI implementation yet.

## 2) Command flow (invocation → completion)

1. Script calls **webcam** API (SDK currently throws error; see `scripts/kit-sdk.ts` around line ~5560).
2. SDK sends JSONL request:
   ```json
   {"type":"webcam", "id":"webcam-1"}
   ```
   (`src/protocol/message.rs` defines `Message::Webcam { id }`)
3. App receives request and should open webcam prompt UI (not yet implemented).
4. User captures image (button, keyboard, or countdown).
5. App sends back `submit` message (`Message::Submit`), e.g.:
   ```json
   {"type":"submit","id":"webcam-1","value":"data:image/jpeg;base64,..."}
   ```
   (`docs/PROTOCOL.md` examples).
6. SDK/API returns data to script as chosen output mode (base64, file path, or clipboard).

## 3) UI layout (webcam preview + controls placement)

- Use existing `PromptBase` and design context (`src/prompts/base.rs`) for theme and design variant.
- Suggested layout (adapted from other prompts):
  - **Header**: Title bar showing "Camera" + status text ("Press Enter to capture").
  - **Preview panel**: Large centered video preview (≈70% of view height) with aspect ratio 16:9 or matching source.
  - **Controls row** below preview: 
    - left side: Capture button (primary), Cancel (`Esc`), and switch camera.
    - right side: Mirror toggle, countdown picker, output mode selector.
  - **Footer**: Keyboard hints (Enter capture, Esc cancel, Space switch, `m` mirror toggle, `c` capture, `[`/`]` change camera) similar to Arg prompt footer style.

## 4) Keyboard shortcuts (all actions)

- `Enter` / `Space` — Capture image and submit.
- `Esc` — Cancel/close.
- `m` — Toggle mirror.
- `c` — Capture.
- `s` — Cycle cameras (e.g., front/back).
- `+` / `-` — Adjust countdown timer.
- `[` `]` — Camera switching (optional).
- `tab` — Move focus between controls (if focusable controls are present).

## 5) Control options (capture, camera switch, mirror, countdown)

- **Capture**: Button and keyboard shortcut.
- **Camera switch**: Cycle between devices if multiple cameras available.
- **Mirror**: Toggle horizontal mirroring of preview and capture.
- **Countdown**: Optional countdown before capture (0, 3, 5, 10 seconds).

## 6) Output modes (clipboard, file, base64)

- **Base64**: Data URL (`data:image/jpeg;base64,...`) as protocol response (`docs/PROTOCOL.md`).
- **Clipboard**: Use built-in clipboard write; return `submit` with `value` as `data:image/jpeg;base64,...` and then trigger clipboard write via existing clipboard APIs (to avoid large IPC size).
- **File**: Save to temporary file (e.g., `~/Library/Containers/...`), return file path as `value`.

## 7) Script API design

- SDK currently throws error (`scripts/kit-sdk.ts`:
  `webcam() is not implemented in Script Kit GPUI`), but proposed design:

  ```ts
  interface WebcamOptions {
    output?: 'base64' | 'file' | 'clipboard';
    camera?: 'front' | 'back' | string;
    mirror?: boolean;
    countdown?: number;
  }

  async function webcam(options?: WebcamOptions): Promise<string>;
  ```

- Should emit `webcam` request and resolve output based on mode:
  - `base64` return Data URL
  - `file` return file path
  - `clipboard` return file path + write to clipboard

## 8) Integration with existing patterns

- Use `PromptBase` for shared theme colors and `DesignContext` (`src/prompts/base.rs`).
- Respect `theme` and design variant (e.g., `DesignVariant` and `DesignContext::new`).
- Follow focus management and keyboard conventions (e.g., `focus_handle` and `focus` patterns) as in Arg/Div prompts.
- Use `focus_coordinator` pattern for opening/closing prompts and updating logs as per AGENTS.md.
- Use `Message::Submit` as output, in same pattern as other prompts (`src/protocol/message.rs`).
- Align UI feedback with `docs/ux/KEYBOARD_NAVIGATION.md` (arrow keys and `Esc`/`Enter` semantics).

## 9) UI states and ASCII mockups

### 9.1 Idle state (camera active)

```
+---------------------------------------------------------------------+
|  Camera Prompt                                   [X]  [_]
|  ─────────────────────────────────────────────────────────────────── |
|  [ Preview: live feed 16:9, border-radius, soft shadow ]            |
|                                                                     |
|  [Capture]   [Cancel]   [Switch Camera]   [Mirror: On/Off]            |
+---------------------------------------------------------------------+
|  PromptFooter: Enter=Capture, Esc=Cancel, M=Mirror, C=Capture, S=Switch |
+---------------------------------------------------------------------+
```

### 9.2 Countdown state

```
+---------------------------------------------------------------------+
|  Camera Prompt                                   [X]  [ _ ]
|  ─────────────────────────────────────────────────────────────────── |
|  [ Preview: live feed, dimmed by 20%, dark overlay ]                  |
|                  ⏱ 3 ... 2 ... 1 ... 0 (overlay)                  |
|  [Capture] [Cancel] [Switch Camera] [Mirror]                          |
+---------------------------------------------------------------------+
|  PromptFooter: Enter=Capture now, Esc=Cancel, Space=Pause countdown      |
+---------------------------------------------------------------------+
```

### 9.3 Captured state

```
+---------------------------------------------------------------------+
|  Camera Prompt                                   [X]  [_]
|  ─────────────────────────────────────────────────────────────────── |
|  [ Captured image preview ]  [Retake] [Use] [Copy to Clipboard]       |
|  (thumbnail strip)                                                  |
+---------------------------------------------------------------------+
|  PromptFooter: Enter=Use, R=Retake, Esc=Cancel, C=Copy               |
+---------------------------------------------------------------------+
```

## 10) Interaction flow and state transitions

**State machine** (simplified):

1. `Idle` → (Capture/C keyboard) → `Countdown` (if countdown > 0)
2. `Countdown` → `Capturing` (after zero) → `Captured`
3. `Idle` → (Capture immediate) → `Captured` (if countdown = 0)
4. `Captured` → (Use/Enter) → `Submit` (send Message::Submit)
5. `Captured` → (Retake) → `Idle`
6. Any state → (Cancel/Esc) → `Cancelled`
7. Any state → (Error) → `Error` (No camera / Permission)

**Detailed transitions**:

- `Idle` → `Countdown`: show overlay with countdown, lock controls except Cancel.
- `Countdown` → `Capturing`: trigger camera snapshot, capture still frame.
- `Capturing` → `Captured`: freeze preview to captured frame, show actions.
- `Captured` → `Submit`: return data and close prompt; optional post effects.
- `Error` → `Idle` (after Retry) or `Cancelled` (after Dismiss).

## 11) Error handling (UI guidance)

- **No camera detected**: display overlay text "No camera found"; disable capture button; provide "Open system preferences" and "Retry".
- **Permission denied**: prompt banner "Camera access blocked" with button to open system preferences; offer "Retry" and "Cancel".
- **Camera in use**: display "Camera busy" with wait indicator and optional "Retry".
- **Capture failure**: toast/inline error and return to `Idle` after user acknowledges.

## 12) Preview quality and frame rate

- Default preview size 1280×720 (or native) with aspect ratio 16:9.
- Target 30fps; fallback to 24fps on low-power devices.
- Use `request_video_capture` with `AVCaptureSessionPreset1280x720`/`HD720` when available; use lower quality if CPU/GPU performance drops.
- If frame drop detected, lower to 480p and notify in logs (and optional UI badge "Low quality").

## 13) Image format options

- **JPEG** (default): compressed, good for small size (quality 0.85).
- **PNG**: lossless for screenshots; larger size, optional "High quality" mode.
- **WebP** if supported by backend; prefer JPEG unless "lossless" selected.
- Output values: base64 Data URL, file path, or clipboard as in `Message::Submit`.

## 14) Accessibility

- Ensure all controls use ARIA labels and are keyboard navigable.
- Add `aria-live` region for countdown state and error status.
- Use high contrast mode colors and large text in footer hints.
- Ensure `Tab` and arrow navigation support for buttons and option toggles.
- Screen reader support: announce "Capture" / "Countdown 3" / "Image saved".

## 15) Animation and transitions timing

- Fade-in preview and overlay: 150ms ease-out.
- Countdown numbers: 500ms per second with subtle scale-in (ease-in-out).
- Capture -> Captured: 200ms cross-fade from live feed to frozen frame.
- Error banners: 250ms slide-down with 300ms pause before dismiss or action.

## 16) Visual polish details

- Border-radius on preview: 12px, subtle shadow (0 6px 20px rgba(0,0,0,0.2)).
- Preview frame: 1px soft border with theme color (`theme.colors.ui.border`).
- Buttons: primary button shadow + hover glow; secondary subdued and icon accents.
- Use icon glyphs for Capture, Switch Camera, and Retry (from icon set like `camera`, `camera-off`, `refresh`).
- Use subtle background gradient in footer and panel for depth.

## 17) Sample Rust struct outline for `WebcamPrompt`

```rust
pub struct WebcamPrompt {
    pub id: PromptId,
    pub title: SharedString,
    pub state: WebcamState,
    pub devices: Vec<CameraDevice>,
    pub selected_device: usize,
    pub mirror: bool,
    pub countdown: u8,
    pub output: WebcamOutput,
    pub captured_image: Option<CapturedImage>,
    pub focus_handle: FocusHandle,
    pub footer: PromptFooter,
    pub error: Option<WebcamError>,
}

pub enum WebcamState { Idle, Countdown, Capturing, Captured, Error, Cancelled }
pub enum WebcamOutput { Base64, File, Clipboard }
pub struct CapturedImage { path: Option<PathBuf>, data_url: Option<SharedString>, mime: ImageFormat }
pub struct CameraDevice { id: String, name: String, facing: CameraFacing }
```

## 18) Integration with `PromptFooter`

- Use `PromptFooter` to display current keyboard shortcuts for each state.
- Example:
  - `Idle`: `Enter/C` capture, `Esc` cancel, `M` mirror, `S` switch, `C` countdown.
  - `Countdown`: `Esc` cancel, `Space` pause/resume.
  - `Captured`: `Enter` use, `R` retake, `C` copy, `Esc` cancel.
- Footer should update based on state to keep context clear and avoid clutter.

## 19) Implementation Roadmap and Planning

- **Phase 1: Basic capture**
  - Build webcam UI prompt shell with live preview, capture button, and cancel.
  - Implement single-frame capture and submit via `Message::Submit` (base64 default).
  - Initial camera selection and error handling for permissions/absence.
- **Phase 2: Advanced controls**
  - Add mirror toggle, countdown, switchable output modes (base64/file/clipboard).
  - Support multiple camera devices, cycle with UI control and keyboard.
  - Add captured image preview, retake/use buttons, and footer hints per state.
- **Phase 3: Polish**
  - Refine preview aspect ratio, transitions, and hover/focus states.
  - Add loading and error banners, and ensure focus/coalescing for UI responsiveness.

- **Platform-specific considerations**
  - **macOS**: Use AVFoundation (`AVCaptureSession`, `AVCaptureDeviceInput`), handle camera permissions via `AVCaptureDevice` authorization.
  - **Linux**: Use V4L2 (`/dev/video*`), support device probing and format negotiation.
  - **Windows**: Use MediaFoundation (`IMFMediaSource`), ensure COM init and frame capture support.

- **Testing strategy**
  - Use mock camera in dev mode (pre-recorded frame source) for unit-level tests.
  - End-to-end smoke tests with `tests/smoke/` using `captureScreenshot()` and fixture images.
  - Validate UI states via screenshot verification (PNG comparison) and logs in JSONL.

- **Performance benchmarks**
  - Target 30fps preview in idle; drop to 24fps/15fps based on load.
  - Measure frame latency and memory usage (video buffer + frame cache) for 1080p capture.

- **Security considerations**
  - Request camera permissions explicitly and handle denial or revocation.
  - Ensure captured data stays local, avoid storing images unless output mode requires.
  - Sanitize file paths and clear clipboard/temporary file after use.

- **Future enhancements**
  - Burst mode (multi-shot capture and picker).
  - Video recording (short clip) with output encoding.
  - Filters and effects (grayscale, blur, timestamp overlay).
