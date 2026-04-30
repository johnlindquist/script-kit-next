---
lat:
  require-code-mention: true
---
# ACP Dictation

These tests lock ACP-targeted dictation delivery so transcripts always land in the intended ACP chat surface.

## Detached window handoff

These specs cover ACP-targeted dictation when a detached ACP popup is already alive.

### Closes detached before embedded reveal

ACP-targeted dictation must close any detached ACP popup before opening embedded ACP, so the orchestrator reveal can focus the main composer.
