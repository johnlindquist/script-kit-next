# Task: Replace Hardcoded Colors with Theme Tokens

**Priority:** HIGH  
**Estimated Effort:** 2-3 hours  
**Skill Reference:** `script-kit-theme`

---

## Problem Description

Production code contains 100+ instances of hardcoded `rgb(0x...)` colors instead of using theme tokens. This breaks:

1. **Theme consistency**: Colors don't match user's theme
2. **Dark/light mode**: Hardcoded colors look wrong in opposite mode
3. **Focus-aware styling**: Can't dim when window loses focus
4. **Accessibility**: Users can't customize for visibility needs

---

## Affected Files (Production Code)

| File | Line | Current Code | Should Be |
|------|------|--------------|-----------|
| `src/render_prompts/arg.rs` | 53 | `rgb(0xffffff)` | `rgb(colors.text.primary)` |
| `src/render_script_list.rs` | 316 | `rgb(0xB85C00)` | `rgb(colors.ui.warning)` |
| `src/app_shell/shell.rs` | 233 | `rgb(0x000000)` | `rgb(colors.background.main)` |
| `src/app_shell/shell.rs` | 265 | `rgba(0x00000080)` | `with_alpha(colors.background.main, 0.5)` |
| `src/app_shell/shell.rs` | 372 | `rgb(0x000000)` | `rgb(colors.background.main)` |
| `src/theme/helpers.rs` | 131 | `rgb(0x00ffff)` | `rgb(colors.accent.selected)` |
| `src/editor.rs` | 1009 | `rgb(0xffffff)` | `rgb(colors.text.primary)` |
| `src/components/prompt_header.rs` | 526 | `rgb(0x000000)` | Theme-aware color |
| `src/prompts/env.rs` | 235 | `rgb(0xffffff)` | `rgb(colors.text.primary)` |

---

## Solution

### Step 1: Understand the Theme Structure

The theme provides colors via `theme.colors`:

```rust
pub struct ColorScheme {
    pub background: BackgroundColors,  // main, title_bar, search_box, log_panel
    pub text: TextColors,              // primary, secondary, tertiary, muted, dimmed
    pub accent: AccentColors,          // selected, selected_subtle
    pub ui: UIColors,                  // border, success, error, warning, info
    pub terminal: TerminalColors,      // ANSI colors
}
```

### Step 2: Fix Each Location

#### Location 1: `src/render_prompts/arg.rs:53`

**Find:**
```rust
.text_color(rgb(0xffffff))
```

**Replace with:**
```rust
.text_color(rgb(self.theme.colors.text.primary))
```

#### Location 2: `src/render_script_list.rs:316`

**Find:**
```rust
rgb(0xB85C00)  // Warning orange
```

**Replace with:**
```rust
rgb(self.theme.colors.ui.warning)
```

#### Location 3-5: `src/app_shell/shell.rs:233, 265, 372`

**Find:**
```rust
.bg(rgb(0x000000))
// or
.bg(rgba(0x00000080))
```

**Replace with:**
```rust
// For solid black background
.bg(rgb(self.theme.colors.background.main))

// For semi-transparent overlay
.bg(with_alpha(self.theme.colors.background.main, 0.5))
```

You may need to add a helper function:
```rust
fn with_alpha(color: u32, alpha: f32) -> Hsla {
    let r = ((color >> 16) & 0xFF) as f32 / 255.0;
    let g = ((color >> 8) & 0xFF) as f32 / 255.0;
    let b = (color & 0xFF) as f32 / 255.0;
    // Convert RGB to HSL and set alpha
    hsla(h, s, l, alpha)
}
```

Or use the existing `hex_to_hsla` and modify alpha:
```rust
let mut color = hex_to_hsla(self.theme.colors.background.main);
color.a = 0.5;
.bg(color)
```

#### Location 6: `src/theme/helpers.rs:131`

**Find:**
```rust
rgb(0x00ffff)  // Cyan cursor
```

**Replace with:**
```rust
rgb(colors.accent.selected)
```

Note: The `helpers.rs` file creates lightweight color structs, so pass the accent color through.

#### Location 7: `src/editor.rs:1009`

**Find:**
```rust
.text_color(rgb(0xffffff))
```

**Replace with:**
```rust
.text_color(rgb(self.theme.colors.text.primary))
```

#### Location 8: `src/components/prompt_header.rs:526`

**Find:**
```rust
rgb(0x000000)  // Black for logo inside yellow
```

This may be intentional for the logo. If so, consider:
```rust
// If this is for contrast on accent background:
rgb(colors.text.on_accent)  // If available
// Or compute contrasting color dynamically
```

#### Location 9: `src/prompts/env.rs:235`

**Find:**
```rust
.text_color(rgb(0xffffff))
```

**Replace with:**
```rust
.text_color(rgb(design_colors.text_on_accent))
// or
.text_color(rgb(self.theme.colors.text.primary))
```

### Step 3: Find All Remaining Hardcoded Colors

Run this search to find all remaining instances:

```bash
# Find rgb(0x...) patterns
grep -rn "rgb(0x" src/ --include="*.rs" | grep -v test | grep -v stories | grep -v storybook

# Find rgba patterns
grep -rn "rgba(0x" src/ --include="*.rs" | grep -v test | grep -v stories | grep -v storybook

# Find hex patterns in hsla
grep -rn "hsla.*0x" src/ --include="*.rs" | grep -v test | grep -v stories | grep -v storybook
```

### Step 4: Create Mapping Reference

When replacing, use this mapping:

| Hardcoded | Theme Token |
|-----------|-------------|
| `0xffffff` (white) | `colors.text.primary` |
| `0x000000` (black) | `colors.background.main` |
| `0x1e1e1e` (dark gray) | `colors.background.main` |
| `0xcccccc` (light gray) | `colors.text.secondary` |
| `0x808080` (medium gray) | `colors.text.muted` |
| `0xfbbf24` (gold/yellow) | `colors.accent.selected` |
| `0x3b82f6` (blue) | `colors.ui.info` |
| `0xef4444` (red) | `colors.ui.error` |
| `0x22c55e` (green) | `colors.ui.success` |
| `0xf59e0b` (orange) | `colors.ui.warning` |

### Step 5: Handle Design Variant Colors

Some prompts use `DesignVariant`. Use design colors when available:

```rust
let tokens = get_tokens(self.design_variant);
let design_colors = tokens.colors();

// Use design_colors.background, design_colors.text_primary, etc.
```

---

## Verification Steps

1. **Build Check:**
   ```bash
   cargo check
   ```

2. **Lint Check:**
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```

3. **Test:**
   ```bash
   cargo test
   ```

4. **Visual Testing - Theme Consistency:**
   ```bash
   cargo build
   echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
   ```
   - Check that all text is readable
   - Check that colors match the theme
   - Look for any stark white or black elements that stand out

5. **Visual Testing - Create Screenshot Test:**
   ```typescript
   // tests/smoke/test-theme-colors.ts
   import '../../scripts/kit-sdk';
   import { writeFileSync, mkdirSync } from 'fs';
   import { join } from 'path';
   
   await div(`
     <div class="flex flex-col gap-2 p-4">
       <div class="text-primary">Primary Text</div>
       <div class="text-secondary">Secondary Text</div>
       <div class="bg-accent p-2">Accent Background</div>
       <div class="border border-ui p-2">Border Color</div>
     </div>
   `);
   
   await new Promise(r => setTimeout(r, 500));
   const shot = await captureScreenshot();
   const dir = join(process.cwd(), 'test-screenshots');
   mkdirSync(dir, { recursive: true });
   writeFileSync(join(dir, 'theme-colors.png'), Buffer.from(shot.data, 'base64'));
   process.exit(0);
   ```

---

## Success Criteria

- [ ] No `rgb(0xffffff)` in production code (excluding tests/stories)
- [ ] No `rgb(0x000000)` in production code (excluding tests/stories)
- [ ] All colors use theme tokens or design colors
- [ ] UI looks consistent with theme
- [ ] `cargo check && cargo clippy && cargo test` passes

---

## Lower Priority (Stories/Storybook)

The following files have hardcoded colors but are lower priority since they're for development/testing:

- `src/storybook/browser.rs` - 30+ hardcoded colors
- `src/stories/*.rs` - Multiple hardcoded colors

These can be fixed later or left as-is for rapid prototyping.

---

## Related Files

- `src/theme/types.rs` - Theme struct definitions
- `src/theme/helpers.rs` - Lightweight color structs
- `src/theme/mod.rs` - Theme module exports
