# AI Chat Prompt Templates and Quick Actions Research

Goal: summarize external patterns around prompt templates, quick actions, slash commands, and prompt suggestions, then translate them into concrete ideas for the Script Kit AI chat window.

## Key patterns observed

- Prompt templates often support variables/placeholders that render form inputs or selects. ChatKit uses double curly braces with optional option lists, e.g. `{{tone|Friendly,Sarcastic,Formal}}`. [1]
- Template systems add structure and governance: versions, model settings, system message, variables, and A/B testing are typical features to manage quality and consistency. [2]
- Prompt libraries are exposed in multiple entry points (profile menu, AI chat button, "View more") to keep templates discoverable in-context. [7]
- Prompt libraries can also be opened with `/` in the chat input, and may include community prompts in addition to user-created ones. [6]
- Quick actions are commonly surfaced as header buttons or side panels (settings, prompts, compositions). [3]
- Prompt suggestions can be invoked with a dedicated trigger like `@` and navigated with arrow keys. [3]
- Slash commands often differ from "quick commands": slash commands gather extra input; quick commands run immediately without extra input. [5]
- Slash command UIs benefit from fast keyboard flows: `/` opens a modal/list, arrow keys or Tab navigate, Enter selects, and `/command + Enter` is a speed path. [4]

## Template systems (what to support)

### Library, discovery, and invocation

- Provide a prompt library UI; add search and categories/tags as it scales (inference). [7]
- Allow `/` to open the prompt library or template list directly in the input. [1][6]
- Offer a dedicated button in the chat header for quick access to prompt templates. [3][7]

### Template structure and governance

Based on common prompt template systems, a template record should include:

- Metadata: name, description, labels/tags. [2]
- Content: system message + user message(s) with placeholders for variables. [2]
- Variables: named placeholders, optional select options, and default values. [1][2]
- Model settings: default model, temperature, max tokens, etc. [2]
- Versions: ability to iterate, compare, and promote a production version. [2]

## Quick actions (UI patterns)

- Header quick action buttons for settings, prompts, and other tools increase discoverability and reduce clicks. [3]
- A quick actions panel or menu (especially on smaller screens) is a proven pattern. [3]
- Prompt suggestions can be triggered with `@` and navigated with arrow keys to insert a template quickly. [3]

## Slash commands (command palette behavior)

- Slash commands work best for flows that require additional user input (arguments). [5]
- Quick commands work best for one-tap actions that can run immediately. [5]
- A `/`-activated command list should allow arrow-key navigation and direct `/command` entry. [4]
- When a command requires arguments, show a small inline form (variables) before running. [1][2]

## Suggestions for the Script Kit AI chat window

### Interaction model

- Implement two command types:
  - Quick actions: immediate actions (e.g. "Summarize", "Explain", "Create ticket") with no extra input. [5]
  - Slash commands: actions that require arguments (e.g. `/summarize 3 bullets`, `/rewrite tone=polite`). [5]
- Support `/` to open the template list (optionally with search/categories) and allow `/command + Enter` for power users. (search/categories are inference) [1][4][6]
- Support `@` to open prompt suggestions, with arrow key navigation and Enter to insert. [3]

### Quick actions placement

- Header buttons for Settings, Prompt Library, and other tools mirror established chat UI patterns. [3]
- Consider a side panel or overflow menu to keep the input area uncluttered. [3]

### Template UX

- When a template is selected, render a mini form for variables (text or select). (inference from placeholder patterns) [1][2]
- Store and expose version history for each template; allow marking an "active" version. [2]
- Provide a minimal template manager to create/edit templates with tags and descriptions. [2]

### Suggested default quick actions (inference)

Based on common usage in chat tools:

- Explain selection
- Summarize and extract actions
- Draft response
- Debug error
- Generate tests

(These are inferred recommendations, not directly specified by the sources.)

## 10 prompt templates (draft)

Each template uses ChatKit-style placeholder syntax to map fields into a form. [1]

1) Explain selection

Template:
```
You are a senior engineer. Explain the following content to a {{audience|Beginner,Intermediate,Expert}} audience.
Use a {{style|Concise,Detailed}} style and include 1-3 key takeaways.

Content:
{{input}}
```

2) Summarize with action items

Template:
```
Summarize the following into {{bullets|3,5,7}} bullets and list any action items separately.
If you see unknowns, add a "Questions" section.

Content:
{{input}}
```

3) Debug error log

Template:
```
You are a debugging assistant. Analyze the error log and propose the top {{count|3,5}} likely causes.
Provide a step-by-step fix plan with commands or code changes.

Environment:
{{environment|macOS,Linux,Windows}}

Error log:
{{log}}
```

4) Generate minimal reproduction

Template:
```
Given the bug description, propose a minimal reproduction with:
1) Steps
2) Expected vs actual
3) Smallest code snippet or script

Bug description:
{{input}}
```

5) Refactor plan

Template:
```
Create a refactor plan with {{phases|2,3,4}} phases.
For each phase, list goals, files to touch, and risks.

Context:
{{input}}
```

6) Write tests

Template:
```
Generate tests for the following change. Include both happy path and edge cases.
Target framework: {{framework|rust,vitest,jest,pytest}}

Change summary:
{{input}}
```

7) Convert to Script Kit SDK usage

Template:
```
Rewrite the following script to use the Script Kit SDK and the stdin JSON protocol where relevant.
Keep behavior the same, and add comments only if necessary.

Script:
{{input}}
```

8) AI chat response template (support)

Template:
```
Draft a support response in a {{tone|Friendly,Professional,Direct}} tone.
Include: summary, next steps, and a question to confirm resolution.

User message:
{{input}}
```

9) Performance investigation

Template:
```
Review the logs and propose performance bottlenecks.
Return:
- Suspected hotspots
- Metrics to add
- Quick wins vs longer-term fixes

Logs:
{{input}}
```

10) Release note draft

Template:
```
Create release notes for the following changes.
Format:
- Highlights (3 bullets)
- Fixes
- Known issues (if any)

Changes:
{{input}}
```

## References

1. ChatKit Prompt Templates (placeholders + slash command): https://docs.chatkit.app/prompt-templates
2. UsageGuard Prompt Templates (versions, variables, model settings, system message): https://docs.usageguard.com/features/prompt-templates
3. PromptBlocks Chat Interface (header quick actions, quick actions panel, @ prompt suggestions): https://www.promptblocks.app/docs/chat
4. ClickUp Slash Commands (slash modal + keyboard navigation + /command speed path): https://help.clickup.com/hc/en-us/articles/6308960837911-Use-Slash-Commands
5. Google Chat Commands (slash vs quick commands; input vs immediate): https://developers.google.com/workspace/chat/commands
6. ChatHub Prompt Library (prompt library + / access + community prompts): https://doc.chathub.gg/features/prompt-library
7. Taskade AI Prompt Templates & Library (multiple access points in AI chat): https://help.taskade.com/en/articles/8958452-ai-prompt-templates-library
