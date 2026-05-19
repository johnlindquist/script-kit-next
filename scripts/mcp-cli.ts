#!/usr/bin/env bun
/**
 * Script Kit MCP CLI
 *
 * Thin JSON CLI for the live Script Kit MCP server. It reads
 * ~/.scriptkit/server.json by default, or accepts env overrides:
 *   SCRIPT_KIT_MCP_SERVER_JSON
 *   SCRIPT_KIT_MCP_ENDPOINT
 *   SCRIPT_KIT_MCP_TOKEN
 */

import {
  chmodSync,
  existsSync,
  lstatSync,
  mkdirSync,
  readFileSync,
  readlinkSync,
  rmSync,
  symlinkSync,
} from "node:fs";
import { homedir } from "node:os";
import { dirname, join, resolve } from "node:path";

type DiscoveryInfo = {
  url: string;
  token: string;
  version?: string;
  capabilities?: Record<string, unknown>;
};

type CliResult =
  | { success: true; data: unknown }
  | { success: false; error: string };

class CliFailure extends Error {}

function usage() {
  return [
    "Script Kit command line",
    "",
    "Usage:",
    "  scriptkit --help",
    "  scriptkit mcp tools",
    "  scriptkit mcp resources",
    "  scriptkit mcp call <tool-name> [json-arguments]",
    "  scriptkit mcp read <resource-uri>",
    "  scriptkit mcp rpc <method> [json-params]",
    "  scriptkit install-command [target-path]",
    "",
    "Examples:",
    "  scriptkit mcp tools",
    "  scriptkit mcp read kit://trigger-builtins",
    "  scriptkit mcp call kit/trigger_builtin '{\"builtinId\":\"builtin/clipboard-history\"}'",
    "  scriptkit install-command ~/.local/bin/scriptkit",
    "",
    "MCP commands require Script Kit to be running so ~/.scriptkit/server.json exists.",
    "The discovery file contains a bearer token; do not paste it into logs or docs.",
  ].join("\n");
}

function mcpUsage() {
  return [
    "Script Kit MCP commands",
    "",
    "Usage:",
    "  scriptkit mcp tools",
    "  scriptkit mcp resources",
    "  scriptkit mcp call <tool-name> [json-arguments]",
    "  scriptkit mcp read <resource-uri>",
    "  scriptkit mcp rpc <method> [json-params]",
    "",
    "Environment overrides:",
    "  SCRIPT_KIT_MCP_SERVER_JSON  Path to server.json",
    "  SCRIPT_KIT_MCP_ENDPOINT     Base URL or /rpc endpoint",
    "  SCRIPT_KIT_MCP_TOKEN        Bearer token",
  ].join("\n");
}

function print(result: CliResult): void {
  console.log(JSON.stringify(result, null, 2));
}

function fail(message: string): never {
  throw new CliFailure(message);
}

function parseJsonArg(raw: string | undefined, fallback: unknown): unknown {
  if (raw === undefined || raw.trim() === "") {
    return fallback;
  }
  try {
    return JSON.parse(raw);
  } catch (error) {
    fail(`Invalid JSON argument: ${error instanceof Error ? error.message : String(error)}`);
  }
}

function discoveryPath(): string {
  return (
    process.env.SCRIPT_KIT_MCP_SERVER_JSON ??
    join(homedir(), ".scriptkit", "server.json")
  );
}

function loadDiscovery(): DiscoveryInfo | null {
  const path = discoveryPath();
  if (!existsSync(path)) {
    return null;
  }
  try {
    return JSON.parse(readFileSync(path, "utf8")) as DiscoveryInfo;
  } catch (error) {
    fail(`Failed to parse ${path}: ${error instanceof Error ? error.message : String(error)}`);
  }
}

function envValue(name: string): string | undefined {
  const value = process.env[name];
  return value && value.trim() !== "" ? value : undefined;
}

function resolveEndpointAndToken(): { endpoint: string; token: string } {
  const discovery = loadDiscovery();
  const token = envValue("SCRIPT_KIT_MCP_TOKEN") ?? discovery?.token;
  const endpointOrBase = envValue("SCRIPT_KIT_MCP_ENDPOINT") ?? discovery?.url;

  if (!endpointOrBase) {
    fail(
      `Missing MCP endpoint. Set SCRIPT_KIT_MCP_ENDPOINT or start Script Kit so ${discoveryPath()} exists.`,
    );
  }
  if (!token) {
    fail(
      `Missing MCP token. Set SCRIPT_KIT_MCP_TOKEN or start Script Kit so ${discoveryPath()} contains a token.`,
    );
  }

  const endpoint = endpointOrBase.endsWith("/rpc")
    ? endpointOrBase
    : `${endpointOrBase.replace(/\/$/, "")}/rpc`;
  return { endpoint, token };
}

function defaultCommandTarget(): string {
  return join(homedir(), ".local", "bin", "scriptkit");
}

function currentScriptPath(): string {
  const url = new URL(import.meta.url);
  if (url.protocol !== "file:") {
    fail("Cannot install command because the current CLI is not running from a file path.");
  }
  return url.pathname;
}

function installCommand(targetArg: string | undefined): CliResult {
  const source = currentScriptPath();
  const target = resolve(targetArg?.trim() || defaultCommandTarget());
  mkdirSync(dirname(target), { recursive: true });

  if (existsSync(target)) {
    const stat = lstatSync(target);
    if (stat.isSymbolicLink()) {
      const existing = readlinkSync(target);
      if (resolve(dirname(target), existing) !== source) {
        rmSync(target);
      }
    } else {
      fail(
        `Refusing to replace non-symlink at ${target}. Remove it or pass a different target path.`,
      );
    }
  }

  if (!existsSync(target)) {
    symlinkSync(source, target);
  }
  chmodSync(source, 0o755);

  return {
    success: true,
    data: {
      command: "scriptkit",
      target,
      source,
      note:
        "Add the target directory to PATH if `scriptkit --help` is not found in new shells.",
    },
  };
}

export async function rpc(method: string, params: unknown): Promise<unknown> {
  const { endpoint, token } = resolveEndpointAndToken();
  const response = await fetch(endpoint, {
    method: "POST",
    headers: {
      authorization: `Bearer ${token}`,
      "content-type": "application/json",
    },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: `script-kit-mcp-cli-${Date.now()}`,
      method,
      params,
    }),
  });

  const text = await response.text();
  let payload: unknown;
  try {
    payload = text ? JSON.parse(text) : null;
  } catch {
    fail(`MCP server returned non-JSON HTTP ${response.status}: ${text}`);
  }

  if (!response.ok) {
    fail(`MCP server returned HTTP ${response.status}: ${JSON.stringify(payload)}`);
  }

  return payload;
}

export async function runMcpCli(argv: string[]): Promise<CliResult | string> {
  const [rawCommand, ...rest] = argv;
  if (!rawCommand || rawCommand === "--help" || rawCommand === "-h") {
    return usage();
  }

  if (rawCommand === "install-command") {
    return installCommand(rest[0]);
  }

  let command = rawCommand;
  let args = rest;
  if (rawCommand === "mcp") {
    const [mcpCommand, ...mcpArgs] = rest;
    if (!mcpCommand || mcpCommand === "--help" || mcpCommand === "-h") {
      return mcpUsage();
    }
    command = mcpCommand;
    args = mcpArgs;
  }

  const [first, second] = args;
  let data: unknown;
  if (command === "tools" || command === "list-tools") {
    data = await rpc("tools/list", {});
  } else if (command === "resources" || command === "list-resources") {
    data = await rpc("resources/list", {});
  } else if (command === "call") {
    if (!first) {
      fail("call requires a tool name");
    }
    data = await rpc("tools/call", {
      name: first,
      arguments: parseJsonArg(second, {}),
    });
  } else if (command === "read") {
    if (!first) {
      fail("read requires a resource URI");
    }
    data = await rpc("resources/read", { uri: first });
  } else if (command === "rpc") {
    if (!first) {
      fail("rpc requires a method");
    }
    data = await rpc(first, parseJsonArg(second, {}));
  } else {
    fail(`Unknown command: ${rawCommand}. Use --help for usage.`);
  }

  return { success: true, data };
}

async function main() {
  const result = await runMcpCli(process.argv.slice(2));
  if (typeof result === "string") {
    console.log(result);
  } else {
    print(result);
  }
}

if (import.meta.main) {
  main().catch((error) => {
    print({
      success: false,
      error: error instanceof Error ? error.message : String(error),
    });
    process.exit(error instanceof CliFailure ? 1 : 1);
  });
}
