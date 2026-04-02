# Zed ACP vs Script Kit UI/UX Audit Index

**Completed:** 2026-04-01  
**Scope:** Comprehensive comparison of Zed's agent coordination panel with Script Kit's chat UI

---

## Documents

### 1. Zed ACP Audit (`zed-acp-audit.md`)
**Purpose:** Detailed inventory of every UX pattern in Zed's ACP agent panel  
**Audience:** Anyone needing to understand Zed's approach  
**Content:**
- 16 sections with code snippets and line numbers
- Visual ASCII diagrams for each major component
- Architecture patterns and performance considerations
- File path reference guide

**Key Sections:**
- Message rendering (user/assistant/thinking blocks)
- Input editor with expand icon (↗)
- Bottom toolbar (mode, model, controls)
- Streaming UI (elapsed time, token count)
- Tool calls (edit, execute, terminal)
- Permission & approval dialogs
- Thinking blocks (collapsible with gradient)
- Message queue UI
- Scroll behavior

**Note:** This is Zed's design for comprehensive agent workflows. Very detailed.

---

### 2. Script Kit Chat Audit (`script-kit-chat-audit.md`)
**Purpose:** Current state audit of Script Kit's ChatPrompt component  
**Audience:** Script Kit developers, maintainers  
**Content:**
- 24 sections analyzing current implementation
- Top-placed input (inverted UX philosophy)
- Conversation turn bundling
- Word-buffered streaming reveal
- Current limitations and what's NOT implemented
- Type system and state management

**Key Sections:**
- Architecture (input at top, messages below)
- Message & turn rendering
- Input field styling (full mode + mini mode)
- Footer toolbar (context-aware buttons)
- Streaming & reveal mechanism
- Built-in AI provider support
- Script generation mode
- What's missing (queue, thinking blocks, tool UI, permissions)

**Note:** Script Kit intentionally simpler. Optimized for chat + script generation.

---

### 3. Gap Analysis (`gap-analysis-zed-vs-script-kit.md`)
**Purpose:** Bridge the two audits with actionable insights  
**Audience:** Product decisions, roadmap planning  
**Content:**
- 10 major gaps with impact assessment
- Feasibility matrix (effort vs. value)
- Implementation priorities
- Design philosophy comparison
- Concrete recommendations

**Key Findings:**

| Gap | Impact | Feasibility | Recommendation |
|-----|--------|-------------|-----------------|
| Thinking blocks | High | Medium | **Add** (medium effort, high value) |
| Streaming feedback | Medium | Easy | **Add** (low effort, medium value) |
| Scroll buttons | Low | Easy | **Consider** (convenience) |
| Message queue | Medium | Hard | **Maybe** (batch automation) |
| Tool calls | High | Hard | **Out of scope** (parent responsibility) |
| Permissions | High | Hard | **Out of scope** (parent responsibility) |
| Activity bar | Medium | Hard | **Out of scope** (layout responsibility) |

**Priority Roadmap:**

**Week 1 (Quick Wins):**
1. Thinking blocks (collapsible with max-height + gradient)
2. Elapsed time indicator during generation

**Week 2:**
3. Scroll-to-recent-prompt button
4. Optional in-toolbar model selector

**Future (if needed):**
5. Message queue for batch automation

---

## Usage Guide

### For Understanding Zed's Pattern
→ Read **Zed ACP Audit** (sections 1-10 for overview)

### For Understanding Script Kit's Current State
→ Read **Script Kit Chat Audit** (sections 1-5 for overview)

### For Making Roadmap Decisions
→ Read **Gap Analysis** (Executive Summary + Implementation Priorities)

### For Implementing Thinking Blocks
→ Reference Zed ACP Audit section 5 (Thinking Blocks) + Gap Analysis section 1

### For Implementing Streaming Feedback
→ Reference Zed ACP Audit section 4 (Streaming UX) + Gap Analysis section 4

---

## Key Insights

### Design Philosophy
- **Zed:** Comprehensive agent coordination (IDE-integrated, deep tool support)
- **Script Kit:** Fast scriptable chat (modular, callback-based)
- **Neither is wrong** — they optimize for different audiences

### Top 3 Differences
1. **Input placement:** Zed bottom (standard), Script Kit top (inverted)
2. **Tool support:** Zed comprehensive, Script Kit delegated to parent
3. **Message bundling:** Zed separate, Script Kit turns (user+assistant together)

### Top 3 Improvements for Script Kit
1. **Thinking blocks** (high value, medium effort)
2. **Streaming feedback** (medium value, low effort)
3. **Scroll controls** (low value, low effort)

---

## File Structure

```
.claude/
├── AUDIT-INDEX.md (this file)
├── zed-acp-audit.md (Zed reference)
├── script-kit-chat-audit.md (Script Kit inventory)
└── gap-analysis-zed-vs-script-kit.md (Actionable roadmap)
```

---

## Questions?

Each document is self-contained with:
- Code snippets & file paths
- Line number references
- ASCII diagrams where helpful
- Clear section headings

Start with the Gap Analysis for quick decisions, then drill into the detailed audits as needed.

