---
name: plan-feature
description: Turn a feature request into an implementation plan with scope, file targets, risks, and verification steps. Use when the user wants planning before coding.
---

# Plan Feature

Use this skill when the request is large enough, ambiguous enough, or risky enough that a plan is more useful than immediate edits.

## Workflow

1. Restate the goal in terms of user-visible outcome and non-goals.
2. Read the relevant code paths before proposing changes.
3. Break the work into a small number of slices with clear boundaries.
4. For each slice, note likely files, data flow, edge cases, and the verification needed.
5. Call out migration, compatibility, and rollback risks early.
6. Prefer the smallest shippable version before follow-up polish.

## Output

- Goal and constraints
- Proposed approach
- Implementation slices
- Risks and unknowns
- Verification plan

## Avoid

- Do not jump straight into code without reading source
- Do not propose unrelated refactors
- Do not leave validation as a vague final step
