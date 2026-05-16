# 033 Sharing and Clipboard Trust Install

Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

## Feature Scope

Map Script Kit GPUI sharing and clipboard trust install behavior:

- Shareable launcher results: scripts, scriptlets/snippets, skills, and agents.
- `copy_deeplink` as a portable share action for shareable results and fallback `scriptkit://run/...` deeplink behavior for non-shareable results.
- `scriptkit-share://v1/<payload>` URI encoding, decoding, wrapping, and clipboard export.
- Recent-export suppression so copying your own share link does not immediately prompt local import.
- Clipboard watcher behavior, change detection, polling interval, duplicate-prompt suppression, and decode failure behavior.
- Parent confirmation dialog/trust prompt before install.
- Install destination under `~/.scriptkit/plugins/<plugin-id>/`, plugin id normalization, unique sibling plugin ids, manifest writing, file writing, path traversal/top-level directory restrictions, and refresh behavior.
- User-visible success/failure feedback.
- Security/privacy boundaries: clipboard payload inspection, trusted install gate, allowed file roots, plugin manifest trust, text-only file collection, and local filesystem writes.
- Tests and source audits that pin action availability, share bundle encoding, install safety, and dialog copy.

## Required Output Shape

```markdown
## 033 Sharing and Clipboard Trust Install

### Executive Summary

### What Users Can Do

### Core Concepts

### Entry Points

### User Workflows

### Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|

### State Machine

### Visual And Focus States

### Keystrokes And Commands

### Actions And Menus

### Automation And Protocol Surface

### Data, Storage, And Privacy Boundaries

### Error, Empty, Loading, And Disabled States

### Code Ownership

### Invariants And Regression Risks

### Verification Recipes

### Agent Notes

### Related Features

### Open Questions And Gaps
```

## Specific Questions To Answer

1. Which selected results become portable share bundles, and which fall back to plain deeplinks?
2. What exact files are included for scripts, scriptlets, skills, and agents?
3. How are plugin manifest fields resolved for each shared kind?
4. What prevents a copied local share link from opening an immediate import prompt?
5. What does the clipboard watcher read, and when does it avoid reading payload data?
6. What exact trust prompt does the user see, and what happens on Install vs Ignore?
7. Where does installation write files, and how are plugin id collisions handled?
8. What file paths are rejected, and which top-level directories are allowed?
9. Which source paths refresh after successful install?
10. What can agents verify without touching the real clipboard or filesystem?
11. What tests should run before changing share bundle format, action availability, startup watcher, or install behavior?
12. What should not be assumed about binary assets, remote transport, signed bundles, or automatic trust?
