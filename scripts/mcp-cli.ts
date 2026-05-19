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

import { existsSync, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

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
    "Usage:",
    "  bun scripts/mcp-cli.ts tools",
    "  bun scripts/mcp-cli.ts resources",
    "  bun scripts/mcp-cli.ts call <tool-name> [json-arguments]",
    "  bun scripts/mcp-cli.ts read <resource-uri>",
    "  bun scripts/mcp-cli.ts rpc <method> [json-params]",
    "",
    "Examples:",
    "  bun scripts/mcp-cli.ts call kit/trigger_builtin '{\"builtinId\":\"builtin/clipboard-history\"}'",
    "  bun scripts/mcp-cli.ts read kit://trigger-builtins",
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
  const [command, first, second] = argv;
  if (!command || command === "--help" || command === "-h") {
    return usage();
  }

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
    fail(`Unknown command: ${command}. Use --help for usage.`);
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
