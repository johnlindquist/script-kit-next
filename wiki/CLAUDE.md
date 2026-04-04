# Script Kit GPUI Wiki Contract

This directory has three layers:

1. `wiki/raw/<snapshot>/<repo-relative-path>` — immutable copied source documents
2. `wiki/pages/*.md` — LLM-maintained wiki pages
3. `wiki/CLAUDE.md` — the schema and operating contract for all wiki operations

## Directory layout

- `wiki/raw/<snapshot>/<repo-relative-path>` — immutable copied source files
- `wiki/pages/*.md` — wiki pages with LLM-maintained narrative and ingest-maintained source links
- `wiki/index.md` — generated page/source index
- `wiki/log.md` — append-only ingest history
- `wiki/sources.json` — ingest manifest and bootstrap seed data

## Page ownership model

The ingest script and page files do not own the same fields.

### Ingest-owned fields

- YAML frontmatter
- `## Key Files`
- `## Source Documents`
- `## Related Pages`
- `wiki/index.md`
- `wiki/log.md`

### Page-owned fields

- the summary paragraph(s) directly below `# <Title>`
- `## Key Facts`
- any optional `## ...` sections after `## Related Pages`

## Non-negotiable rules

1. Never edit files under `wiki/raw/`.
2. `wiki/raw/` paths are keyed by an explicit snapshot, normally the current git SHA passed to ingest.
3. Every page under `wiki/pages/` must include this YAML frontmatter:
   - `title`
   - `slug`
   - `sourceSnapshot`
   - `sourceDocuments`
   - `relatedPages`
   - `generatedBy`
   - `generatedAt`
4. Every `sourceDocuments` entry must point to `raw/<snapshot>/...`.
5. Re-ingest must preserve existing page summary, `## Key Facts`, and any optional trailing sections.
6. Re-ingest may rewrite only frontmatter plus the ingest-owned fields listed above.
7. Keep page slugs stable. Update existing pages instead of creating near-duplicates.
8. `wiki/index.md` is regenerated on every ingest.
9. `wiki/log.md` is append-only.
10. `wiki/sources.json` provides source metadata and bootstrap seed content for missing pages. It is not the authority for prose in an existing page.

## Required page sections

Every page must contain these sections in this order:

1. `## Key Facts`
2. `## Key Files`
3. `## Source Documents`
4. `## Related Pages`

Additional sections are allowed after `## Related Pages`.

## Ingest command

```bash
bun scripts/wiki/ingest.ts --root . --snapshot <git-sha> --config wiki/sources.json
```

## Snapshot policy

Use the exact commit SHA for immutable raw copies. When tracked source documents change, rerun ingest with the new SHA instead of editing old raw files.
