/**
 * Agent adapter. Default backend is the Claude Code CLI in print mode
 * (`claude -p --output-format json`); override with SK_MIGRATE_AGENT_CMD to
 * point at any command that reads a prompt on stdin and writes the response to
 * stdout (plain text is accepted, the Claude JSON envelope is unwrapped).
 *
 * Porting is a pure text transform — the agent needs no tools, so print mode
 * with a single turn is the cheapest correct shape.
 */

export interface AgentResult {
  text: string;
  costUsd?: number;
}

const DEFAULT_CMD = "claude -p --output-format json";
// Long scripts + parallel calls sharing rate limits can push a single port
// turn well past 5 minutes; 10 is the default, SK_MIGRATE_AGENT_TIMEOUT_MS overrides.
const AGENT_TIMEOUT_MS = parseInt(process.env.SK_MIGRATE_AGENT_TIMEOUT_MS ?? "", 10) || 600_000;

export function agentCommand(): string {
  return process.env.SK_MIGRATE_AGENT_CMD || DEFAULT_CMD;
}

export async function callAgent(prompt: string): Promise<AgentResult> {
  const cmd = agentCommand();
  const proc = Bun.spawn(["sh", "-c", cmd], {
    stdin: "pipe",
    stdout: "pipe",
    stderr: "pipe",
  });
  proc.stdin.write(prompt);
  proc.stdin.end();

  let timedOut = false;
  const killer = setTimeout(() => {
    timedOut = true;
    proc.kill();
  }, AGENT_TIMEOUT_MS);
  const [stdout, stderr, code] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);
  clearTimeout(killer);

  if (timedOut) {
    throw new Error(
      `agent command timed out after ${Math.round(AGENT_TIMEOUT_MS / 1000)}s (raise SK_MIGRATE_AGENT_TIMEOUT_MS): ${cmd}`,
    );
  }
  if (code !== 0) {
    throw new Error(
      `agent command failed (exit ${code}): ${cmd}\n${stderr.trim().slice(-1000)}`,
    );
  }

  // Claude Code print-mode JSON envelope: { result: string, total_cost_usd, ... }
  try {
    const envelope = JSON.parse(stdout);
    if (envelope && typeof envelope.result === "string") {
      return { text: envelope.result, costUsd: envelope.total_cost_usd };
    }
  } catch {
    // plain-text backend
  }
  return { text: stdout };
}

/** Extract the content between ===NAME=== and ===END_NAME=== sentinels. */
export function extractBlock(text: string, name: string): string | null {
  const re = new RegExp(
    `===${name}===\\s*\\n([\\s\\S]*?)\\n?===END_${name}===`,
  );
  const m = text.match(re);
  if (!m) return null;
  let body = m[1];
  // Tolerate an agent that wrapped the block body in a code fence anyway.
  const fenced = body.match(/^\s*```[a-z]*\n([\s\S]*?)\n```\s*$/);
  if (fenced) body = fenced[1];
  return body.trim() + "\n";
}

export function parseJsonBlock<T>(text: string, name: string): T | null {
  const block = extractBlock(text, name);
  if (!block) return null;
  try {
    return JSON.parse(block) as T;
  } catch {
    return null;
  }
}
