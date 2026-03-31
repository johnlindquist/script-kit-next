# Script Kit Examples

Working examples demonstrating Script Kit patterns. Learn by reading, then create your own in the matching workspace directory:

- scripts → `~/.scriptkit/kit/main/scripts/`
- extension / scriptlet bundles → `~/.scriptkit/kit/main/extensions/`
- mdflow agents → `~/.scriptkit/kit/main/agents/`

## Start Here

| Goal | Copy from | Write to | Read next |
|------|-----------|----------|-----------|
| New script | `scripts/hello-world.ts` | `~/.scriptkit/kit/main/scripts/<name>.ts` | `~/.scriptkit/skills/script-authoring/SKILL.md` |
| New extension / scriptlet bundle | `extensions/starter.md` | `~/.scriptkit/kit/main/extensions/<name>.md` | `~/.scriptkit/skills/scriptlets/SKILL.md` |
| New mdflow agent | `agents/review-pr.claude.md` | `~/.scriptkit/kit/main/agents/<name>.<backend>.md` | `~/.scriptkit/skills/agents/SKILL.md` |

In this repo, "extension bundle" and "scriptlet bundle" mean the same thing.

Pick one artifact, copy one starter, save it under `kit/main/`, then stop. Do not create multiple artifact types for one request.

## Scripts (`scripts/`)

| File | Pattern Demonstrated |
|------|---------------------|
| `hello-world.ts` | Basic prompt (`arg`) and HTML display (`div`) |
| `choose-from-list.ts` | Rich choices with descriptions and preview panels |
| `clipboard-transform.ts` | Clipboard read/transform/write workflow |
| `path-picker.ts` | File picker and Bun file operations |

## Extensions (`extensions/`)

Reference markdown bundles copied from the built-in examples kit.

| File | Pattern Demonstrated |
|------|---------------------|
| `starter.md` | Smallest copyable bundle for one-shot authoring |
| `howto.md` | Best first read for bundle rules and metadata |
| `main.md` | Core scriptlet patterns |
| `advanced.md` | Richer bundle patterns and edge cases |

Copy patterns from these files into `~/.scriptkit/kit/main/extensions/`.

## Agents (`agents/`)

Reference mdflow agent files.

| File | Pattern Demonstrated |
|------|---------------------|
| `review-pr.claude.md` | Minimal single-turn Claude agent |
| `plan-feature.i.gemini.md` | Interactive Gemini agent with `_inputs` |

Copy patterns from these files into `~/.scriptkit/kit/main/agents/`.

## Creating Your Own

Copy any example to your workspace:

```bash
cp ~/.scriptkit/examples/scripts/hello-world.ts ~/.scriptkit/kit/main/scripts/my-script.ts
cp ~/.scriptkit/examples/extensions/starter.md ~/.scriptkit/kit/main/extensions/my-bundle.md
cp ~/.scriptkit/examples/agents/review-pr.claude.md ~/.scriptkit/kit/main/agents/my-agent.claude.md
```

Then edit to suit your needs. Script Kit will detect it immediately.
