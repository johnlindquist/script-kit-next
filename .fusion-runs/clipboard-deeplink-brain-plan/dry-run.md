# Local Fusion Dry Run

- Output directory: /Users/johnlindquist/dev/script-kit-gpui/.fusion-runs/clipboard-deeplink-brain-plan
- Providers: codex-gpt-5.5-high

## Panel Commands

- codex-gpt-5.5-high: `codex exec --skip-git-repo-check --ephemeral -C /Users/johnlindquist/dev/script-kit-gpui -s read-only -m gpt-5.5 -c 'model_reasoning_effort="high"' -` (prompt via stdin)

## Judge Command

- codex-gpt-5.5-high-judge: `codex exec --skip-git-repo-check --ephemeral -C /Users/johnlindquist/dev/script-kit-gpui -s read-only -m gpt-5.5 -c 'model_reasoning_effort="high"' -`

## Critic Command

- codex-gpt-5.5-high-judge: `codex exec --skip-git-repo-check --ephemeral -C /Users/johnlindquist/dev/script-kit-gpui -s read-only -m gpt-5.5 -c 'model_reasoning_effort="high"' -` (conditional on judge escalation)

## Synthesizer Command

- codex-gpt-5.5-high-synthesizer: `codex exec --skip-git-repo-check --ephemeral -C /Users/johnlindquist/dev/script-kit-gpui -s read-only -m gpt-5.5 -c 'model_reasoning_effort="high"' -`
