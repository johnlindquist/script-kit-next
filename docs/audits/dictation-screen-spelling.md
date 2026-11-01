# Where screen-aware dictation gets its spellings

**Finding:** both apps use what is in front of the speaker, but only Raycast's traced dictation path reads pixels. The useful latency trick is to start gathering potential spellings while the speaker is talking. Reading the screen for the first time on key-up is much more expensive than applying a spelling that is already known.

## Installed builds, not just product descriptions

Static inspection of the arm64 code shipped in Raycast **2.10.1.0** and Superwhisper **3.1.0**. Neither application was launched, no user database was opened, and no screen, clipboard, microphone or Accessibility data was captured. REA 6.5.0 `inspect-macho` established executable identity and imports. REA `function` with Ghidra 12.1.4 analyzed two bounded function bodies at their original addresses; Apple LLVM's disassembly of the entire executable resolved their external callees and string references. Addresses below are unslid. This establishes client behavior, not what a remote model does, actual competitor latency, or the behavior of every mode.

### Raycast: take a picture, but send spelling clues to the rewriter

- In the shipped `backend/index.mjs`, `vLa` checks Screen Recording permission, then calls `raycast.captureDictationContext`. At session start the call runs as a promise alongside `startDictation`; its result belongs to that session. At stop, `DLa`/`jLa` build `userPrompt` from the available context. The accept path awaits style lookup, **not** pending screen capture. A missing or late capture needn't delay dictation.
- The native handler name is at `0x10024de04`. The path branches at `0x10024df78` to the focused-window capturer. Its code references `/usr/sbin/screencapture`, a temporary PNG and `FocusedWindowCapturer.swift`. At `0x10024e000` the dictation caller passes `1` and branches through `0x1007d7eb8` to `0x1007d8470`. That function selects Vision's **accurate**, not fast, recognition level for `1`, enables automatic language detection and language correction, and performs recognition on a URL. This is a dictation call path, not an inference from an OCR framework also used for PDFs.
- On success, `0x10024e0a0` calls the entity extractor at `0x1007ad334`; its name tagger calls `NLTagger.enumerateTags` at `0x1007ad7d0`. Backend formatter `eha` labels the results **App, Names, Organizations, Places, URLs, Emails, Phone, Dates, Terms** and stops before exceeding its 1,500-character budget. `dha` includes that text in the cleanup prompt. The prompt instructs the model to use app context to resolve spelling or recognition errors. No image is an argument to that stop call. This is separate from the optional caret-aware paste rule, which adjusts leading space and first-letter case.
- [Raycast Dictation manual](https://manual.raycast.com/ai/dictation#app-context). The manual describes the feature and permission. The capture, OCR setting and timing above come from the installed build.

### Superwhisper: read the UI tree and supply names to the mode

- `AxManager.swift` observes frontmost-app/window changes. Its code references the frontmost window UI tree (`0x1000182f0`); the tree builder reads AX attributes (`0x100025e7c`) and references `AXChildrenInNavigationOrder` (`0x1000261e4`). The name-collection path uses Apple's Natural Language tagger (`0x1000188c4`). This is text supplied by the app's accessibility interface, not text recognized in an image.
- The mode's context formatter at `0x100a78a50` conditionally assembles the current app, URL, focused element, **“Focused element content”** (`0x100a78d08`) and **“Names and Usernames”** (`0x100a78e9c`). This is context for processing a transcript, not a global autocorrect dictionary. No Vision text-recognition call was found in the inspected main executable's dictation path. That is not a claim about every helper or future version.
- [Superwhisper's context guide](https://superwhisper.com/docs/common-issues/context) distinguishes timing: selection at record start; application context after voice transcription, before AI processing. Its [mode guide](https://superwhisper.com/docs/modes/built-in#super) distinguishes context-aware Super Mode from transcription-only Voice to Text. A focused-window observer does not prove that all text extraction happens on key-down.

## How much time is actually needed?

Local, offscreen fixture on an Apple M4 Max, macOS 26.5.1. A 1440 × 900 bitmap rendered an issue thread, names, source paths and mixed-case identifiers in 22 px type. Each Vision mode ran in a fresh process: one first request, then 30 warm requests with small position and build-number changes. The timer covers Vision and reading the recognized strings; it excludes drawing the fixture, acquiring a real window, audio transcription and paste.

| Vision recognition level | First request | Warm median | Warm p95 | Exact spelling over 31 images |
|---|---:|---:|---:|---|
| Fast | 213 ms | 31 ms | 36 ms | `OpenAI` 31/31; `TypeScript` and `macOS` 0/31 |
| Accurate | 344 ms | 122 ms | 131 ms | `TypeScript` and `macOS` 31/31; `OpenAI` 0/31 (`OpenAl`, final lowercase L) |

These are not competitor timings. Even accurate OCR can be a worse source of spelling than the accessibility text. Script Kit's clipboard OCR adapter currently passes level `1` despite its “accurate” comment; the Apple SDK declares **accurate = 0, fast = 1**. Do not silently change clipboard behavior when adding dictation.

**From recognized pixels to corrected text:** taking observed words with internal capitals from the fast OCR result and feeding them into the unmodified `src/dictation/polish.rs::apply_vocabulary` changed “quick look” and “open ai” to `QuickLook` and `OpenAI`. The accurate result fixed “type script” and “mac os”, but left “open ai” alone instead of substituting the OCR error `OpenAl`. This simple mixed-case filter deliberately missed `Raycast`; a real term picker should use titles and named entities too.

With the observed vocabulary padded to 64 candidates, 2,000 optimized Rust calls per condition gave **0.013 ms p95 for 22 transcript words**, and **0.058–0.061 ms p95 for 176 words**. Those numbers include building the matcher's keys and rewriting the text, not extracting candidate vocabulary. This was a standalone run of the exact matcher source, not an installed dictation session. It handles case, spacing and hyphenation across up to three words, not sound-alikes such as *Shawn* and *Sean*.

## Recommendation for Script Kit

1. Start with an **opt-in, local, per-recording vocabulary**. At hotkey-down, bind a background read to the original app/window and recording generation. Prefer bounded visible AX text near the cursor and names in window labels. Never wait for it to start the microphone. Skip secure fields, excluded apps and uncertain window identities; no clipboard or permission prompt during dictation.
2. Use OCR only as an optional fallback for apps with little accessible text, such as a canvas. Capture only the pinned window, with existing Screen Recording permission. The native-ID and PID check in `src/platform/screenshots_window_open.rs` is a useful starting point; the generic AI screenshot helper can fall back to the entire screen and is **not** appropriate. `src/ocr.rs` implements Vision for RGBA images; `src/platform/visual_context.rs` still has placeholder capture and OCR functions.
3. Keep a capped list of distinctive terms in memory, not a screenshot or a transcript of the window. Explicit user vocabulary wins. At delivery, take only a **ready** result for the same recording/window; otherwise deliver normally. Discard on cancel, target change or completion. Do not append the screen vocabulary to the cloud cleanup prompt, logs or history. The existing polish path can call an optional model; that is **not** required to apply a spelling locally.
4. Start with unambiguous normalized matches. For genuinely different spellings, evaluate recognition hints in local engines that accept them; don't introduce automatic fuzzy substitution of ordinary words. Before enabling by default, measure **added key-up-to-delivery p95** against the same pipeline with context disabled (proposed budget: **under 5 ms**); test very short utterances, unavailable AX, denied capture, cancellation, a same-app window switch and ambiguous names. The offscreen results establish feasibility, not that product-level target.

## Script Kit implementation benchmark

Accessibility text is on by default; window OCR is opt-in. Change the mode in [Dictation Setup](../guides/dictation.md#spell-names-from-the-window). `dictation.screenSpelling` accepts `off`, `accessibility`, or `accessibilityAndOcr`; `bun scripts/config-cli.ts get dictation.screenSpelling` and `set dictation.screenSpelling <mode>` read or change it. The direct external cursor route freezes the original field, PID, window ID and recording generation. Kit commands, Markdown review, retries and chat do not collect window vocabulary. A busy worker, missing permission, stale target or late result does not block delivery. At most 48 terms are considered. Transcripts longer than 8 KiB skip the screen pass rather than processing just part of the text.

On the M4 Max, unoptimized Cargo test profile, 250 finalizations per case with 48 candidate terms:

| Transcript | Initial correction p95 | Prepared vocabulary p95 |
|---|---:|---:|
| 111 bytes | 0.131 ms | 0.031 ms |
| 895 bytes | 0.580 ms | 0.159 ms |
| 8,192 bytes | 5.321 ms | 1.652 ms |

Pending or unavailable context cost at most **0.0003 ms p95** in these samples. The timer surrounds the production session's readiness check, rewrite and drop, **not** microphone stop, model inference, journal or macOS paste. The collector runs independently. On a 1200 × 660 offscreen thread fixture, actual Apple Vision OCR plus term collection took 1.57 s on its first call and 233 ms p95 over 30 warm calls; the pipeline corrected “type script” and “quick look” from those pixels. Acquisition of a live window was not timed. The AX branch was exercised with a supplied excerpt, not someone’s desktop.

Reproduce without an open window, microphone, network model or Screen Recording permission:

```sh
SK_SCREEN_SPELLING_BENCH=1 ./scripts/agentic/agent-cargo.sh test --lib -- \
  screen_spelling dictation::polish::tests exact_record_and_version_survive_new_connections --nocapture
```

For an operator-driven comparison, use the same audio model, destination and utterance, alternating off and on. After each successful paste, read `getState.oneWindow.cursorDictation.screenSpelling`: `lastDelivery` reports `status`, `source`, `changes` and correction time in `micros` (null if no screen pass ran). `stopToVerifiedDeliveryMs` includes capture stop, transcription, any enabled cleanup and paste verification, including with spelling off. Neither field includes screen text. The offscreen sample is not an end-to-end hotkey latency measurement.

### Evidence identity

| Subject | SHA-256 | REA `function` body (arm64) |
|---|---|---|
| Raycast executable | `e2b79f7a2ec06ec7789ebf9d4ac1caf6de2be84fb632f610b5869dcaf334f2d8` | `0x1007d8470`, 396 bytes; `aed8f70f0ed7afda9e29a3802a4ac650c158655231b5011fc5248c86e316f163` |
| Raycast backend JS | `a8d537eb132bdad3bc7386c666a0cb1941729289144ed95f61dbfecf6b9a6294` | `vLa` / `eha` / `dha` / `DLa` / `jLa` |
| Superwhisper universal executable | `c07ac1bd29dca3c437f1918f6ed59400b910f66f6183baa7e130ad27432d1cf7` | `0x100a78a50`, 1,208 bytes; `fe61c6d50c908925a49caffc959fc77340edcc4c37a2e55cd7085931162d3955` |
