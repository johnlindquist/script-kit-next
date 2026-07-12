/**
 * Eval for flows/builtins.md, born from the 2026-07-11 Tips regression: an
 * agent built a new builtin browser with a hand-rolled footer (no native
 * footer surface) and a selectable list that never scrolled its selection
 * into view. The flow must plan any new/changed builtin browser against the
 * shared anatomy: tracked uniform_list + scroll_to_item for selection, and
 * the persistent native footer (native_footer_surface + FooterButtonConfig +
 * main_window_footer_slot fallback).
 *
 * Repo-bound: runs in the real repository (cwd ".."), planning only, no
 * edits. Cost: one codex turn per case.
 */
import type { EvalCase } from "/Users/johnlindquist/dev/mdflow/src/evals";

const cases: EvalCase[] = [
  {
    name: "new builtin browser plan reuses native footer and scroll-into-view list anatomy",
    cwd: "..",
    prompt:
      "Plan only, do not edit any files: add a new built-in 'Snippets' browser to the launcher — a filterable list of snippets on the left, a preview pane on the right, Enter copies the selected snippet, Esc goes back. Report the exact files, shared components, and helpers you would use for the list, the keyboard/wheel/click selection behavior, and the footer, plus the gates that prove consistency.",
    timeoutMs: 300_000,
    check: ({ stdout, exitCode }) => {
      if (exitCode !== 0) return `exit ${exitCode}`;
      const out = stdout.toLowerCase();
      if (!out.includes("scroll_to_item"))
        return "plan never scrolls the selection into view (missing scroll_to_item)";
      if (!out.includes("uniform_list") && !out.includes("track_scroll"))
        return "plan does not use a tracked uniform_list for the selectable list";
      if (!out.includes("native_footer_surface"))
        return "plan does not register a native footer surface for the new view";
      if (!out.includes("footerbuttonconfig") && !out.includes("footer_button"))
        return "plan does not declare footer buttons with the shared FooterButtonConfig components";
      if (!out.includes("main_window_footer_slot"))
        return "plan does not route the GPUI fallback footer through main_window_footer_slot";
      if (/\bcargo (check|test|build)\b/.test(out) && !out.includes("agent-cargo"))
        return "proposed bare cargo instead of ./scripts/agentic/agent-cargo.sh";
      return null;
    },
  },
  {
    name: "footer change plan refuses hand-rolled footer chrome",
    cwd: "..",
    prompt:
      "Plan only, do not edit any files: the Tips built-in needs a third footer button, 'Open Docs' on ⌘D. Report exactly where the button is declared, how it is dispatched, and which existing components render it. Explicitly state what you must NOT do.",
    timeoutMs: 300_000,
    check: ({ stdout, exitCode }) => {
      if (exitCode !== 0) return `exit ${exitCode}`;
      const out = stdout.toLowerCase();
      if (!out.includes("main_window_footer_buttons_for_current_view"))
        return "plan does not declare the button in main_window_footer_buttons_for_current_view";
      if (!out.includes("dispatch_main_window_footer_action"))
        return "plan does not dispatch through dispatch_main_window_footer_action";
      if (!out.includes("footerbuttonconfig") && !out.includes("footer_button"))
        return "plan does not use the shared FooterButtonConfig components";
      if (/\bcargo (check|test|build)\b/.test(out) && !out.includes("agent-cargo"))
        return "proposed bare cargo instead of ./scripts/agentic/agent-cargo.sh";
      return null;
    },
  },
];

export default cases;
