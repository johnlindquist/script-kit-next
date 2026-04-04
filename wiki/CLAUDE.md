# Script Kit GPUI Wiki Contract

This directory is the project wiki. It has one immutable layer and one generated wiki layer.

## Directory layout

- `wiki/raw/<snapshot>/<repo-relative-path>` — immutable copied source files
- `wiki/pages/*.md` — generated wiki pages
- `wiki/index.md` — generated page/source index
- `wiki/log.md` — append-only ingest history
- `wiki/sources.json` — authoritative ingest manifest

## Non-negotiable rules

1. Never edit files under `wiki/raw/`.
2. `wiki/raw/` paths are keyed by an explicit snapshot, normally the current git SHA.
3. Every page under `wiki/pages/` must include this YAML frontmatter:
   - `title`
   - `slug`
   - `sourceSnapshot`
   - `sourceDocuments`
   - `relatedPages`
   - `generatedBy`
   - `generatedAt`
4. Every `sourceDocuments` entry must point to `raw/<snapshot>/...`.
5. `wiki/index.md` is regenerated on every ingest.
6. `wiki/log.md` is append-only.
7. In v1, the ingest script owns the full body of each page in `wiki/pages/`.
8. Keep page slugs stable. Update existing pages instead of creating near-duplicates.

## Required page sections

Every generated page must contain these sections in this order:

1. `## Key Facts`
2. `## Key Files`
3. `## Source Documents`
4. `## Related Pages`

## Ingest command

```bash
bun scripts/wiki/ingest.ts --root . --snapshot <git-sha> --config wiki/sources.json
```

## Snapshot policy

Use the exact commit SHA for immutable raw copies. For this bootstrap cycle, use `4be166ea`. If a source changes later, run ingest again with a new snapshot instead of editing old raw files.
