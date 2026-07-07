/**
 * Eval for flows/brain.md: the flow must know the Day Page's load-bearing
 * surface contracts — the clipboard shelf projection (day file canonical,
 * editor shows the visible body only) and the kit:// preview return
 * semantics (footer-owned actions, "Back to …" never a bare "Close").
 *
 * Repo-bound: runs in the real repository (cwd ".."), planning only, no
 * edits. Cost: one codex turn per case.
 */
import type { EvalCase } from "/Users/johnlindquist/dev/mdflow/src/evals";

const cases: EvalCase[] = [
  {
    name: "keeps the clipboard shelf split/join contract when changing Today",
    cwd: "..",
    prompt:
      "Plan only, do not edit any files: we want to show a small icon next to each kept clipboard entry on the Today screen. Report which files own that surface, the projection contract between the day file and the editor buffer, and the exact gate that proves the day-file round trip stays lossless.",
    timeoutMs: 300_000,
    check: ({ stdout, exitCode }) => {
      if (exitCode !== 0) return `exit ${exitCode}`;
      const out = stdout.toLowerCase();
      if (!out.includes("sediment"))
        return "missing owner src/day_page/sediment.rs";
      if (!out.includes("shelf")) return "missing the clipboard shelf concept";
      if (!/split_day_page_clipboard_shelf|join_day_page_clipboard_shelf|split\s*(→|->)\s*join|lossless/.test(out))
        return "missing the lossless split/join projection contract";
      if (!out.includes("day_page::sediment"))
        return "missing the day_page::sediment test gate";
      if (/\bcargo (check|test|build)\b/.test(out) && !out.includes("agent-cargo"))
        return "proposed bare cargo instead of ./scripts/agentic/agent-cargo.sh";
      return null;
    },
  },
  {
    name: "keeps preview close-as-return semantics and footer-owned actions",
    cwd: "..",
    prompt:
      "Plan only, do not edit any files: a report says the kit:// resource preview on the Day Page needs a more obvious way to dismiss it. Report where the preview's actions are rendered, what the dismiss affordance must be labeled, and why adding an in-body Close link would violate the surface contract.",
    timeoutMs: 300_000,
    check: ({ stdout, exitCode }) => {
      if (exitCode !== 0) return `exit ${exitCode}`;
      const out = stdout.toLowerCase();
      if (!out.includes("footer"))
        return "missing footer ownership of preview actions";
      if (!out.includes("back to"))
        return 'missing the "Back to …" return-label contract';
      if (!/kit_resource_preview_return_label|day_page_footer_buttons/.test(out))
        return "missing the owning helpers (return label / footer buttons)";
      if (/\bcargo (check|test|build)\b/.test(out) && !out.includes("agent-cargo"))
        return "proposed bare cargo instead of ./scripts/agentic/agent-cargo.sh";
      return null;
    },
  },
];

export default cases;
