# AI Chat Tool Use & Actions UX Patterns

Scope: UX patterns observed in major AI chat products/docs for tool use and actions, focused on tool call visualization, approval flows, action buttons, and result display.

## 1) Tool call visualization

Patterns
- Show tool call payloads (inputs/outputs) inline in the conversation so users can confirm what will run or what ran.
- Provide an expand/collapse affordance (chevron/caret) to reveal full JSON details.
- Surface tool call metadata (tool name, args) as a distinct step in the chat so users can follow the chain of tool use.
- Distinguish read vs write actions (or show a warning state for write actions) to set user expectations before execution.

Evidence/examples
- ChatGPT developer mode shows JSON tool payloads and allows expanding/collapsing full tool input/output. (OpenAI)
- ChatGPT shows tool-call payloads in the UI for MCP connectors so users can confirm inputs/outputs. (OpenAI)
- VS Code Copilot tool confirmations use a chevron to expand tool details before approval. (Microsoft)

Design notes
- Use a compact “tool pill” summary (tool name + short argument summary) with a disclosure to reduce visual noise.
- Consider a distinct visual treatment for tool calls vs normal assistant text (background tint, icon, or label).

## 2) Approval flows (pre-action)

Patterns
- Require explicit user confirmation for write/modify actions; allow read-only actions to run with lighter friction.
- Show the data to be sent in the confirmation prompt and provide Allow/Deny choices.
- Offer “remember approval” or “always allow” for low-risk operations (typically reads), but disable it for consequential writes.
- Allow users to edit tool parameters before executing the action.

Evidence/examples
- ChatGPT developer mode requires confirmation for write actions and lets users remember approvals per conversation. (OpenAI)
- ChatGPT MCP connectors require manual confirmation for write tools, with tool-call payloads shown. (OpenAI)
- Microsoft 365 Copilot shows confirmation prompts with the data to be sent and Allow/Decline; “Always allow” typically appears for GET, not for POST/PATCH/PUT/DELETE; `x-openai-isConsequential` controls this. (Microsoft)
- VS Code Copilot lets users expand tool details and edit input parameters before selecting Allow. (Microsoft)

Design notes
- Use clear, action-specific confirmation text (e.g., “Create ticket in Jira?” vs “Proceed?”).
- If you allow approval memory, scope it to the conversation/session and make it easy to reset.

## 3) Action buttons and controls

Patterns
- Provide explicit approval/deny buttons in the tool confirmation step.
- Expose quick controls to expand details, edit parameters, and reset saved approvals.
- Offer “Show output” / “Show terminal” or equivalent toggles for inspecting tool results.

Evidence/examples
- VS Code Copilot shows an “Allow” action after parameter review and provides “Show Output” and “Show Terminal” for command results. (Microsoft)
- VS Code Copilot supports resetting saved tool approvals. (Microsoft)

Design notes
- Keep action buttons adjacent to the tool call card, and use color/labels to indicate risk.
- Provide a short “what will happen” subtext under the primary action when risk is high.

## 4) Result display (post-action)

Patterns
- Display tool results as their own block in the conversation (distinct from assistant text), then follow with a user-friendly summary.
- Support rich result payloads (text, images, documents) and explicit error state flags.
- Confirm action completion with a structured result card for consequential actions.

Evidence/examples
- Claude tool use expects a tool_result block that can contain text, images, or documents and can be marked as error. (Anthropic)
- Gemini tool flow returns a final user-friendly response grounded in tool results. (Google)
- Microsoft Teams Copilot validation requires completion confirmation in a card for action scenarios. (Microsoft)

Design notes
- Keep result blocks close to the initiating tool call and visually link them (e.g., via timeline or nested indentation).
- For failures, surface the error reason plus a next-step action (retry, edit params, or cancel).

## 5) Cross-cutting UX principles

- Transparency: expose tool call inputs/outputs and tool names so users can audit behavior.
- Control: require confirmation for consequential actions; allow parameter edits and approval reset.
- Traceability: keep tool calls, confirmations, and results in a readable step-by-step flow.
- Safety defaults: default to manual approval for writes and make auto-approve explicit and reversible.

## Sources

OpenAI
- https://platform.openai.com/docs/guides/developer-mode
- https://developers.openai.com/apps-sdk/deploy/connect-chatgpt
- https://platform.openai.com/docs/guides/tools

Microsoft
- https://code.visualstudio.com/docs/copilot/chat/chat-tools
- https://learn.microsoft.com/it-it/microsoft-365-copilot/extensibility/api-plugin-confirmation-prompts
- https://learn.microsoft.com/en-us/microsoftteams/platform/concepts/deploy-and-publish/appsource/prepare/review-copilot-validation-guidelines

Anthropic
- https://platform.claude.com/docs/en/agents-and-tools/tool-use/implement-tool-use

Google
- https://ai.google.dev/gemini-api/docs/tools
