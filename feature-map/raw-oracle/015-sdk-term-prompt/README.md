# 015 SDK TermPrompt Raw Oracle Status

Feature 015 was selected as the next uncovered atlas pass because feature 014 explicitly distinguishes Quick Terminal from SDK-spawned `AppView::TermPrompt`.

Status: completed after CLI fallback.

What is preserved here:

- `prompt.md`: the intended Oracle prompt.
- `bundle-map.md`: the full bundle manifest, including the large bundle and the tight fallback bundle.
- `answer.md`: canonical extracted Oracle answer from the successful CLI fallback.
- `output.log`: full successful Oracle CLI session log.
- `session.json`: successful Oracle CLI session metadata.
- `attempts/`: complete logs and metadata copied from earlier failed Oracle attempts.

Earlier MCP/browser attempts failed or stalled before a recoverable answer was produced. The successful run used the Oracle CLI fallback with slug `sdk-term-atlas-cli`, inline tight bundle delivery, `gpt-5.4-pro`, and `--write-output feature-map/raw-oracle/015-sdk-term-prompt/answer.cli.md`.

Observed attempts:

| Slug | Result |
|---|---|
| `sdk-term-prompt-atlas` | API engine failed because `OPENAI_API_KEY` was missing. |
| `sdk-term-prompt-atlas-2` | Browser upload submitted, then left stale running metadata after second-poller heartbeat. |
| `sdk-term-prompt-atlas-3` | Conversation reattach submitted, then left stale running metadata with only partial "thinking" capture. |
| `sdk-term-prompt-atlas-4` | Browser upload aborted before submit because attachments were not present in the composer. |
| `sdk-term-prompt-atlas-5` | Inline large bundle submitted, then left stale running metadata after second-poller start. |
| `sdk-term-prompt-atlas-tight` | Tight attachment bundle submitted, then left stale running metadata after heartbeat. |
| `sdk-term-prompt-atlas-keep` | Keep-browser upload aborted before submit because attachments were not present in the composer. |
| `sdk-term-prompt-atlas-keep-2` | Keep-browser inline tight bundle submitted, then left stale running metadata after heartbeat. |
| `sdk-term-prompt-gemini` | Gemini browser mode completed with zero output. |
| `sdk-term-atlas-cli` | Oracle CLI fallback completed and produced the canonical `answer.md`. |
