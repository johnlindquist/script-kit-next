/**
 * Autonomous Testing Reminder Plugin
 * 
 * Encourages agents to follow the autonomous testing principles from AGENTS.md.
 * Injects testing guidelines into compaction context.
 */
export const AutonomousTestingReminder = async () => {
  const TESTING_GUIDELINES = `
## Autonomous Testing Protocol (from AGENTS.md)

**ALL UI changes MUST be tested using the stdin JSON protocol before committing.**

### The Build-Test-Iterate Loop (MANDATORY)

\`\`\`bash
# 1. Build
cargo build

# 2. Run test via stdin JSON with AI compact logs
echo '{"type": "run", "path": "'$(pwd)'/tests/smoke/<test>.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# 3. Check output for expected behavior
# 4. If broken, fix code and repeat from step 1
\`\`\`

### Key Principles

1. **NEVER pass scripts as command line arguments** - use stdin JSON protocol
2. **ALWAYS use SCRIPT_KIT_AI_LOG=1** - saves ~70% tokens
3. **YOU must test it** - do not ask the user to test
4. **Run verification before commits**: \`cargo check && cargo clippy && cargo test\`

### Available stdin Commands

\`\`\`json
{"type": "run", "path": "/absolute/path/to/script.ts"}
{"type": "show"}
{"type": "hide"}
{"type": "setFilter", "text": "search term"}
\`\`\`

### Test Locations

- \`tests/smoke/\` - E2E integration tests
- \`tests/sdk/\` - Individual SDK method tests

### Visual Testing (for layout issues)

\`\`\`bash
./scripts/visual-test.sh tests/smoke/<test>.ts 3
\`\`\`
`.trim()

  return {
    // Inject testing guidelines into compaction context
    "experimental.session.compacting": async (_input, output) => {
      output.context.push(TESTING_GUIDELINES)
    }
  }
}
