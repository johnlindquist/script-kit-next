# GPUI Cookbook

Practical, copy-first patterns for building and extending Script Kit GPUI safely.

## Entries

- [`keyboard-handling.md`](./keyboard-handling.md): Keyboard event wiring, key normalization, and matching both key variants.
- [`theme-and-focus.md`](./theme-and-focus.md): Theme color usage, focus-aware rendering, and avoiding hardcoded colors.
- [`uniform-list.md`](./uniform-list.md): Scrollable lists with fixed-height rows, scroll handles, and selection behavior.
- [`prompt-wrapper-pattern.md`](./prompt-wrapper-pattern.md): Wrapper-versus-inner prompt architecture for reusable rendering shells.
- [`stdin-protocol-add-command.md`](./stdin-protocol-add-command.md): Adding new JSONL protocol commands end-to-end.
- [`cx-notify-when-and-where.md`](./cx-notify-when-and-where.md): When and where to call `cx.notify()` so UI updates render correctly.

## How to use this cookbook

Before writing GPUI code, find the relevant cookbook entry, locate the canonical file it references, and copy the pattern. Never invent GPUI APIs.
