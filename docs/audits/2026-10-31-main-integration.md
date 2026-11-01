# October 2026 integration verification

Internal engineering record. This is a development integration, not a signed release or an installation on the operator’s Mac.

## Source and delivery

- Starting local `main`: `e456a2dc2`. Development was on `gpui-upgrade` at `f5e34fc68`; the two descendant dictation commits ending at `b319c5a21` were incorporated without replacing working files. A before/after tree comparison accounted for all 330 initially changed or untracked paths.
- The release-contract run used `65d7a892e`. The follow-up through `4f96c5228` repaired binary-test ownership and outdated expectations, clarified clean-chat behavior in the installed guide, and finished the illustrated changelog. No squashing, rebasing, remote push, app installation, or deletion of old branches, stashes, or other worktrees.
- [Illustrated October changelog](https://merry-flint-85wx.here.now/) · [offline edition](../changelog/2026-10/index.html). All 14 public files were downloaded and compared byte-for-byte with the checked-in edition. Its six app screenshots are lossless device-pixel renders with sample data, not captures of a private desktop. The keyboard lightbox, focus return and narrow-screen navigation were exercised on the published page. The three linked browser examples use sample data, not live access to a Mac.

## Automated results

| Check | Result |
| --- | --- |
| App library | 14,685 passed, 0 failed; 46 ignored tests were not enabled |
| Main-window Enter, keyboard and view rules | 103 passed against the integrated binary test target |
| One-window reducer, properties and replay | 259 passed |
| Release integration and domain suites | 55 integration tests and 186 clipboard/protocol/storage tests passed |
| SDK | ES2022 type and capability fixtures passed; 261 behavioral fixtures passed |
| Other checks | Strict library/binary Clippy, Cargo check, formatting, surface/catalog consistency, first-run, permissions, mock-AI and privacy fixtures passed |
| Additional targeted coverage | 246 changed Bun checks and 14 browser-source audit cases passed |

These totals describe their own suites; do not add repeated focused checks to the library total. The application built from `4f96c5228` had verified manifest SHA-256 `bd1b05d5b331cf120421c377ad6398349139d444799a42ff5a54eb27687b28d7`. A fresh hidden subject-selection, action and note-readback run against that executable passed and closed its runtime. The full frame set below was captured before the later harness-only and documentation follow-ups; it is not a second set of captures from the final binary.

## What the checks exercise

- An isolated HTTP MCP server and temporary Notes store exercised scoped grants and cold readback after server restart. Hidden GPUI runs exercised subject pick/stage/dispatch, draft editing, cancellable sends, SDK keyboard/button submission and cancellation. Neither harness needed an operator credential, native input or live inference.
- Seven owned-renderer chapters covered cursor cancellation and restoration, destination switching, preview/cancel/apply for a fixture filename, opt-in intent and unavailable services, chat Markdown, webcam countdown, and a template editor. All 53 frames met their geometry/interaction expectations. These chapters exercise the application with seeded inputs; they do not exercise a physical microphone, camera or a real third-party Save dialog.
- The regenerated matrix includes the subject-tab fixture from compiled discovery. The source census is 34 kinds and 49 mappings; historical descriptors remain distinguishable from runnable surfaces. The eight retired Confirm-window design conflicts were verified against the exporter history, not reintroduced to match a stale count.

## Source-bound release checks

The first full proof run found stale surface counts, retired popup owners, outdated built-in labels and a standalone build-script test that could not link the new native build dependency. The checks now bind to the actual renderer, routes and registered operations. An unavailable connector never gets credited as a successful send.

The shared local test registry also contained 2,741 missing records and 11 untrusted linked records. Its cleanup guard refused to delete synthetic artifacts, as designed. No registry entry or shared cache was removed to make a test pass. The 53 configured Bun proof suites were run in two disjoint sets at `65d7a892e`: 13 suites in a clean same-commit SSD checkout with private synthetic fixtures (766 passed), and 40 in the canonical checkout (1,069 passed). The 60-second Bun case timeout accommodated SSD fixture I/O; application deadlines, snapshot/identity checks and storage limits were unchanged. This is complete suite-file coverage, not one green monolithic invocation in the shared registry.

A supplementary browser-source audit stopped when the managed storage inventory timed out during a build. After the competing SSD work ended, fresh admission measured the complete tree within limits. The one-worker rerun passed all 14 cases with the same guard. Neither storage location nor its minimum free-space reserve was changed.

## Limits

Physical hotkeys, macOS permission prompts, microphone/camera hardware, OCR on a personal window, actual focus changes in another app/browser, real provider credentials and signed/notarized installation were not exercised. No global input, screen takeover, personal clipboard capture or paid model call was used to qualify this integration. No driver sessions were left in `/tmp/sk-driver-sessions`. The small local release log and owned runtime receipts were retained for diagnosis; unrelated operator records were not cleaned up.
