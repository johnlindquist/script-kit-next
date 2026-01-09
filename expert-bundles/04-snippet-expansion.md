# Snippet/Expansion Logic Expert Bundle

## Original Goal

> Improving the snippet/expansion logic for when it replaces text
>
> This is the original task description that prompted the creation of this bundle.

## Executive Summary

Script Kit supports text expansion where typing a trigger (e.g., ";;email") expands to full text. The expansion system uses keyboard monitoring to detect triggers and text injection to replace them. Issues may occur with timing, selection handling, or special characters.

### Key Problems:
1. **Text selection before replacement** - May not correctly select trigger text
2. **Timing issues** - Fast typing may interrupt expansion
3. **Special character handling** - Unicode or modifier keys may break expansion

### Required Fixes:
1. **src/expand_manager.rs** - Fix text selection and replacement logic
2. **src/expand_matcher.rs** - Improve trigger detection accuracy
3. **src/text_injector.rs** - Ensure clean text injection without artifacts
4. **src/snippet.rs** - Handle template variable substitution correctly

### Files Included:
- `src/expand_manager.rs`: Manages expansion detection and execution
- `src/expand_matcher.rs`: Matches typed text against triggers
- `src/snippet.rs`: Snippet formatting and variable substitution
- `src/text_injector.rs`: Injects text via keyboard simulation
- `src/scriptlets.rs`: Scriptlet parsing and metadata
- `src/scriptlet_metadata.rs`: Scriptlet metadata extraction

---

