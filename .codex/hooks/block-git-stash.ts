#!/usr/bin/env bun

import { readFileSync } from "node:fs";

const BLOCK_MESSAGE =
  "Multiple AI agents are working against this branch. Please wait a few minutes for them to finish, then continue your work";

type HookPayload = {
  tool_name?: string;
  tool_input?: {
    command?: string;
    cmd?: string;
  };
};

type PermissionDecision = {
  hookSpecificOutput: {
    hookEventName: "PreToolUse";
    permissionDecision: "deny";
    permissionDecisionReason: string;
  };
};

const input = readFileSync(0, "utf8");
const payload = JSON.parse(input) as HookPayload;
const command = payload.tool_input?.command ?? payload.tool_input?.cmd ?? "";

const COMMAND_SEPARATORS = new Set(["&&", "||", ";", "|", "&"]);
const WRAPPER_COMMANDS = new Set(["command", "builtin", "env", "nohup"]);
const STASH_READONLY_SUBCOMMANDS = new Set([
  "list",
  "show",
  "pop",
  "apply",
  "drop",
  "clear",
  "branch",
]);

function tokenizeShell(commandText: string): string[] {
  const tokens: string[] = [];
  let current = "";
  let quote: "'" | '"' | null = null;

  for (let index = 0; index < commandText.length; index += 1) {
    const char = commandText[index];
    const next = commandText[index + 1];

    if (quote === "'") {
      if (char === "'") {
        quote = null;
      } else {
        current += char;
      }
      continue;
    }

    if (quote === '"') {
      if (char === '"') {
        quote = null;
      } else if (char === "\\" && next !== undefined) {
        current += next;
        index += 1;
      } else {
        current += char;
      }
      continue;
    }

    if (char === "'" || char === '"') {
      quote = char;
      continue;
    }

    if (/\s/.test(char)) {
      if (current.length > 0) {
        tokens.push(current);
        current = "";
      }
      continue;
    }

    if ((char === "&" || char === "|") && next === char) {
      if (current.length > 0) {
        tokens.push(current);
        current = "";
      }
      tokens.push(char + next);
      index += 1;
      continue;
    }

    if (COMMAND_SEPARATORS.has(char)) {
      if (current.length > 0) {
        tokens.push(current);
        current = "";
      }
      tokens.push(char);
      continue;
    }

    if (char === "\\" && next !== undefined) {
      current += next;
      index += 1;
      continue;
    }

    current += char;
  }

  if (current.length > 0) {
    tokens.push(current);
  }

  return tokens;
}

function splitCommands(tokens: string[]): string[][] {
  const commands: string[][] = [];
  let current: string[] = [];

  for (const token of tokens) {
    if (COMMAND_SEPARATORS.has(token)) {
      if (current.length > 0) {
        commands.push(current);
        current = [];
      }
      continue;
    }

    current.push(token);
  }

  if (current.length > 0) {
    commands.push(current);
  }

  return commands;
}

function isEnvAssignment(token: string): boolean {
  return /^[A-Za-z_][A-Za-z0-9_]*=.*/.test(token);
}

function findGitInvocation(tokens: string[]): string[] | null {
  let index = 0;

  while (index < tokens.length && isEnvAssignment(tokens[index])) {
    index += 1;
  }

  while (index < tokens.length && WRAPPER_COMMANDS.has(tokens[index])) {
    index += 1;
    while (index < tokens.length && isEnvAssignment(tokens[index])) {
      index += 1;
    }
  }

  while (index < tokens.length && tokens[index].startsWith("-")) {
    index += 1;
    if (tokens[index - 1] === "env" && index < tokens.length) {
      continue;
    }
  }

  const executable = tokens[index];
  if (!executable) {
    return null;
  }

  if (executable === "git" || executable.endsWith("/git")) {
    return tokens.slice(index + 1);
  }

  return null;
}

function shouldBlockGitStash(gitArgs: string[]): boolean {
  if (gitArgs.includes("--autostash")) {
    return true;
  }

  let index = 0;

  while (index < gitArgs.length) {
    const token = gitArgs[index];
    if (token === "-c" || token === "--config-env") {
      index += 2;
      continue;
    }
    if (token === "-C") {
      index += 2;
      continue;
    }
    if (token.startsWith("--")) {
      index += 1;
      continue;
    }
    if (token.startsWith("-")) {
      index += 1;
      continue;
    }
    break;
  }

  if (gitArgs[index] !== "stash") {
    return false;
  }

  const subcommand = gitArgs[index + 1];
  if (!subcommand) {
    return true;
  }

  if (subcommand.startsWith("-")) {
    return true;
  }

  return !STASH_READONLY_SUBCOMMANDS.has(subcommand);
}

function buildDenyDecision(reason: string): PermissionDecision {
  return {
    hookSpecificOutput: {
      hookEventName: "PreToolUse",
      permissionDecision: "deny",
      permissionDecisionReason: reason,
    },
  };
}

const tokens = tokenizeShell(command);
const commands = splitCommands(tokens);

for (const shellCommand of commands) {
  const gitArgs = findGitInvocation(shellCommand);
  if (!gitArgs) {
    continue;
  }

  if (shouldBlockGitStash(gitArgs)) {
    process.stdout.write(
      `${JSON.stringify(buildDenyDecision(BLOCK_MESSAGE))}\n`,
    );
    process.exit(0);
  }
}
