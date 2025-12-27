/**
 * AI Logging Reminder Plugin
 * 
 * Reminds agents to use SCRIPT_KIT_AI_LOG=1 when running the app.
 * Injects the log format legend into compaction context so agents
 * always have the legend available after context compaction.
 */
export const AiLoggingReminder = async () => {
  const LOG_LEGEND = `
## AI Compact Log Format (SCRIPT_KIT_AI_LOG=1)

**ALWAYS use \`SCRIPT_KIT_AI_LOG=1\` when running the app for testing.**

Format: \`SS.mmm|L|C|message\`
- SS.mmm = seconds.millis in current minute
- L = level: i=INFO, w=WARN, e=ERROR, d=DEBUG, t=TRACE
- C = category code (see below)

**Category Codes:**
| P=POSITION | A=APP | U=UI | S=STDIN | H=HOTKEY | V=VISIBILITY |
| E=EXEC | K=KEY | F=FOCUS | T=THEME | C=CACHE | R=PERF |
| W=WINDOW_MGR | X=ERROR | M=MOUSE_HOVER | L=SCROLL_STATE |
| Q=SCROLL_PERF | D=DESIGN | G=SCRIPT | N=CONFIG | Z=RESIZE |

**Example:**
\`\`\`bash
echo '{"type": "run", "path": "..."}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
\`\`\`
`.trim()

  return {
    // Inject log legend into compaction context
    "experimental.session.compacting": async (_input, output) => {
      output.context.push(LOG_LEGEND)
    }
  }
}
