Your previous port of the Script Kit v1 script `{{FILENAME}}` FAILED validation. Fix it.

## Validator that failed: {{VALIDATOR_ID}}

Raw validator output (this is ground truth — the tool actually ran):

```
{{VALIDATOR_FAILURE}}
```

## Your previous output (the file that failed)

```ts
{{PREVIOUS_OUTPUT}}
```

## Original v1 source (for reference — behavior must still match it)

```ts
{{SCRIPT_SOURCE}}
```

## Migration guidance for the APIs involved

{{COMPAT_GUIDANCE}}

## Rules (unchanged)

- Only ambient v2 SDK globals plus bun/node built-ins; no `@johnlindquist/kit` import; no new npm dependencies.
- Preserve all metadata fields with identical values.
- Declare every user-visible difference in `behavior_changes`.
- Fix the reported failure without breaking what already passed. Change as little as possible.

## Output contract

Respond with EXACTLY these two blocks and nothing else:

===PORTED_SCRIPT===
<the complete corrected file>
===END_PORTED_SCRIPT===
===MIGRATION_NOTE===
{"summary": "<one sentence>", "behavior_changes": [], "confidence": "high" | "medium" | "low"}
===END_MIGRATION_NOTE===
