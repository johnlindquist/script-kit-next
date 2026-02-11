# Prompt Footer Component Audit

## Scope
- `src/components/prompt_footer.rs`
- Focus: fixed-height layout, padding, truncation behavior, button spacing, hover/active cues, vibrancy/opacity.

## Current Footer Contract (Observed)
- Height is hard-locked via `.h/.min_h/.max_h(px(FOOTER_HEIGHT))` with overflow clipping (`src/components/prompt_footer.rs:493`, `src/components/prompt_footer.rs:494`, `src/components/prompt_footer.rs:495`, `src/components/prompt_footer.rs:497`).
- Horizontal padding is `12px`; bottom optical padding is `2px` (`src/components/prompt_footer.rs:54`, `src/components/prompt_footer.rs:56`, `src/components/prompt_footer.rs:498`, `src/components/prompt_footer.rs:500`).
- Left side (`logo + helper`) is `flex_1` with overflow clipping (`src/components/prompt_footer.rs:518`, `src/components/prompt_footer.rs:521`).
- Right side (`info + actions`) is always rendered as a non-flex action cluster (`src/components/prompt_footer.rs:435`, `src/components/prompt_footer.rs:457`, `src/components/prompt_footer.rs:485`).
- Helper text truncates at `420px`; info label truncates at `220px` (`src/components/prompt_footer.rs:38`, `src/components/prompt_footer.rs:40`, `src/components/prompt_footer.rs:445`, `src/components/prompt_footer.rs:534`).

## Findings: Where Layout Can Surprise Users

### 1) Height documentation drift vs runtime contract
- Evidence:
  - Doc comment says `Height: 40px fixed` (`src/components/prompt_footer.rs:302`).
  - Runtime uses `FOOTER_HEIGHT` token (`src/components/prompt_footer.rs:493`).
  - `FOOTER_HEIGHT` is currently `30.0` (`src/window_resize/mod.rs:229`).
- User impact:
  - Designers and implementers can size wrappers for 40px and end up with clipping/tight vertical rhythm.

### 2) Right-side action cluster can starve left helper area
- Evidence:
  - Left side is shrinkable `flex_1` (`src/components/prompt_footer.rs:519`), while right side has no equivalent shrink priority and always includes buttons (`src/components/prompt_footer.rs:485`).
  - Buttons have no truncation rules on labels/shortcuts (`src/components/prompt_footer.rs:372`, `src/components/prompt_footer.rs:381`).
- User impact:
  - Long action labels crowd out helper text first, even when helper contains current-step guidance.

### 3) Truncation priorities are implicit and asymmetric
- Evidence:
  - Helper and info are ellipsized (`src/components/prompt_footer.rs:447`, `src/components/prompt_footer.rs:536`).
  - Primary/secondary labels never truncate and are not width-capped (`src/components/prompt_footer.rs:372`, `src/components/prompt_footer.rs:462`, `src/components/prompt_footer.rs:476`).
- User impact:
  - Low-priority labels (e.g. verbose button text) can consume space while higher-value context gets clipped.

### 4) Button spacing expands more than token names imply
- Evidence:
  - Container gap is `4px` (`src/components/prompt_footer.rs:44`, `src/components/prompt_footer.rs:457`).
  - Divider adds its own `mx(4px)` (`src/components/prompt_footer.rs:66`, `src/components/prompt_footer.rs:419`).
- User impact:
  - Inter-button distance around the divider appears larger than normal action spacing, which can read as inconsistent grouping.

### 5) Clickability cues depend on callback presence, not just visual state
- Evidence:
  - Clickable only when callback exists and not disabled (`src/components/prompt_footer.rs:151`, `src/components/prompt_footer.rs:355`, `src/components/prompt_footer.rs:356`).
  - If callback is missing but `disabled == false`, button keeps normal styling with no hover/active state (`src/components/prompt_footer.rs:384`, `src/components/prompt_footer.rs:389`).
- User impact:
  - Controls can look “available” but provide no interaction feedback or action, creating inconsistent affordances.

### 6) Hard-coded label suppression couples behavior to string values
- Evidence:
  - `"Built-in"` info label is hidden by content check (`src/components/prompt_footer.rs:82`, `src/components/prompt_footer.rs:170`).
  - `"Run Command"` primary button is hidden by content check (`src/components/prompt_footer.rs:84`, `src/components/prompt_footer.rs:176`).
- User impact:
  - Localization or copy changes can unexpectedly show/hide footer elements, causing layout jumps and unclear action sets.

### 7) Vibrancy handling differs sharply between light and dark paths
- Evidence:
  - Light mode uses opaque neutral (`0xf0eeefff`) (`src/components/prompt_footer.rs:136`).
  - Dark mode uses low-alpha overlay from accent subtle token (`src/components/prompt_footer.rs:139`).
- User impact:
  - Contrast and separation behavior differ by theme in ways that can make button hover/active feedback feel uneven across themes.

## Canonical Footer Slots (Proposed)

```
| leading_brand | leading_context ............. | trailing_meta | trailing_actions |
```

- `leading_brand`:
  - Optional logo/icon only.
  - Fixed footprint; never truncates.
- `leading_context`:
  - Optional helper/instruction text.
  - Single line with ellipsis.
  - Yields space in this order: trailing_meta -> leading_context; trailing_actions stays intact.
- `trailing_meta`:
  - Optional low-priority metadata (language, count, mode).
  - Always truncates before any action text.
- `trailing_actions`:
  - Required action cluster.
  - Contains primary action and optional secondary action.
  - Action visibility is config-driven, not label-driven.

## Integration Guidance

1. Treat footer actions as explicit slots, not inferred labels.
- Use booleans/config fields for visibility.
- Avoid sentinel string behavior for hiding elements.

2. Enforce affordance consistency.
- If an action has no callback, either hide it or mark disabled so it visually communicates non-interactivity.
- Ensure enabled actions always have pointer + hover + active feedback.

3. Define a deterministic truncation order.
- First truncate/hide `trailing_meta`.
- Then truncate `leading_context`.
- Keep `trailing_actions` stable to preserve muscle memory for primary/secondary targets.

4. Normalize spacing ownership.
- Keep either container `gap` or divider `mx` as the source of horizontal rhythm, not both.
- This avoids “extra” spacing around the divider that can imply unintended grouping.

5. Align theme behavior for vibrancy/opacity intent.
- Decide whether footer should consistently block vibrancy in both themes or consistently blend in both themes.
- Keep hover/active contrast perceptually equivalent across light and dark paths.

6. Keep labels concise at call sites.
- Primary/secondary labels should be short enough to avoid starving helper text.
- Put volatile context in `leading_context` or `trailing_meta`, not long action labels.
