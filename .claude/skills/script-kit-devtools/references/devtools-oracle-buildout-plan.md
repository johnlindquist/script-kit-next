# Oracle DevTools Buildout Plan

Oracle browser session `devtools-all-surfaces-buildout-plan-2` reviewed the surface inventory and confirmed the pivot: Script Kit DevTools should be protocol/MCP/CLI primitives for proving real UI behavior, while recipes remain regression wrappers only after direct red/green proof exists.

## Corrections From Oracle

The inventory is a source-backed backlog, not a coverage model. A surface should not count as covered until there is a target-scoped runtime receipt with target identity, surface contract, semantic tree, layout, scroll, text, focus, event trace, action receipts, visual proof, and fail-closed classification.

Coverage aliases must be routing hints only. `ScriptList -> main`, `ActionsDialog -> actions-dialog`, `AttachmentPortalBrowser -> dictation-history`, and `AcpChat -> notes-acp` are useful lookup hints, but they must not count as coverage because they can hide missing AppView variants, detached hosts, and portal modes.

The inventory must preserve `dismissPolicy`. Escape, Cmd-W, window blur, backdrop click, popup dismissal, and focus routing bugs cannot be investigated correctly if the generated dismiss contract is dropped.

## Shared Receipt Envelope

Every primitive should eventually return the same envelope: `schemaVersion`, `tool`, `command`, `invocationId`, `sessionId`, repo metadata, start/end times, strict target identity, preconditions, result payload, assertions, classification, warnings, errors, and redaction policy.

The core classifications should include `ok`, `reproduced`, `fixed`, `not-reproduced`, `blocked-by-missing-primitive`, `blocked-by-target-ambiguity`, `blocked-by-stale-generation`, `blocked-by-unsafe-operation`, `blocked-by-permission`, `blocked-by-real-data-risk`, `blocked-by-native-escalation-required`, `blocked-by-fixture-only`, and `blocked-by-timeout`.

## Primitive Families

The buildout should prioritize shared primitives before surface-specific wrappers: `devtools.targets.list`, `devtools.targets.inspect`, `devtools.surface.inspect`, `devtools.elements.snapshot`, `devtools.layout.measure`, `devtools.scroll.inspect`, `devtools.text.measure`, `devtools.focus.inspect`, `devtools.keyboard.inspect`, `devtools.act.*`, `devtools.visual.compare`, `devtools.media.inspect`, `devtools.events.record`, `devtools.storage.fingerprint`, and `devtools.investigate.*`.

The minimum acceptance bar for any surface is strict target identity, surface inspect with contract and runtime state, stable semantic elements, layout with overlaps and resize pressure, scroll/text/focus where applicable, at least one safe user-like action with pre/post receipts, strict nonblank visual capture, action-correlated events, red/green compare, and exported investigation artifact.

## Work Packages

The recommended package order is:

1. Schema and fail-closed receipt contracts.
2. Target identity and surface inspect.
3. Elements, layout, scroll, text, focus, keyboard, and AX parity.
4. Safe actions and keyboard routing.
5. Launcher, actions popup, main menu resizing, shortcuts, and aliases.
6. Prompt runtime and oversized container pressure.
7. Built-in filterable lists and preview surfaces.
8. Portals, resources, and context.
9. ACP/chat and composer state.
10. Notes resizing and Notes-hosted ACP.
11. Dictation, media, permissions, and target delivery.
12. Platform windowing, permissions, and visual proof.
13. Storybook, design, and theme.
14. Investigation, events, storage, security, diagnostics, and replay.

## Bug-Proofing Rules

Actions popup bugs require route stack, parent target, subject semantic id, section bounds, anchor rect, popup rect, placement, clipping edges, disabled reasons, and shortcut label layout.

Dynamic main menu resizing bugs require before/after window rects, desired content height, measured content height, row/input/footer/preview heights, max/min allowed height, clipped rows, overlap pairs, and resize cause.

Oversized prompt container bugs require `clientHeight`, `scrollHeight`, `canScrollY`, clipping, overflow policy, resize pressure score, footer/input/content overlap counts, and same-stack red/green comparison.

Notes resize bugs require active note identity, dirty state, selection range, scroll anchor, editor/preview overlap count, clipped node count, shortcut focus owner, and embedded ACP generation when ACP is visible.

Dictation bugs require passive permission status, microphone device snapshot, model readiness generation, recording state generation, audio level metrics, transcript fingerprint, target delivery generation, target identity, insertion range, wrong-target refusal when applicable, and cleanup receipt.
