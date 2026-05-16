We are iterating through every Script Kit GPUI app feature and building FEATURE_MAP.md. This is Feature 006: Notes Window / Notes Browse / Notes-hosted ACP.

Context bundle attached from /Users/johnlindquist/dev/script-kit-gpui includes AGENTS.md/CLAUDE.md process rules, owning skill .agents/skills/notes-window/SKILL.md, adjacent skills for ACP/actions/protocol/agentic-testing, relevant lat.md pages including notes.md, acp-chat.md, tests/notes-acp.md, verification.md, Notes source files, Notes tests, and agentic scripts.

Important repo rules to account for:
- lat.md is the architecture/test knowledge graph. Behavior/architecture/test changes require lat.md updates and lat check.
- Verification should be state-first where possible; screenshots only when state cannot prove the behavior.
- Notes is a separate floating host, not a launcher panel clone.

Task: Produce a terse but comprehensive feature map for Feature 006 only. Focus on what a user can do, every relevant state, keystroke/shortcut, scenario, visual/surface state, ownership boundary, and verification/check implication. Do not write code. Do not create downloadable artifacts.

Use this exact structure:

## 006 Notes Window / Notes Browse / Notes-hosted ACP

### Boundaries
- Owns:
- Does not own:
- Adjacent feature dependencies:

### User Stories
- As a user, ...

### State Model
- Window/surface modes:
- Persistent data/storage:
- Runtime caches/handles:
- Error/empty/loading states:

### Keystrokes And Commands
- Shortcut/gesture:
  - Context:
  - Expected behavior:
  - Edge cases:

### Scenarios
- Scenario name:
  - Start state:
  - Steps:
  - Expected result:
  - Verification signal:

### Visual States
- State:
  - What is visible:
  - Focus owner:
  - Footer/actions/chrome:
  - Accessibility/automation target if applicable:

### Invariants And Regression Risks
- Invariant:
  - Why it matters:
  - Files/tests that pin it:

### Verification Map
- Source/static checks:
- Runtime/state-first proofs:
- Visual/native proofs only if needed:

### Open Questions / Gaps
- Gap:
  - Why it matters:
  - Suggested next check:

### Suggested next feature
- 007 <name>

Keep it dense and useful for implementation/review agents. Prefer concrete file/function/test references from the bundle over generic descriptions. If uncertain, mark as inference.
