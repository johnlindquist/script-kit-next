/**
 * Eval for flows/components.md, born from the 2026-07-11 Tips regression:
 * footer-button lookalikes were hand-built in a surface renderer instead of
 * reusing the footer_chrome / native footer component family, and a
 * selectable list shipped without scroll-into-view. The components flow is
 * the shared-UI owner, so it must steer any footer/list work back to the
 * shared component contract.
 *
 * Repo-bound: runs in the real repository (cwd ".."), planning only, no
 * edits. Cost: one codex turn per case.
 */
import type { EvalCase } from "/Users/johnlindquist/dev/mdflow/src/evals";

const cases: EvalCase[] = [
  {
    name: "footer button work routes to footer_chrome and the native footer family",
    cwd: "..",
    prompt:
      "Plan only, do not edit any files: a surface needs footer buttons with a label plus keycap (like 'Copy Example ↵' and 'Back Esc'). Report which existing components and files own footer buttons and keycaps, how a main-window surface gets them, and what a renderer must never do.",
    timeoutMs: 300_000,
    check: ({ stdout, exitCode }) => {
      if (exitCode !== 0) return `exit ${exitCode}`;
      const out = stdout.toLowerCase();
      if (!out.includes("footer_chrome"))
        return "missing the footer_chrome.rs component family";
      if (!out.includes("footerbuttonconfig") && !out.includes("native footer"))
        return "missing the native footer / FooterButtonConfig ownership";
      if (!out.includes("main_window_footer_slot") && !out.includes("render_simple_hint_strip"))
        return "missing the sanctioned GPUI fallback path (main_window_footer_slot / render_simple_hint_strip)";
      if (/\bcargo (check|test|build)\b/.test(out) && !out.includes("agent-cargo"))
        return "proposed bare cargo instead of ./scripts/agentic/agent-cargo.sh";
      return null;
    },
  },
  {
    name: "selectable list guidance demands scroll-into-view",
    cwd: "..",
    prompt:
      "Plan only, do not edit any files: a new filterable list surface highlights a selected row and moves the selection with arrow keys. Report the shared list anatomy it must use, what happens on every selection move, and the exemplar files plus the gate that locks this behavior.",
    timeoutMs: 300_000,
    check: ({ stdout, exitCode }) => {
      if (exitCode !== 0) return `exit ${exitCode}`;
      const out = stdout.toLowerCase();
      if (!out.includes("scroll_to_item"))
        return "guidance never scrolls the selection into view (missing scroll_to_item)";
      if (!out.includes("uniform_list") && !out.includes("track_scroll"))
        return "guidance does not use a tracked uniform_list";
      if (/\bcargo (check|test|build)\b/.test(out) && !out.includes("agent-cargo"))
        return "proposed bare cargo instead of ./scripts/agentic/agent-cargo.sh";
      return null;
    },
  },
];

export default cases;
