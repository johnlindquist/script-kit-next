# DevTools Truth Scenario Oracle Prompt

Use the attached packx bundle as the source of truth for current Script Kit GPUI DevTools coverage, existing agentic scripts, existing tests, and repo process rules.

Review the proposed `dt-truth-*` scenarios for internal text/action truthfulness. Enforce this definition of brand-new:

- The scenario ID does not appear in existing agentic scripts, DevTools scripts, or tests.
- The scenario is not executed through `scripts/agentic/index.ts`, `scripts/agentic/user-story-audit.ts`, `scripts/agentic/surface-navigator.ts`, smoke tests, source audits, or any existing stress recipe.
- The scenario is not a renamed duplicate of existing coverage such as keyboard hint parity, footer persistence, actions discoverability, Mini/Full layout continuity, file search preview sanitization, current-app frontmost command behavior, or destructive confirmation safety.
- The scenario uses at least three user-meaningful transitions.
- The scenario is non-destructive and avoids OS mutation, system pasteboard mutation, external activation, shell execution, microphone capture, and process control.
- The proof target is visible text truthfulness against selected semantic ID, footer intent, action ID, handler ID, side-effect class, disabled reason, focus owner, route generation, and target surface.

Accept, revise, or replace the 50 candidate stories. Return text only. Do not create downloadable files. Name the first local implementation slice and exact verification commands. The first slice must not introduce a runner; it should create only a ledger, receipt schema, README, and source contract.

Use Oracle session slug `new-devtools-scenarios-plan`.
