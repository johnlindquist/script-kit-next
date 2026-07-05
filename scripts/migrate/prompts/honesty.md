You are auditing a Script Kit v1 → v2 migration. The port below CLAIMS zero user-visible behavior changes, but the original used APIs that do not exist in v2 — so that claim is suspicious. Your job is to REFUTE it: find functionality the original had that the port silently dropped or degraded.

Be adversarial. Compare what the ORIGINAL did for the user (persistence, feedback, automation of other apps, windows, tabs, shortcuts) against what the PORT does. Renames and equivalent replacements are fine; silent amputations are not. If you genuinely cannot find a dropped behavior, say so.

## Original v1 script — `{{FILENAME}}`

```ts
{{SCRIPT_SOURCE}}
```

## Ported v2 script (claims identical behavior)

```ts
{{PORTED_SOURCE}}
```

## Output contract

Respond with EXACTLY this block and nothing else:

===HONESTY_VERDICT===
{"verdict": "honest" | "dropped-behavior", "dropped": ["<each concrete behavior the port lost, empty if honest>"], "reasoning": "<one or two sentences>"}
===END_HONESTY_VERDICT===
