/**
 * Eval for flows/clipboard.md, ported from the retired imps eval scenario
 * clipboard-no-popup-001: the flow must route clipboard-to-brain work to the
 * right owners and gates without reviving the popup path.
 *
 * Repo-bound: runs in the real repository (cwd ".."), planning only, no
 * edits. Cost: one codex turn per case.
 */
import type { EvalCase } from "/Users/johnlindquist/dev/mdflow/src/evals";

const cases: EvalCase[] = [
  {
    name: "routes clipboard-to-brain work to owners and the no-popup gate",
    cwd: "..",
    prompt:
      "Plan only, do not edit any files: change clipboard-to-brain tracking without reintroducing popup UI. Report the owned paths you would touch, the required preflight, and the exact gate that proves no popup machinery returns.",
    timeoutMs: 300_000,
    check: ({ stdout, exitCode }) => {
      if (exitCode !== 0) return `exit ${exitCode}`;
      const out = stdout.toLowerCase();
      if (!out.includes("clipboard_history")) return "missing owner path src/clipboard_history/**";
      if (!out.includes("sediment")) return "missing sediment ownership (src/day_page/sediment.rs)";
      if (!/no[_-]?popup/.test(out)) return "missing the no-popup contract gate";
      if (/\bcargo (check|test|build)\b/.test(out) && !out.includes("agent-cargo"))
        return "proposed bare cargo instead of ./scripts/agentic/agent-cargo.sh";
      return null;
    },
  },
];

export default cases;
