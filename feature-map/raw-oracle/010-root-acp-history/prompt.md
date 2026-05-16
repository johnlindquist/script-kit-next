We are iterating through every Script Kit GPUI app feature and building FEATURE_MAP.md. This is Feature 010: Root Unified Search ACP History / AI Conversations root rows / resume path.

Context bundle attached from /Users/johnlindquist/dev/script-kit-gpui includes AGENTS.md/CLAUDE.md process rules, owning main-menu-search-selection, acp-chat-core, acp-context-composer, builtin-filterable-surfaces, storage-cache-security, protocol, and agentic-testing skills, exact lat.md sections for Root Unified Search ACP History and verification, source-audit tests, menu syntax source-filter tests, and focused implementation excerpts around root ACP History symbols.

Important repo rules to account for:
- lat.md is the architecture/test knowledge graph. Behavior/architecture/test changes require lat.md updates and lat check.
- Verification should be state-first where possible; screenshots only when state cannot prove the behavior.
- Root ACP History rows are passive saved-conversation launcher results; selecting one resumes through the shared ACP history resume path.

Task: Produce a terse but comprehensive feature map for Feature 010 only. Focus on what a user can do, every relevant state, keystroke/shortcut, scenario, visual/surface state, ownership boundary, and verification/check implication. Do not write code. Do not create downloadable artifacts.

Use this exact structure:

## 010 Root Unified Search ACP History / AI Conversations root rows / resume path

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
- 011 <name>

Keep it dense and useful for implementation/review agents. Prefer concrete file/function/test references from the bundle over generic descriptions. If uncertain, mark as inference.
