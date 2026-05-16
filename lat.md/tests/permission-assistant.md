---
lat:
  require-code-mention: true
---
# Permission Assistant

Permission Assistant tests pin the passive macOS setup flow for privacy permissions.

## Built-in assistant entry points

Accessibility and Screen Recording built-ins must route to the retained assistant instead of legacy prompt or settings-only commands.

## Passive detection does not prompt

Permission status reads and MCP permission tools must use passive preflight APIs and must not request access, write TCC, or automate System Settings.

## Overlay lifetime and teardown

The assistant must retain one active native overlay handle and drop it through a deterministic controller teardown path.
