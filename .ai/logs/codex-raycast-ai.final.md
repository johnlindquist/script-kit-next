# Raycast AI + Productivity Research (Comprehensive)

<!-- markdownlint-disable MD013 -->

Last verified: **2026-02-07** using Raycast public docs/manual/pricing pages.

## 1) Raycast AI (built-in chat, models, pricing)

### What Raycast AI includes

- **Quick AI** for one-off queries from root search (`Tab` to switch).
- **AI Chat** as a dedicated chat window.
- **AI Commands** (built-in and custom prompt workflows).
- **AI Extensions** with `@mentions` to invoke extension capabilities in AI contexts.

### Models available

- Raycast exposes a large multi-provider model catalog and updates it frequently.
- Providers shown in public model docs include: **OpenAI, Anthropic, Google, Perplexity, Mistral, Meta, xAI, Qwen, DeepSeek, Moonshot, Groq/Together-hosted models**.
- The developer API docs also enumerate many model IDs and fallback behavior if a selected model is unavailable.

### AI pricing/limits (as documented)

- Free users get **50 free AI messages** to try AI.
- Pro is required for sustained/unlimited Raycast-hosted AI usage (subject to request limits).
- Documented request limits for paid AI tiers include baseline per-minute/per-hour limits and additional per-24h/per-week caps for some advanced models.
- Advanced AI is an add-on tier in pricing, with broader model access.
- BYOK is supported (Anthropic/Google/OpenAI/OpenRouter), with caveats documented in the AI manual.

## 2) AI Commands (custom AI prompts, prompt templates)

### Built-in + custom commands

- Raycast ships built-in AI commands (e.g., writing/explain workflows).
- You can duplicate built-ins and create custom commands from scratch.
- Translation is explicitly demonstrated as a custom AI command pattern.

### Prompt variables/templates

- AI commands support dynamic placeholders such as:
  - `{selection}`
  - `{argument ...}`
  - `{clipboard}`
- Dynamic placeholder docs show argument support and modifiers (`uppercase`, `lowercase`, `trim`, etc.).

### Prompt template ecosystems

- **Prompt Explorer (`ray.so/prompts`)** provides prompt templates that can be selected and **added to Raycast as AI Commands**.
- Raycast manual also includes import guides for AI Commands and AI Chat Presets.

## 3) AI Extensions (how extensions use AI)

### End-user behavior

- Users can invoke AI extensions via `@` mentions across AI surfaces (root/Quick AI/AI commands/chat contexts).
- AI extensions are designed for natural-language interaction with extension tools.

### Developer model

- You convert extensions into AI extensions by adding **tools** (function-like capabilities) with descriptions.
- Core concepts in docs: **Tools, Instructions, Evals**.
- AI extension docs emphasize tool input schemas, optional confirmations, and evaluation-driven testing.

### API specifics

- Raycast AI API provides direct AI access without requiring the extension developer to manage external API keys.
- Access check is explicit: `environment.canAccess(AI)`.
- If user lacks Pro access, the call path prompts for access and can error.
- `AI.ask(...)` supports options for model/creativity and supports streaming (`data` events) plus abort via `AbortSignal`.

## 4) Translator feature

- Pricing table includes **Translator** as a plan feature.
- Raycast AI docs show translation as a first-class AI command workflow:
  - “Translate {selection} to English”
  - “Translate {selection} to {argument name="Language"}”
- Prompt Explorer includes translation prompt templates (e.g., language argument-driven translation prompts).

## 5) Focus / Do Not Disturb mode

- Raycast Focus exposes automation Shortcuts:
  - **Start Focus Session**
  - **Complete Focus Session**
- Raycast docs explicitly mention combining Focus Shortcuts with **Do Not Disturb** in workflows.
- Focus Filter integration lets Raycast Focus session start with system Focus and supports blocked app/site categories.
- Related productivity: Auto Quit Applications closes configured inactive apps to reduce distractions.

## 6) Raycast Pro features (free vs paid)

### Free baseline (documented)

- Core launcher + productivity features are available in free plan.
- Free plan includes limited AI trial usage and limited Notes count.

### Paid/Pro (documented)

- Pricing and feature grid list paid features including:
  - **Cloud Sync**
  - **Custom Themes**
  - **Translator**
  - **Unlimited Notes** (vs 5 on free)
  - **Custom Window Management Commands**
- Pricing page documents Pro and team annual/monthly prices and AI add-on pricing.

### Notable pricing snapshots captured

- Pro: `$10/month` or `$8/month annual`.
- Team Pro: `$15/user/month` or `$12/user/month annual`.
- Advanced AI add-on: `+$8/month` personal / `+$8/user/month` team.

## 7) Team / organization features

- Teams documentation describes:
  - Create organization + handle
  - Private extension store for org members
  - Invite workflows via org invite links
  - Private extension publishing/distribution
- Pricing/docs indicate team collaboration features:
  - Shared Commands
  - Shared Quicklinks
  - Shared Snippets
  - Private Store
- Enterprise controls shown in pricing docs include:
  - AI Control Center (org AI toggle, BYOK, provider allow-list)
  - Admin controls (SAML/SCIM, domain capture, full cloud sync control, 2FA enforcement, extensions allow-list)
- Free/Team plans are stated as usable in organizational/corporate environments.

## 8) Cloud sync (settings/snippets/extensions across devices)

- Cloud Sync requires Pro.
- Cloud Sync docs state synchronization across Macs for:
  - Search history, aliases, hotkeys
  - Quicklinks, snippets
  - Notes
  - Extensions + extension settings
  - AI chats/presets/commands
  - Themes
- Not synced:
  - Clipboard history
  - Script commands
  - Credentials/passwords
  - General/Advanced settings

## 9) Raycast Notes (standalone notes feature)

- Notes are a built-in note-taking experience with markdown-oriented editing and fast command-palette access.
- Core commands:
  - `Raycast Notes` (toggle notes window)
  - `Create Note`
  - `Search Notes`
- Notes support structured markdown formatting, task lists, pinning, and keyboard-driven navigation.
- Pricing grid indicates **5 notes on free** and **unlimited on paid tiers**.

## 10) Calculator features

- Raycast calculator is always available from root search.
- Supports:
  - Math expressions
  - Unit/currency conversion
  - Crypto conversion
  - Date/time expressions
  - Timezone conversion (e.g., `5pm london in sf`)
- Natural-language parsing is documented as a key capability.

## 11) Color picker

- Raycast Store features a widely used **Color Picker** extension (system-wide picking + conversion tooling).
- Documented capabilities include:
  - Pick color from desktop
  - Organize colors
  - Convert color formats
  - Menu bar access to recent picks
- This appears as extension-based functionality rather than a current standalone “core feature” page in the manual.

## 12) Other productivity features (not exhaustive)

From pricing/manual/core links, additional productivity surface includes:

- Clipboard History
- Snippets
- Quicklinks
- Window Management (+ custom window commands on paid tiers)
- Emoji Picker
- Calendar/system commands
- Deeplinks (`raycast://...`) for automation
- Auto Quit Applications (focus support)
- AI Prompt/Chat preset import workflows

---

## Sources

### Raycast AI, commands, models

- <https://manual.raycast.com/ai>
- <https://manual.raycast.com/ai-extensions>
- <https://manual.raycast.com/dynamic-placeholders>
- <https://manual.raycast.com/ai/how-to-import-ai-commands>
- <https://manual.raycast.com/ai/how-to-import-ai-chat-presets>
- <https://www.raycast.com/core-features/ai/models>
- <https://ray.so/prompts>

### Developer AI + AI extensions

- <https://developers.raycast.com/api-reference/ai>
- <https://developers.raycast.com/ai/create-an-ai-extension>
- <https://developers.raycast.com/ai/learn-core-concepts-of-ai-extensions>
- <https://developers.raycast.com/ai/write-evals-for-your-ai-extension>
- <https://developers.raycast.com/ai/follow-best-practices-for-ai-extensions>

### Pricing, plans, teams

- <https://www.raycast.com/pricing>
- <https://www.raycast.com/teams>
- <https://developers.raycast.com/teams/getting-started>
- <https://developers.raycast.com/teams/publish-a-private-extension>

### Focus, Notes, Cloud Sync, Calculator, Quicklinks, automation, color picker

- <https://manual.raycast.com/focus/how-to-create-a-shortcut>
- <https://manual.raycast.com/focus/how-to-create-a-focus-filter>
- <https://manual.raycast.com/auto-quit-applications>
- <https://manual.raycast.com/notes>
- <https://manual.raycast.com/cloud-sync>
- <https://manual.raycast.com/windows/calculator>
- <https://manual.raycast.com/quicklinks>
- <https://manual.raycast.com/deeplinks>
- <https://www.raycast.com/thomas/color-picker>
