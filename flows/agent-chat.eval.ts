/**
 * Eval for flows/agent-chat.md: the flow must know the load-bearing
 * contracts of the AI input's attachments and chips — the attachment-gated
 * accent logic shared by `@` context mentions and `-` flow tokens, the
 * compact-token + alias-registry staging that keeps chips honest, and the
 * skill/flow staging parity of the `-` flow search.
 *
 * Quality bar (deliberately higher than the older suites): each case
 * requires exact owning helpers by name, the gating rule ("attached, not
 * lookalike"), and the smallest verifying gate — not just file ownership.
 *
 * Repo-bound: runs in the real repository (cwd ".."), planning only, no
 * edits. Cost: one codex turn per case.
 */
import type { EvalCase } from "/Users/johnlindquist/dev/mdflow/src/evals";

const agentCargoGuard = (out: string): string | null =>
  /\bcargo (check|test|build)\b/.test(out) && !out.includes("agent-cargo")
    ? "proposed bare cargo instead of ./scripts/agentic/agent-cargo.sh"
    : null;

const cases: EvalCase[] = [
  {
    name: "knows flow tokens share the attachment-gated accent of @ chips",
    cwd: "..",
    prompt:
      "Plan only, do not edit any files: a report says that after picking a flow from the `-` flow search in the Agent Chat composer, the staged `-name` token renders in the plain text color while attached `@` mentions render in the accent color. Report which helper owns the `@` accent gating, what condition gates a token getting the accent (versus a lookalike token that must stay plain), which helper stages the flow's context part, and where in the composer render the highlight ranges are assembled.",
    timeoutMs: 300_000,
    check: ({ stdout, exitCode }) => {
      if (exitCode !== 0) return `exit ${exitCode}`;
      const out = stdout.toLowerCase();
      if (!out.includes("attached_inline_mention_highlight_ranges"))
        return "missing the owning @ accent helper attached_inline_mention_highlight_ranges";
      if (!/attach/.test(out) || !/(lookalike|unattached|not attached|actually attached)/.test(out))
        return "missing the gating rule: accent only for actually-attached parts, lookalikes stay plain";
      if (!/build_flow_context_part|flow_owner_label/.test(out))
        return "missing the flow staging helper (build_flow_context_part / FLOW_OWNER_LABEL)";
      if (!out.includes("mention_highlights"))
        return "missing the render assembly point (mention_highlights)";
      return agentCargoGuard(out);
    },
  },
  {
    name: "keeps compact-token chips honest via the alias registry and sync",
    cwd: "..",
    prompt:
      "Plan only, do not edit any files: we want the Agent Chat composer to keep showing only a compact basename token after the user picks a file from the `@file:` subsearch, while the submitted context still carries the full path. Report which registry preserves the full path behind the compact token, which routine keeps pending context parts in sync when inline tokens are typed or deleted, and how a context part maps back to its inline token.",
    timeoutMs: 300_000,
    check: ({ stdout, exitCode }) => {
      if (exitCode !== 0) return `exit ${exitCode}`;
      const out = stdout.toLowerCase();
      if (!/typed_mention_aliases|alias registry|mention alias/.test(out))
        return "missing the typed_mention_aliases registry that preserves the full path";
      if (!out.includes("sync_inline_mentions"))
        return "missing sync_inline_mentions as the token<->part lifecycle owner";
      if (!/part_to_inline_token/.test(out))
        return "missing part_to_inline_token (part -> inline token mapping)";
      return agentCargoGuard(out);
    },
  },
  {
    name: "keeps `-` flow staging parity with the skill accept path",
    cwd: "..",
    prompt:
      "Plan only, do not edit any files: a reviewer asks why accepting a flow from the `-` flow search leaves only a short `-name` token in the composer instead of pasting the flow's markdown. Report what the composer keeps versus what the submitted prompt carries, which helper builds the staged prompt wording for flows versus skills, what distinguishes a staged flow part from a staged skill part, and the smallest test gate that covers this area.",
    timeoutMs: 300_000,
    check: ({ stdout, exitCode }) => {
      if (exitCode !== 0) return `exit ${exitCode}`;
      const out = stdout.toLowerCase();
      if (!out.includes("build_staged_skill_prompt"))
        return "missing build_staged_skill_prompt as the staged-prompt owner";
      if (!/flow_owner_label|owner_label/.test(out))
        return "missing the owner-label discriminator between flow and skill parts";
      if (!/compact|short|token in the composer|-name/.test(out))
        return "missing the compact-token-in-composer vs full-markdown-in-prompt split";
      if (!/agent-cargo\.sh test/.test(out))
        return "missing a concrete agent-cargo test gate";
      return agentCargoGuard(out);
    },
  },
];

export default cases;
