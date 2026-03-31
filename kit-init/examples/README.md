# Script Kit Examples

Working examples demonstrating Script Kit patterns. Learn by reading, then create your own in `~/.scriptkit/kit/main/scripts/`.

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
| `main.md` | Core scriptlet patterns |
| `advanced.md` | Richer bundle patterns |
| `howto.md` | Bundle layout, metadata blocks, and fence types |

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
```

Then edit to suit your needs. Script Kit will detect it immediately.
