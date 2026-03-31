# Script Kit One-Shot Starters

Use this file when the fastest harness answer is:
pick one artifact, copy one starter, save it under `kit/main/`, then stop.

## Pick the Artifact

| Request shape | Artifact | Copy from | Write to |
|---------------|----------|-----------|----------|
| "make a clipboard cleanup command" | Script | `scripts/hello-world.ts` | `~/.scriptkit/kit/main/scripts/clipboard-cleanup.ts` |
| "make a bundle of text snippets" | Extension bundle | `extensions/starter.md` | `~/.scriptkit/kit/main/extensions/snippets.md` |
| "make an agent that reviews staged changes" | mdflow agent | `agents/review-pr.claude.md` | `~/.scriptkit/kit/main/agents/review-pr.claude.md` |

## Copy Commands

```bash
cp ~/.scriptkit/examples/scripts/hello-world.ts ~/.scriptkit/kit/main/scripts/my-script.ts
cp ~/.scriptkit/examples/extensions/starter.md ~/.scriptkit/kit/main/extensions/my-bundle.md
cp ~/.scriptkit/examples/agents/review-pr.claude.md ~/.scriptkit/kit/main/agents/my-agent.claude.md
```

## Rules

- Pick the smallest artifact that fits.
- Save only under `~/.scriptkit/kit/main/`.
- For scripts, start with `import "@scriptkit/sdk";`.
- For extension bundles, prefer `metadata` code fences.
- For agents, use underscore-prefixed `_sk_*` metadata keys.
