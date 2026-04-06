# Agentic Typing Issues

Date: 2026-04-06
Surface: ACP chat `@` mention picker and attachment portals

## What failed during autonomous verification

- `setAcpInput("@bro")` reliably opened the ACP mention picker and logged
  `acp_mention_picker_refreshed`, so the input mutation path itself is fine.
- The stdin `simulateKey` path for `AppView::AcpChatView` does **not** route
  Enter/arrow keys through ACP's real picker handling. In practice:
  - `simulateKey enter` submits the composer instead of accepting the current
    mention-picker row.
  - `simulateKey` also lacks ACP-specific arrow-key navigation, so the agent
    cannot move from `Browser URL` down to `Browse Files…` through the same
    test path.
- The `window.ts` agentic screenshot helper currently looks for the packaged
  app name `Script Kit`, but the local dev binary shows up to `System Events`
  as `script-kit-gpui`. That made `window.ts status/list/capture` miss the
  live window during local verification.
- The resolver-backed stdin `captureWindow` fallback also failed in this
  session with `Main automation window not registered`, so screenshot capture
  through the normal agentic path was not dependable once ACP was open.

## What worked

- Real macOS key events sent through `System Events` did drive the ACP picker.
- Using actual Down + Enter produced:
  - `acp_picker_item_accepted ... item_label=Browse Files…`
  - `attachment_portal_opened kind=FileSearch`
  - `RESIZE_SIZE_END ... height=500 width=750`
- AX window bounds from `System Events` matched the expanded portal size:
  - ACP before portal: `480x440`
  - File-search portal open: `750x500`

## Practical takeaway

For ACP mention-picker verification, the current autonomous stack is missing a
true picker-aware key path. The reliable options today are:

1. Add ACP-aware Enter/arrow routing to stdin `simulateKey` for
   `AppView::AcpChatView`, or
2. Teach the agentic window helpers to target the local dev process name
   `script-kit-gpui` and keep window capture/state receipts working there.

Without one of those fixes, autonomous verification of `@`-picker flows is
fragile and tends to diverge from the real user key path.
