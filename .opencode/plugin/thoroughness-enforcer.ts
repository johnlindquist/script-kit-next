import type { Plugin } from "@opencode-ai/plugin"
import { logTriggered, logSkipped } from "../lib/logger"

const PLUGIN_NAME = "thoroughness-enforcer"

/**
 * Thoroughness Enforcer Plugin (OpenCode port)
 *
 * Automatically detects when users expect thorough, comprehensive work
 * and enforces task completion before allowing the agent to stop.
 *
 * Original: https://github.com/johnlindquist/thoroughness-enforcer
 *
 * When a user's prompt contains thoroughness indicators like:
 * - High confidence: "exhaustive", "thorough", "comprehensive", "deep dive", "ultrathink"
 * - Medium confidence: "all", "every", "each", "entire", "complete", "fully"
 * - Contextual: "make sure", "ensure", "verify all", "check all"
 *
 * The plugin:
 * 1. Captures the prompt and analyzes it for thoroughness indicators
 * 2. Injects context reminding the agent about thoroughness expectations
 * 3. Blocks stopping until the agent verifies completeness (up to 3 review checkpoints)
 * 4. Generates review prompts tailored to the original request
 */

// =============================================================================
// Analyzer - Detects thoroughness indicators in prompts
// =============================================================================

const DEFAULT_INSTRUCTION_WORD_LIMIT = 250

function countWords(text: string): number {
  const trimmed = text.trim()
  if (!trimmed) return 0
  return trimmed.split(/\s+/).filter(Boolean).length
}

interface InstructionExtractionResult {
  text: string
  shouldAnalyze: boolean
  reason: "short_prompt" | "short_after_separator" | "too_long"
}

/**
 * Extracts the instruction portion of a prompt for thoroughness analysis
 *
 * Logic:
 * - If the full prompt is < maxWords, analyze the full prompt
 * - If >= maxWords, look for "---" markdown separator
 *   - If found and text after last "---" is < maxWords, analyze that portion
 *   - Otherwise, skip analysis (likely a context dump)
 */
function extractInstructions(
  prompt: string,
  maxWords: number = DEFAULT_INSTRUCTION_WORD_LIMIT
): InstructionExtractionResult {
  const totalWords = countWords(prompt)

  if (totalWords < maxWords) {
    return { text: prompt, shouldAnalyze: true, reason: "short_prompt" }
  }

  const separatorPattern = /^---+$/m
  const parts = prompt.split(separatorPattern)

  if (parts.length > 1) {
    const lastPart = parts[parts.length - 1]!.trim()
    const lastPartWords = countWords(lastPart)

    if (lastPartWords < maxWords && lastPartWords > 0) {
      return { text: lastPart, shouldAnalyze: true, reason: "short_after_separator" }
    }
  }

  return { text: "", shouldAnalyze: false, reason: "too_long" }
}

interface AnalysisResult {
  isThorough: boolean
  matchedIndicators: string[]
  confidence: "high" | "medium" | "low"
  originalPrompt: string
}

/**
 * Thoroughness indicators categorized by strength
 */
const INDICATORS = {
  high: [
    "exhaustive", "thorough", "thoroughly", "comprehensive", "comprehensively",
    "in-depth", "in depth", "deep dive", "leave no stone unturned",
    "don't miss anything", "do not miss anything", "cover everything",
    "be thorough", "be comprehensive", "complete analysis", "full analysis",
    "ultrathink", "think hard", "think deeply",
  ],
  medium: [
    "all", "every", "each", "entire", "whole", "complete", "completely",
    "fully", "full", "everything", "everywhere", "anywhere", "nothing left",
    "no exceptions", "without exception", "across the board",
    "end to end", "end-to-end", "100%", "100 percent",
  ],
  low: [
    "make sure", "ensure", "verify", "check all", "scan", "audit",
    "review all", "fix all", "update all", "refactor all",
    "test all", "cover all", "handle all",
  ],
}

const AMPLIFIERS = [
  "really", "absolutely", "definitely", "certainly", "must",
  "need to", "have to", "make sure to", "don't forget", "do not forget",
  "important", "critical", "crucial",
]

const DIMINISHERS = [
  "all i need", "all you need", "just need", "only need",
  "simple", "quick", "briefly", "just", "only",
]

function analyzePrompt(prompt: string): AnalysisResult {
  const normalizedPrompt = prompt.toLowerCase()
  const matchedIndicators: string[] = []
  let highMatches = 0
  let mediumMatches = 0
  let lowMatches = 0

  const hasDiminisher = DIMINISHERS.some((d) => normalizedPrompt.includes(d))
  const hasAmplifier = AMPLIFIERS.some((a) => normalizedPrompt.includes(a))

  for (const indicator of INDICATORS.high) {
    if (normalizedPrompt.includes(indicator)) {
      matchedIndicators.push(indicator)
      highMatches++
    }
  }

  for (const indicator of INDICATORS.medium) {
    const pattern = indicator.length <= 4
      ? new RegExp(`\\b${indicator}\\b`, "i")
      : new RegExp(indicator, "i")

    if (pattern.test(normalizedPrompt)) {
      matchedIndicators.push(indicator)
      mediumMatches++
    }
  }

  for (const indicator of INDICATORS.low) {
    if (normalizedPrompt.includes(indicator)) {
      matchedIndicators.push(indicator)
      lowMatches++
    }
  }

  let confidence: "high" | "medium" | "low" = "low"

  if (highMatches > 0) {
    confidence = "high"
  } else if (mediumMatches >= 2 || (mediumMatches >= 1 && hasAmplifier)) {
    confidence = "medium"
  } else if (mediumMatches >= 1 || lowMatches >= 2) {
    confidence = "low"
  }

  if (hasDiminisher && confidence !== "low") {
    confidence = confidence === "high" ? "medium" : "low"
  }

  if (hasAmplifier && !hasDiminisher && confidence !== "high" && matchedIndicators.length > 0) {
    confidence = confidence === "low" ? "medium" : "high"
  }

  const isThorough = matchedIndicators.length > 0 && confidence !== "low"

  return { isThorough, matchedIndicators, confidence, originalPrompt: prompt }
}

function generateReviewPrompt(analysis: AnalysisResult): string {
  const indicators = analysis.matchedIndicators.slice(0, 5).join('", "')

  return `The user's original request contained thoroughness indicators: "${indicators}"

Original request:
"${analysis.originalPrompt.slice(0, 500)}${analysis.originalPrompt.length > 500 ? "..." : ""}"

Before stopping, verify:

1. **Completeness Check**
   - Have you addressed ALL aspects of the request?
   - Are there any items, files, or cases you haven't covered?
   - Did you handle edge cases?

2. **Quantifier Verification**
   - The user used words like "${indicators}" - did you truly cover everything implied?
   - List what you DID cover vs what might be missing

3. **Quality Check**
   - Is the work thorough, not just "good enough"?
   - Would the user be satisfied this covers EVERYTHING they asked for?

If anything is incomplete, continue working. If truly complete, you may stop.`
}

// =============================================================================
// Session State
// =============================================================================

interface SessionState {
  activePrompt: string
  analysis: AnalysisResult
  capturedAt: number
  promptCount: number
  stopDenialCount: number
  analysisPromptNumber: number
  thoroughnessHistory: Array<{
    promptNumber: number
    timestamp: number
    confidence: "high" | "medium" | "low"
    indicators: string[]
  }>
}

const MAX_STOP_DENIALS = 3

function createEmptyAnalysis(): AnalysisResult {
  return {
    isThorough: false,
    matchedIndicators: [],
    confidence: "low",
    originalPrompt: "",
  }
}

function createSessionState(): SessionState {
  return {
    activePrompt: "",
    analysis: createEmptyAnalysis(),
    capturedAt: Date.now(),
    promptCount: 0,
    stopDenialCount: 0,
    analysisPromptNumber: 0,
    thoroughnessHistory: [],
  }
}

// =============================================================================
// Plugin Export
// =============================================================================

// Session-keyed state map to support parallel sessions
const sessions = new Map<string, SessionState>()

function getState(sessionId: string): SessionState {
  let state = sessions.get(sessionId)
  if (!state) {
    state = createSessionState()
    sessions.set(sessionId, state)
  }
  return state
}

function clearState(sessionId: string): void {
  sessions.delete(sessionId)
}

// Type for extracting session ID from various hook inputs
interface WithSessionId {
  sessionID?: string
  session_id?: string
}

function extractSessionId(input: unknown): string | null {
  const obj = input as WithSessionId
  return obj?.sessionID || obj?.session_id || null
}

const ThoroughnessEnforcer: Plugin = async ({ client }) => {
  return {
    // Track session events and user messages
    event: async ({ event }) => {
      // Extract session ID from event
      const eventWithSession = event as { session_id?: string; sessionID?: string; properties?: { session_id?: string } }
      const sessionId = eventWithSession.session_id || 
                        eventWithSession.sessionID || 
                        eventWithSession.properties?.session_id

      if (event.type === "session.created" && sessionId) {
        sessions.set(sessionId, createSessionState())
        logTriggered(sessionId, PLUGIN_NAME, "event", "Session created - initialized thoroughness tracking")
        return
      }
      
      if (event.type === "session.deleted" && sessionId) {
        clearState(sessionId)
        logTriggered(sessionId, PLUGIN_NAME, "event", "Session deleted - cleared thoroughness tracking")
        return
      }

      // Detect user messages for thoroughness analysis
      if (event.type === "message.updated") {
        const props = event as { 
          properties?: { 
            message?: { role?: string; content?: unknown }
            session_id?: string 
          } 
        }
        const message = props.properties?.message
        const msgSessionId = sessionId || props.properties?.session_id
        
        if (message?.role === "user" && msgSessionId) {
          const state = getState(msgSessionId)
          const content = typeof message.content === "string" 
            ? message.content 
            : JSON.stringify(message.content || "")
          
          state.promptCount++
          
          const extraction = extractInstructions(content)
          
          if (extraction.shouldAnalyze) {
            const analysis = analyzePrompt(extraction.text)
            
            if (analysis.isThorough) {
              const confidenceOrder = { high: 3, medium: 2, low: 1 }
              const shouldUpdate = !state.analysis.isThorough ||
                confidenceOrder[analysis.confidence] >= confidenceOrder[state.analysis.confidence]
              
              if (shouldUpdate) {
                state.activePrompt = content
                state.analysis = analysis
                state.analysisPromptNumber = state.promptCount
                state.stopDenialCount = 0
                state.capturedAt = Date.now()
                
                state.thoroughnessHistory.push({
                  promptNumber: state.promptCount,
                  timestamp: Date.now(),
                  confidence: analysis.confidence,
                  indicators: analysis.matchedIndicators,
                })
                
                logTriggered(msgSessionId, PLUGIN_NAME, "event", `Detected thoroughness (${analysis.confidence})`, {
                  indicators: analysis.matchedIndicators.slice(0, 5),
                  promptNumber: state.promptCount
                })
              }
            } else {
              logSkipped(msgSessionId, PLUGIN_NAME, "event", "User message analyzed - no thoroughness indicators")
            }
          } else {
            logSkipped(msgSessionId, PLUGIN_NAME, "event", `Prompt too long to analyze (${extraction.reason})`)
          }
        }
        return
      }
      
    },

    // Block stop if thoroughness requirements not met
    stop: async (input) => {
      const sessionId = extractSessionId(input)
      if (!sessionId) {
        logSkipped(null, PLUGIN_NAME, "stop", "No session ID available")
        return
      }
      
      const state = getState(sessionId)
      
      // No thoroughness detected - allow stop
      if (!state.analysis.isThorough) {
        logSkipped(sessionId, PLUGIN_NAME, "stop", "Stop allowed - no thoroughness requirements active")
        return
      }

      // Safety valve: After MAX_STOP_DENIALS, allow stop
      if (state.stopDenialCount >= MAX_STOP_DENIALS) {
        logTriggered(sessionId, PLUGIN_NAME, "stop", `Stop allowed - max denials reached (${MAX_STOP_DENIALS})`)
        clearState(sessionId)
        return
      }

      // Increment denial count
      state.stopDenialCount++
      const denialCount = state.stopDenialCount

      logTriggered(sessionId, PLUGIN_NAME, "stop", `BLOCKING STOP: Thoroughness check ${denialCount}/${MAX_STOP_DENIALS}`, {
        confidence: state.analysis.confidence,
        indicators: state.analysis.matchedIndicators.slice(0, 3)
      })

      // Generate review prompt
      const reviewPrompt = generateReviewPrompt(state.analysis)

      let denialMessage: string

      if (denialCount === 1) {
        denialMessage = `## THOROUGHNESS CHECK (Attempt ${denialCount}/${MAX_STOP_DENIALS})

${reviewPrompt}

This is your first review checkpoint. Take time to verify your work is complete.`
      } else if (denialCount === 2) {
        denialMessage = `## THOROUGHNESS CHECK (Attempt ${denialCount}/${MAX_STOP_DENIALS})

You've attempted to stop again. Let's be more specific:

${reviewPrompt}

**Required before stopping:**
1. Re-read the original request word by word
2. List each requirement and confirm it's addressed
3. Check for any "all", "every", "each" quantifiers - did you handle them ALL?
4. Run tests or verification if applicable

If you've truly covered everything, explain what you verified in your next response.`
      } else {
        denialMessage = `## FINAL THOROUGHNESS CHECK (Attempt ${denialCount}/${MAX_STOP_DENIALS})

This is your final verification. After this, you will be allowed to stop.

${reviewPrompt}

**Final checklist:**
- [ ] All files/items mentioned are handled
- [ ] All edge cases considered
- [ ] All error conditions handled
- [ ] Work has been tested/verified
- [ ] Nothing from the original request is missing

Make this final pass count. Verify or explain what you may have missed.`
      }

      // Prompt the agent to continue (target specific session)
      await client.session.prompt({
        path: { id: sessionId },
        body: {
          parts: [{ type: "text", text: denialMessage }],
        },
      })
    },

    // Add thoroughness context to system prompt when active
    "experimental.chat.system.transform": async (input, output) => {
      const sessionId = extractSessionId(input)
      if (!sessionId) {
        logSkipped(null, PLUGIN_NAME, "system.transform", "No session ID available")
        return
      }
      
      const state = getState(sessionId)
      
      if (state.analysis.isThorough) {
        const indicators = state.analysis.matchedIndicators.slice(0, 5).join('", "')
        
        output.system.push(`<thoroughness-enforcer confidence="${state.analysis.confidence}">
Thoroughness indicators detected in user request: "${indicators}"

This request expects comprehensive, thorough work. The system will verify completeness before allowing task completion.

Key expectations:
- Cover ALL cases, not just common ones
- Handle edge cases and exceptions
- Verify work is complete, not just "done"
- The stop action will be blocked until you demonstrate thoroughness (up to ${MAX_STOP_DENIALS} review checkpoints)

Stop denial count: ${state.stopDenialCount}/${MAX_STOP_DENIALS}
</thoroughness-enforcer>`)
        logTriggered(sessionId, PLUGIN_NAME, "system.transform", `Injected thoroughness context (${state.analysis.confidence})`, {
          stopDenials: state.stopDenialCount
        })
      } else {
        logSkipped(sessionId, PLUGIN_NAME, "system.transform", "No active thoroughness requirements")
      }
    },

    // Preserve state through compaction
    "experimental.session.compacting": async (input, output) => {
      const sessionId = extractSessionId(input)
      if (!sessionId) {
        logSkipped(null, PLUGIN_NAME, "session.compacting", "No session ID available")
        return
      }
      
      const state = getState(sessionId)
      
      if (state.analysis.isThorough) {
        const indicators = state.analysis.matchedIndicators.slice(0, 5).join('", "')
        output.context.push(`<thoroughness-state>
Active thoroughness enforcement (${state.analysis.confidence} confidence)
Indicators: "${indicators}"
Stop denials: ${state.stopDenialCount}/${MAX_STOP_DENIALS}
Original request (truncated): "${state.activePrompt.slice(0, 200)}..."
</thoroughness-state>`)
        logTriggered(sessionId, PLUGIN_NAME, "session.compacting", "Preserved thoroughness state", {
          confidence: state.analysis.confidence,
          stopDenials: state.stopDenialCount
        })
      } else {
        logSkipped(sessionId, PLUGIN_NAME, "session.compacting", "No thoroughness state to preserve")
      }
    },
  }
}

export default ThoroughnessEnforcer
