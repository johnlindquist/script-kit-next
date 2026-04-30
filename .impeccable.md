## Design Context

### Users
Power developers and automation enthusiasts who demand speed and precision. They invoke Script Kit as a launcher/command palette — it must appear instantly, respond to keystrokes without lag, and disappear the moment the task is done. The interface should evoke **confidence**: every interaction feels deliberate, fast, and under their control.

### Brand Personality
**Fast. Focused. Minimal.**

Script Kit is a sharp tool, not a playground. It respects the user's time and attention. No unnecessary animation, no visual noise, no chrome that doesn't earn its place. The gold accent (#fbbf24) is the one warm touch — a signature that says "this is Script Kit" without shouting.

### Aesthetic Direction
- **Reference:** Raycast — clean launcher with macOS vibrancy, polished transitions, keyboard-first interaction, information-dense but not cluttered
- **Anti-references:** Electron apps with visible latency, over-decorated dashboards, anything that feels like a web page pretending to be native. Hover-dependent UIs that hide functionality behind mouse discovery.
- **Theme:** Dark mode primary with native macOS vibrancy (popover blur). Semi-transparent backgrounds let the desktop bleed through. Light mode supported but secondary
- **Visual tone:** Native macOS feel — if Apple made a scriptable launcher, it would look like this. Precision over personality

### Design Principles

1. **Three keys, nothing more** — The footer shows at most three affordances: Run (Enter), Actions (⌘K), AI (Tab). If it doesn't fit in three slots, it belongs in the Actions dialog, not the chrome. This applies universally across all windows and surfaces.

2. **Discovery lives in Actions** — Features, commands, and contextual operations are discoverable through the Actions dialog (⌘K), not through persistent chrome, hover states, or tooltips. The main surface stays clean; ⌘K is the single entry point for "what can I do here?"

3. **Peek, don't clutter** — For list-only surfaces, detail lives behind ⌘I (info/peek). Press to see, Esc to return. No inline expansion, no hover cards, no progressive disclosure on mouse. Exception: when the preview IS the experience (clipboard content, file preview, live theme swatch), a split panel is justified — see Surface Layouts below.

4. **Whisper chrome** — UI surfaces use ultra-low opacity (0.03–0.06 at rest). Borders are hairline or absent. Backgrounds are barely perceptible. Content gets full opacity; everything else fades to near-invisible. Let vibrancy and spacing define structure, not boxes and dividers.

5. **Speed is the design** — Every pixel serves instant comprehension. If an element slows the user down (visually or mechanically), remove it. Sub-frame response to input is non-negotiable.

6. **Keyboard-first, always** — The mouse is a fallback. Every interaction must be reachable and obvious via keyboard. Visual affordances reinforce keyboard shortcuts, not compete with them.

7. **Native or nothing** — Respect macOS conventions. Vibrancy, system fonts, PopUp panel behavior, proper focus/unfocus dimming. Users should forget they're in a third-party app.

### Text Opacity Tiers (theme-configurable)

All text elements use `text_primary` (white on dark, black on light) as the base color. Brightness is controlled purely via these semantic opacity tiers — no double-dimming from secondary/muted/dimmed base colors. These live on `BackgroundOpacity` and are configurable per-theme.

| Tier | Default | Theme field | Use |
|------|---------|-------------|-----|
| **Name** | 1.0 | `text_name` | Names, primary labels — always full brightness |
| **Strong** | 0.80 | `text_strong` | Badges, shortcuts, section headers — clearly readable chrome |
| **Muted** | 0.65 | `text_muted_alpha` | Focused descriptions, source hints — readable but secondary |
| **Hint** | 0.45 | `text_hint` | Hovered descriptions, type labels — visible but recessive |
| **Placeholder** | 0.40 | `text_placeholder` | Placeholders, idle captions, header hints |
| **Icon** | 0.50 | `text_icon` | Idle icons — recedes behind name text |

### Background Opacity Tiers

| Tier | Range | Use |
|------|-------|-----|
| **Ghost** | 0.03–0.06 | Row hover bg, whisper surfaces, dividers |
| **Subtle** | 0.15–0.20 | Row selection bg, subtle highlights |
| **Medium** | 0.40–0.55 | Input backgrounds, panels |
| **Solid** | 0.75–1.0 | Window surfaces, dialogs, footers |

### Interaction State Ladder

Hover, focus, and active/toggle states use a 3-tier luminance ladder. Each tier uses a different base color and opacity so states are always visually distinct — even over vibrancy blur.

| State | Base color | Opacity | Purpose |
|-------|-----------|---------|---------|
| **Idle** | — | 0% | Transparent — no background |
| **Hover** | `text.primary` | 0.06 (bg) | Ghost-tier row tint; text grading provides the real affordance |
| **Focused / Selected** | `text.primary` | 0.20 (bg) | Subtle row highlight; description grades up to muted opacity |
| **Active / Toggle** | Same as Focused | Same | Pressed/toggled buttons match focused style exactly |

**Raycast-style text grading (the real differentiation):**

| Text element | Idle | Hovered | Focused | Tier |
|-------------|------|---------|---------|------|
| **Name** | 100% | 100% | 100% | `text_name` |
| **Description** | hidden | 45% | 65% | `text_hint` / `text_muted_alpha` |
| **Icon** | 50% | 50% | 100% | `text_icon` |
| **Source hint** | hidden | 45% | 45% | `text_hint` |
| **Section header** | 80% | — | — | `text_strong` |
| **Badge / shortcut** | 80% | 80% | 80% | `text_strong` |
| **Placeholder** | 40% | — | — | `text_placeholder` |

**Rules:**
- All text uses `text_primary` as the base color. Opacity alone controls brightness.
- Row bg stays ghost/subtle so vibrancy shows through; text grading carries the visual weight.
- Active/toggle states match focused style. Hovering over active keeps active style.
- Chrome tokens `hover_rgba` and `selection_rgba` resolve from `text.primary` in `src/theme/chrome.rs`.

### List Item Anatomy

**Unfocused row:** Icon + name at full white. Right-aligned metadata in hint opacity. No description. No borders. No row dividers.

**Hovered row:** Name stays full white. Description appears at 45% opacity (hint tier). Ghost-tier bg tint (0.06). Clear step up from idle but clearly lighter than focused.

**Focused row:** Gold left-bar accent (#fbbf24). Name full white. Description at 65% opacity (muted tier). Subtle bg highlight (0.20). Right-aligned metadata tags in muted opacity.

**Section headers:** Uppercase label at strong tier (80%). Count and icon at muted tier (65%). No separator lines — spacing alone defines groups.

**Footer:** Exactly `↵ Run · ⌘K Actions · Tab AI`. Hint opacity. Right-aligned. Nothing else.

### Surface Layouts

#### The Decision Rule

**Ask: "Is the list item the content, or a label pointing at content?"**

- If the name IS the thing (a script, an app, a process, an emoji) → **Mini view**. The list item is self-selecting. ⌘I shows configuration, metadata, settings — useful but not required to choose.
- If the name is a LABEL for content you can't see (a clipboard entry, a file, a theme, a window) → **Expanded view**. The list item is meaningless without its preview. You can't confidently select without seeing what it points to.

#### Mini View (list-only + ⌘I info)

Use when the list item name is sufficient to make the right selection. ⌘I info shows configuration and metadata — things you might want to know but don't need to choose.

| Surface | Why mini works | ⌘I info shows |
|---------|---------------|---------------|
| Main menu | "Summarize" is self-explanatory | Description, shortcut, script path, last run |
| App launcher | "Slack" is obvious | App version, path, bundle ID |
| Process manager | Process name + PID is enough | Memory, uptime, CPU, logs |
| Favorites | Script name tells you what it does | Same as main menu info |
| AI presets | Preset name describes behavior | Full system prompt, model, parameters |
| Emoji picker | Grid layout — visual content scanned by shape, not name | (see Grid layout below) |

#### Expanded View (list + preview split)

Use when the list item is a label pointing at content that must be seen to select correctly. The preview IS the decision — removing it would force blind selection.

| Surface | Why expanded is required | Preview shows |
|---------|------------------------|---------------|
| Clipboard history | "Text clip from 2m ago" could be anything | Full text/image content |
| File search | Filename alone can't distinguish similar files | File content, image thumbnail |
| Theme chooser | "Nordic Frost" is meaningless without colors | Live color preview, the theme applied |
| Window switcher | Multiple "Safari" windows need differentiation | Window thumbnail |

**Rules for the expanded split:**
- List side follows mini list anatomy (icon + name, gold bar, no row dividers)
- Preview side is chromeless — content flush, no wrapping borders or headers
- Divider between panels: hairline or spacing only
- Footer still follows three-key pattern
- No additional chrome around either panel

#### The Litmus Test

> If you deleted the preview panel and a user said "I can still pick the right one" → it's a mini view.
> If you deleted the preview panel and a user said "I'm guessing now" → it's an expanded view.

#### Other Layouts

**Editor** (code editor prompt): Justified exception — the editor surface needs its full area. Footer simplifies to three-key hint strip.

**Grid** (emoji picker, icon browsers): Grids are the correct layout when content is inherently visual and high-density — emoji, icons, images. You don't scan emoji by name, you scan by shape. The grid stays. Apply mini chrome to the surrounding shell (bare input, hint strip footer, no heavy borders) but keep the grid itself.

### Actions Dialog (⌘K)

The actions dialog is the single discovery surface. It must feel like a natural extension of the main list — same visual language, not a separate UI system.

**Container:** No rounded corners on the dialog frame. Match the main window's sharp edge treatment. The dialog is a panel, not a modal card.

**Row anatomy:** Same as main list — action name left, shortcut glyphs right-aligned. No row dividers. Focused row gets gold bar + ghost-opacity background. Destructive actions use red text + red-tinted shortcut glyphs.

**Shortcut glyphs:** Use the separated key-cap style (individual ⌘ ⇧ K boxes), NOT the inline text style (^⇧K) from the main menu. But render them **smaller** — they should be secondary to the action name, not competing with it. Target hint opacity for the glyph background, muted opacity for the glyph text. The glyphs are a reference, not a call to action.

**Section headers:** Uppercase category labels (Actions, Edit, Share, Destructive) in hint opacity. No separator lines — spacing alone defines groups, same as main list section headers.

**Search input:** Bare input at top, no border, gold cursor. Same treatment as main menu input.

### Interaction Model

| Key | Action | Scope |
|-----|--------|-------|
| **Enter** | Run / Submit / Confirm | Universal |
| **⌘K** | Open Actions dialog | Universal — the single discovery surface for all contextual operations |
| **Tab** | AI chat | Universal |
| **⌘I** | Info / Peek | Universal — toggles detail view, Esc to return |
| **Esc** | Back / Dismiss | Universal — returns from peek, closes dialogs, dismisses window |
