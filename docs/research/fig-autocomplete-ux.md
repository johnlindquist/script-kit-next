# Fig (Amazon Q Developer CLI) Autocomplete UX Research

This document captures UX patterns and design decisions from Fig (now Amazon Q Developer CLI) that are relevant to Script Kit's autocomplete and suggestion systems.

## Overview

Fig (acquired by AWS and rebranded as Amazon Q Developer CLI) pioneered IDE-style autocomplete for terminal environments. The system provides context-aware suggestions for 500+ CLI tools through a combination of dropdown menus and inline ghost text suggestions.

**Key Sources:**
- [withfig/autocomplete GitHub Repository](https://github.com/withfig/autocomplete)
- [Fig User Manual](https://fig.io/user-manual/autocomplete)
- [Amazon Q Developer CLI Documentation](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line.html)

---

## Two Distinct Suggestion Modes

Fig implements two independent suggestion mechanisms that can work together or separately:

### 1. Dropdown/Popup Menu

A graphical dropdown appears below or beside the cursor showing available completions.

**Characteristics:**
- Appears automatically as the user types
- Positioned relative to cursor using Accessibility API (macOS)
- Shows icons, names, and descriptions for each suggestion
- Supports keyboard navigation (arrow keys)
- Selection via Tab or Enter

**Visual Elements:**
- Icon (left side) - indicates suggestion type
- Name/displayName - primary text
- Description - secondary text, often truncated
- Type indicators via icon style

### 2. Inline Ghost Text

Gray "ghost text" appears directly on the command line showing a potential completion.

**Characteristics:**
- Appears as dimmed/gray text inline with cursor
- Based on shell history and current context
- Accept with Right Arrow or Tab
- Dismiss by continuing to type
- Can be enabled/disabled independently from dropdown

**Key Insight:** These two modes serve different purposes:
- Dropdown: Exploring available options, discovery
- Ghost text: Quick completion of familiar/recent commands

---

## Suggestion Types and Icons

Fig uses a type system to categorize suggestions and automatically assign appropriate icons:

| Type | Description | Default Icon Behavior |
|------|-------------|----------------------|
| `subcommand` | CLI subcommands (e.g., `git checkout`) | Command-specific icon |
| `option` | Flags and options (e.g., `--force`) | Flag icon |
| `arg` | Arguments to commands | Contextual |
| `file` | File paths | System file icon |
| `folder` | Directory paths | System folder icon |
| `special` | Special suggestions | Custom |
| `mixin` | User-defined suggestions | Custom |
| `shortcut` | Keyboard shortcuts | Key icon |

### Icon API

Fig provides a rich icon system via the `fig://` URL scheme:

```
fig:///path/to/file          # System icon for file
fig://~/Desktop              # Tilde expansion
fig://icon?type=txt          # Extension-based icon
fig://icon?type=folder       # Generic folder icon
fig://template?color=2ecc71&badge=checkmark  # Templated icon with badge
```

**Built-in Icons:** alert, android, apple, asterisk, aws, azure, box, carrot, characters, command, commandkey, commit, database, docker, firebase, gcloud, git, github, gitlab, gradle, heroku, invite, kubernetes, netlify, node, npm, option, package, slack, string, twitter, vercel, yarn (30+ total)

---

## Suggestion Object Properties

Each suggestion is defined by these key properties:

```typescript
interface Suggestion {
  name?: string | string[];      // Filterable name(s)
  displayName?: string;          // Text shown in UI
  insertValue?: string;          // Value inserted on selection
  description?: string;          // Shown below/beside suggestion
  icon?: string;                 // Icon URL, emoji, or fig:// URL
  type?: SuggestionType;         // Determines default icon
  priority?: number;             // 0-100, higher = appears first
  hidden?: boolean;              // Only show on exact match
  isDangerous?: boolean;         // Disables auto-execute
  deprecated?: boolean;          // Shows deprecation styling
}
```

### Insert Value Features

The `insertValue` property supports special characters:
- `\n` - Insert newline
- `\b` - Backspace
- `{cursor}` - Position cursor after insertion

---

## Keyboard Navigation

### Dropdown Navigation
| Key | Action |
|-----|--------|
| Up/Down Arrow | Navigate suggestions |
| Tab | Insert selected suggestion |
| Enter | Insert and potentially execute |
| Escape | Dismiss dropdown |
| Continue typing | Filter suggestions |

### Ghost Text
| Key | Action |
|-----|--------|
| Right Arrow | Accept suggestion |
| Tab | Accept suggestion |
| Any other key | Ignore and continue typing |

---

## Completion Specs Architecture

Fig uses declarative TypeScript schemas called "completion specs" to define CLI tool behavior:

```typescript
const completionSpec: Fig.Spec = {
  name: "git",
  description: "The content tracker",
  subcommands: [
    {
      name: "checkout",
      description: "Switch branches or restore files",
      args: {
        name: "branch",
        generators: branchGenerator,  // Dynamic suggestions
      },
      options: [
        {
          name: ["-b", "--branch"],
          description: "Create new branch",
          args: { name: "new-branch" }
        }
      ]
    }
  ]
};
```

### Key Concepts

1. **Static Definitions:** Subcommands and options defined declaratively
2. **Dynamic Generators:** Arguments generated at runtime via shell commands
3. **Templates:** Prebuilt generators for common patterns (files, folders)
4. **Contextual Generators:** Suggestions based on other flags in the command

---

## Dynamic Suggestions (Generators)

Generators enable context-aware suggestions by running shell commands:

```typescript
const branchGenerator: Fig.Generator = {
  script: "git branch --format='%(refname:short)'",
  postProcess: (output) => {
    return output.split('\n').map(branch => ({
      name: branch.trim(),
      icon: "fig://icon?type=git",
      description: "Local branch"
    }));
  }
};
```

### Generator Types (in order of customizability)

1. **Templates:** Prebuilt for files/folders
2. **Script as String:** Simple shell command
3. **Script Function:** Function returning command string
4. **Custom Generator:** Full control over execution

### Contextual Generators

Generate suggestions based on other parts of the current command:

```typescript
// For `cd`, combines CWD with typed path to suggest directories
const cdGenerator = {
  custom: async (tokens, executeShellCommand) => {
    const path = tokens[tokens.length - 1];
    const result = await executeShellCommand(`ls ${path}`);
    // ... process and return suggestions
  }
};
```

---

## Terminal Integration

### How Fig Tracks Input

Fig evolved through multiple approaches:

1. **Early MVP:** CGEventTap keylogger (deprecated - brittle)
2. **ZSH Integration:** Direct edit buffer API access
3. **figterm (Primary):** Pseudoterminal layer between shell and terminal
4. **Rust Rewrite:** Performance-optimized version using tokio

### figterm Architecture

- **ANSI Escape Codes:** Invisible markers injected into prompt
- **Screen Grid:** Annotated representation of terminal cells
- **Cell Types:** prompt, suggestion, output, edit buffer
- **Privacy:** Only sees what terminal emulator sees (passwords via sudo invisible)

### Shell Integration

Fig modifies shell configuration files:
- `.bashrc`, `.zshrc`, `.zprofile`, `.profile`, `.bash_profile`
- Sources `fig.sh` on every new shell session
- Uses ZSH hooks (zle-line-init, zle-keymap-select) for real-time updates

### Accessibility API (macOS)

- Positions dropdown window relative to cursor
- Reads terminal content when shell API unavailable
- Required for VSCode integrated terminal support

---

## Theming and Customization

### Theme Commands

```bash
q theme dark       # Dark theme
q theme light      # Light theme
q theme system     # Follow system preference
```

### Available Settings

| Setting | Description |
|---------|-------------|
| Keybindings | Custom keyboard shortcuts |
| Theme | Visual appearance |
| Dimensions | Dropdown width/height |
| Fuzzy search | Enable/disable fuzzy matching |
| Trailing space | Allow autocomplete with trailing space |
| Tab-only trigger | Only show on Tab press |

---

## Performance Considerations

### Design Principles

1. **Local Processing:** Generators run locally, no cloud round-trip for suggestions
2. **Lazy Loading:** Completion specs loaded on-demand per CLI tool
3. **Spec Caching:** Current spec remains loaded until different CLI invoked
4. **Rust Core:** Performance-critical paths in Rust (figterm rewrite)

### User-Reported Impact

- "Zero complaints as to how it works or its performance"
- Reduced ZSH plugin dependencies improved shell startup time
- "Feels a lot like using an IDE"

---

## Suggestions for Script Kit

Based on this research, here are recommendations for Script Kit's autocomplete implementation:

### 1. Dual-Mode Suggestions

Consider implementing both:
- **Dropdown panel:** For discovery and exploration
- **Inline hints:** For quick, familiar completions

### 2. Rich Suggestion Objects

Implement suggestion properties:
```rust
struct Suggestion {
    name: String,
    display_name: Option<String>,
    insert_value: Option<String>,
    description: Option<String>,
    icon: Option<Icon>,
    suggestion_type: SuggestionType,
    priority: u8,
    is_dangerous: bool,
}
```

### 3. Keyboard Navigation

Essential shortcuts:
- Arrow keys for navigation
- Tab/Enter for selection
- Escape to dismiss
- Type-to-filter

### 4. Dynamic Generators

Support context-aware suggestions via:
- Shell command execution
- File system queries
- Script-defined generators

### 5. Icon System

Implement a flexible icon system:
- Type-based defaults
- File extension icons
- Custom icon URLs
- Emoji support

### 6. Theme Integration

Use existing theme system for:
- Dropdown background/border
- Text colors (primary, secondary, muted)
- Selection highlight
- Icon coloring

### 7. Performance

- Load suggestion providers lazily
- Cache frequently-used completions
- Run generators asynchronously
- Debounce rapid input

### 8. Accessibility

- Ensure keyboard-only navigation
- Proper focus management
- Screen reader support for suggestions

---

## References

- [GitHub: withfig/autocomplete](https://github.com/withfig/autocomplete)
- [Fig User Manual: Autocomplete](https://fig.io/user-manual/autocomplete)
- [Fig Docs: Suggestion Reference](https://fig.io/docs/reference/suggestion)
- [Fig Docs: Icon API](https://fig.io/docs/reference/suggestion/icon-api)
- [Fig Docs: Creating Completion Specs](https://fig.io/docs/getting-started/first-completion-spec)
- [Fig Docs: Dynamic Suggestions](https://fig.io/docs/concepts/dynamic-suggestions)
- [Fig Blog: How Fig Knows What You Typed](https://fig.io/blog/post/how-fig-knows-what-you-typed)
- [Amazon Q Developer CLI Documentation](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line.html)
- [GitHub: aws/amazon-q-developer-cli-autocomplete](https://github.com/aws/amazon-q-developer-cli-autocomplete)
- [Autocomplete Deep Dive Article](https://www.blog.brightcoding.dev/2025/09/10/autocomplete-for-terminal-commands-a-deep-dive-into-figs-open-source-engine/)
