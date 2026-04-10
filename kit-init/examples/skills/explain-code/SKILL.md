---
name: explain-code
description: Explain how a file, subsystem, or feature works by tracing entrypoints, data flow, state changes, and important contracts. Use when the user wants orientation instead of edits.
---

# Explain Code

Use this skill when the user asks how something works, where behavior comes from, or how multiple files fit together.

## Workflow

1. Start from the file, symbol, or feature the user named.
2. Trace the real entrypoints, state mutations, async boundaries, and outputs in source.
3. Call out invariants, side effects, and compatibility contracts that shape the design.
4. If the behavior spans multiple files, give a short map before walking through the flow.
5. Use exact file references for the key jumps.

## Output

- Short overview first
- Then the flow in execution order
- Then important gotchas or extension points

## Avoid

- Do not dump every helper in the directory
- Do not answer from memory when the source can be read
- Do not skip the reason a surprising code path exists
