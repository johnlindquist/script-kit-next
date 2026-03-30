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

Built-in extension bundles showing scriptlet patterns (managed by the app).

## Creating Your Own

Copy any example to your scripts directory:

```bash
cp ~/.scriptkit/examples/scripts/hello-world.ts ~/.scriptkit/kit/main/scripts/my-script.ts
```

Then edit to suit your needs. Script Kit will detect it immediately.
