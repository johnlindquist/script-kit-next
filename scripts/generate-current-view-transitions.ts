#!/usr/bin/env bun
/**
 * Generate the agent-readable inventory of direct `current_view` mutations.
 *
 * This is intentionally an inventory, not the final state machine. It makes the
 * remaining transition sites visible to agents while route owners are migrated
 * behind narrower APIs.
 *
 * Usage:
 *   bun scripts/generate-current-view-transitions.ts --write
 *   bun scripts/generate-current-view-transitions.ts --check
 *   bun scripts/generate-current-view-transitions.ts --stdout
 */

import { existsSync, readFileSync, readdirSync, statSync, writeFileSync } from "fs";
import { join, relative, resolve } from "path";

const PROJECT_ROOT = resolve(import.meta.dir, "..");
const OUTPUT_PATH = "docs/ai/contracts/current-view-transitions.json";
const SCHEMA_VERSION = 1;
const SOURCE_DIRS = [
  "src/app_actions",
  "src/app_execute",
  "src/app_impl",
  "src/main_entry",
  "src/main_sections",
  "src/prompt_handler",
];

type Operation = "assign" | "replace" | "transitionHelper";

interface TransitionHelperContract {
  inferredTarget: string;
  requiresManualReview: boolean;
  rekeysMainAutomationSurface?: boolean;
  focusTarget?: string;
  embeddedAiWindowUpsert?: boolean;
  clearsActionsPopupState?: boolean;
  transitionContract: TransitionContractMetadata;
}

interface TransitionContractMetadata {
  oldView: string;
  newView: string;
  surfaceKind?: string;
  semanticSurface?: string;
  mainAutomationRekey?: boolean;
  focusTarget?: string;
  focusCoordinatorRequest?: string;
  focusedInput?: string;
  focusedInputMap?: Record<string, string>;
  resize?: string;
  activePopupContract?: string;
  stateSnapshot?: string;
  delegatesTo?: string;
  embeddedAiWindowUpsert?: boolean;
  agent_chatSurfaceEvent?: string;
  actionsCleanup?: string;
}

const TRANSITION_HELPERS: Record<string, TransitionHelperContract> = {
  transition_current_view_and_rekey_main_automation_surface: {
    inferredTarget: "argument",
    requiresManualReview: true,
    rekeysMainAutomationSurface: true,
    transitionContract: {
      oldView: "runtimeCurrentView",
      newView: "dynamicArgument",
      surfaceKind: "derivedFromNewView",
      semanticSurface: "rekeyMainAutomationSurfaceFromCurrentView",
      mainAutomationRekey: true,
      focusTarget: "callerOwned",
      focusedInput: "callerOwned",
      resize: "callerOwned",
      activePopupContract: "stateReceiptOnly",
      stateSnapshot: "getState.surfaceContract",
    },
  },
  enter_embedded_agent_chat_surface: {
    inferredTarget: "AppView::AgentChatView",
    requiresManualReview: false,
    rekeysMainAutomationSurface: true,
    focusTarget: "AgentChat",
    embeddedAiWindowUpsert: true,
    clearsActionsPopupState: true,
    transitionContract: {
      oldView: "runtimeCurrentView",
      newView: "AppView::AgentChatView",
      surfaceKind: "AgentChat",
      embeddedAiWindowUpsert: true,
      mainAutomationRekey: true,
      agent_chatSurfaceEvent: "EmbeddedOpened",
      actionsCleanup: "clearActionsPopupState",
      focusTarget: "AgentChat",
      focusCoordinatorRequest: "FocusRequest::agent_chat",
      focusedInput: "None",
      resize: "callerOwned",
      stateSnapshot: "getState.surfaceContract",
    },
  },
  restore_current_view_with_focus: {
    inferredTarget: "argument",
    requiresManualReview: true,
    transitionContract: {
      oldView: "runtimeCurrentView",
      newView: "dynamicArgument",
      mainAutomationRekey: false,
      focusTarget: "dynamicArgument",
      focusedInputMap: {
        MainFilter: "MainFilter",
        ActionsDialog: "ActionsSearch",
        default: "None",
      },
      resize: "callerOwned",
      stateSnapshot: "getState.surfaceContract",
    },
  },
  show_script_list_with_main_filter_focus: {
    inferredTarget: "AppView::ScriptList",
    requiresManualReview: false,
    rekeysMainAutomationSurface: true,
    focusTarget: "MainFilter",
    transitionContract: {
      oldView: "runtimeCurrentView",
      newView: "AppView::ScriptList",
      delegatesTo: "restore_current_view_with_focus",
      focusTarget: "MainFilter",
      focusedInput: "MainFilter",
      mainAutomationRekey: true,
      semanticSurface: "rekeyMainAutomationSurfaceFromCurrentView",
      resize: "callerOwned",
      stateSnapshot: "getState.surfaceContract",
    },
  },
};

interface TransitionEntry {
  file: string;
  line: number;
  owner: string;
  receiver: "self" | "view";
  operation: Operation;
  expression: string;
  inferredTarget: string;
  requiresManualReview: boolean;
  helper?: string;
  rekeysMainAutomationSurface?: boolean;
  focusTarget?: string;
  embeddedAiWindowUpsert?: boolean;
  clearsActionsPopupState?: boolean;
  transitionContract?: TransitionContractMetadata;
}

interface CurrentViewTransitionInventory {
  schemaVersion: number;
  generatedFrom: string[];
  inventory: string;
  entries: TransitionEntry[];
}

function rustFiles(dir: string): string[] {
  const absolute = resolve(PROJECT_ROOT, dir);
  if (!existsSync(absolute)) {
    return [];
  }
  return readdirSync(absolute)
    .flatMap((name) => {
      const path = join(absolute, name);
      const stat = statSync(path);
      if (stat.isDirectory()) {
        return rustFiles(relative(PROJECT_ROOT, path));
      }
      return path.endsWith(".rs") ? [relative(PROJECT_ROOT, path)] : [];
    })
    .sort();
}

function lineNumber(source: string, index: number): number {
  return source.slice(0, index).split("\n").length;
}

function lineAt(source: string, line: number): string {
  return source.split("\n")[line - 1]?.trim() ?? "";
}

function maskRangePreservingLines(source: string, start: number, end: number): string {
  return `${source.slice(0, start)}${source
    .slice(start, end)
    .replace(/[^\n]/g, " ")}${source.slice(end)}`;
}

function maskCfgTestModules(source: string): string {
  let masked = source;
  let searchIndex = 0;
  const cfgTestModuleRegex = /#\[cfg\(test\)\]\s*mod\s+[A-Za-z0-9_]+\s*\{/g;
  while (true) {
    cfgTestModuleRegex.lastIndex = searchIndex;
    const match = cfgTestModuleRegex.exec(masked);
    if (!match) {
      return masked;
    }
    const cfgIndex = match.index;
    const openIndex = cfgTestModuleRegex.lastIndex - 1;

    let depth = 0;
    let endIndex = openIndex;
    for (; endIndex < masked.length; endIndex += 1) {
      const char = masked[endIndex] ?? "";
      if (char === "{") {
        depth += 1;
      } else if (char === "}") {
        depth -= 1;
        if (depth === 0) {
          endIndex += 1;
          break;
        }
      }
    }

    masked = maskRangePreservingLines(masked, cfgIndex, endIndex);
    searchIndex = endIndex;
  }
}

function collectExpression(lines: string[], startLineIndex: number, initial: string): string {
  const chunks = [initial.trim()];
  let balance = (initial.match(/\{/g)?.length ?? 0) - (initial.match(/\}/g)?.length ?? 0);
  let lineIndex = startLineIndex + 1;
  while (balance > 0 && lineIndex < lines.length) {
    const line = lines[lineIndex] ?? "";
    chunks.push(line.trim());
    balance += (line.match(/\{/g)?.length ?? 0) - (line.match(/\}/g)?.length ?? 0);
    lineIndex += 1;
  }
  return chunks
    .join(" ")
    .replace(/\s+/g, " ")
    .replace(/;\s*$/, "");
}

function inferTarget(expression: string): string {
  const appView = expression.match(/AppView::([A-Za-z0-9_]+)/);
  if (appView) {
    return `AppView::${appView[1]}`;
  }
  if (expression.startsWith("return_view")) {
    return "return_view";
  }
  if (expression.startsWith("previous")) {
    return "previous";
  }
  if (expression.startsWith("other")) {
    return "other";
  }
  if (expression.startsWith("view")) {
    return "view";
  }
  return "dynamic";
}

function ownerForLine(source: string, line: number): string {
  const lines = source.split("\n");
  for (let index = line - 1; index >= 0; index -= 1) {
    const text = lines[index]?.trim() ?? "";
    const fnMatch = text.match(/(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z0-9_]+)/);
    if (fnMatch) {
      return fnMatch[1] ?? "unknown";
    }
  }
  return "module";
}

function collectFileEntries(file: string): TransitionEntry[] {
  const source = maskCfgTestModules(readFileSync(resolve(PROJECT_ROOT, file), "utf8"));
  const lines = source.split("\n");
  const entries: TransitionEntry[] = [];
  const assignmentRegex = /\b(self|view)\.current_view\s*=/g;
  let match: RegExpExecArray | null;
  while ((match = assignmentRegex.exec(source)) !== null) {
    const line = lineNumber(source, match.index);
    const text = lineAt(source, line);
    const rhs = text.slice(text.indexOf("=") + 1);
    const expression = collectExpression(lines, line - 1, rhs);
    entries.push({
      file,
      line,
      owner: ownerForLine(source, line),
      receiver: (match[1] ?? "self") as "self" | "view",
      operation: "assign",
      expression,
      inferredTarget: inferTarget(expression),
      requiresManualReview: !expression.includes("AppView::"),
    });
  }

  const replaceRegex = /std::mem::replace\(&mut\s+(self|view)\.current_view,\s*([^)]+)\)/g;
  while ((match = replaceRegex.exec(source)) !== null) {
    const line = lineNumber(source, match.index);
    const expression = match[2]?.trim() ?? "";
    entries.push({
      file,
      line,
      owner: ownerForLine(source, line),
      receiver: (match[1] ?? "self") as "self" | "view",
      operation: "replace",
      expression,
      inferredTarget: inferTarget(expression),
      requiresManualReview: false,
    });
  }

  for (const [helper, contract] of Object.entries(TRANSITION_HELPERS)) {
    const helperRegex = new RegExp(`\\b(self|view)\\.${helper}\\(`, "g");
    while ((match = helperRegex.exec(source)) !== null) {
      const line = lineNumber(source, match.index);
      const callStart = source.indexOf("(", match.index);
      let depth = 0;
      let end = callStart;
      for (; end < source.length; end += 1) {
        const char = source[end] ?? "";
        if (char === "(" || char === "{" || char === "[") {
          depth += 1;
        } else if (char === ")" || char === "}" || char === "]") {
          depth -= 1;
          if (depth === 0) {
            end += 1;
            break;
          }
        }
      }
      const expression = source
        .slice(callStart + 1, end - 1)
        .trim()
        .replace(/\s+/g, " ");
      entries.push({
        file,
        line,
        owner: ownerForLine(source, line),
        receiver: (match[1] ?? "self") as "self" | "view",
        operation: "transitionHelper",
        expression,
        inferredTarget:
          contract.inferredTarget === "argument" ? inferTarget(expression) : contract.inferredTarget,
        requiresManualReview:
          contract.inferredTarget === "argument"
            ? contract.requiresManualReview || !expression.includes("AppView::")
            : contract.requiresManualReview,
        helper,
        rekeysMainAutomationSurface: contract.rekeysMainAutomationSurface,
        focusTarget: contract.focusTarget,
        embeddedAiWindowUpsert: contract.embeddedAiWindowUpsert,
        clearsActionsPopupState: contract.clearsActionsPopupState,
        transitionContract: contract.transitionContract,
      });
    }
  }

  return entries.sort((a, b) => a.line - b.line);
}

function generateInventory(): CurrentViewTransitionInventory {
  const files = SOURCE_DIRS.flatMap(rustFiles);
  const entries = files.flatMap(collectFileEntries).sort((a, b) => {
    if (a.file === b.file) {
      return a.line - b.line;
    }
    return a.file.localeCompare(b.file);
  });
  return {
    schemaVersion: SCHEMA_VERSION,
    generatedFrom: SOURCE_DIRS,
    inventory: "ScriptListApp/AppView current_view transition sites",
    entries,
  };
}

function renderJson(inventory: CurrentViewTransitionInventory): string {
  return `${JSON.stringify(inventory, null, 2)}\n`;
}

function hasFlag(flag: string): boolean {
  return process.argv.includes(flag);
}

const output = renderJson(generateInventory());
const outputPath = resolve(PROJECT_ROOT, OUTPUT_PATH);

if (hasFlag("--stdout")) {
  process.stdout.write(output);
} else if (hasFlag("--check")) {
  const current = readFileSync(outputPath, "utf8");
  if (current !== output) {
    throw new Error(
      `${OUTPUT_PATH} is stale. Run: bun scripts/generate-current-view-transitions.ts --write`,
    );
  }
} else if (hasFlag("--write")) {
  writeFileSync(outputPath, output);
} else {
  process.stderr.write(
    "Usage: bun scripts/generate-current-view-transitions.ts --write|--check|--stdout\n",
  );
  process.exit(2);
}
