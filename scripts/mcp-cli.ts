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
  fchmodSync,
  closeSync,
  constants as fsConstants,
  existsSync,
  fstatSync,
  lstatSync,
  mkdirSync,
  openSync,
  readFileSync,
  readlinkSync,
  realpathSync,
  statSync,
  symlinkSync,
} from "node:fs";
import { homedir } from "node:os";
import { fileURLToPath } from "node:url";
import { dirname, isAbsolute, join, resolve, sep } from "node:path";

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
    "  scriptkit --help | --skill",
    "  scriptkit agent discover | note | run | available | take | reply | away",
    "  scriptkit install-skill <skills-directory> | inspect-command <name>",
    "  scriptkit grant-agent <name> <private-credential-file> <scope> ...",
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
    "  SCRIPT_KIT_MCP_ENDPOINT     Loopback base URL or /rpc endpoint; another server requires its own credential",
    "  SCRIPT_KIT_MCP_TOKEN        Bearer token",
    "  SCRIPT_KIT_MCP_TIMEOUT_MS   Request timeout, at most 120000ms",
    "  SCRIPT_KIT_MCP_MAX_RESPONSE_BYTES  Response budget, at most 67108864 bytes",
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

// HOME is already a pathname, not another shell expression. Validate before join can erase '..'.
function checkedHome(): string {
  const home = (process.platform === "win32" ? undefined : process.env.HOME) ?? homedir();
  if (!isAbsolute(home) || pathParts(home).includes("..")) fail("Use an absolute home path without '..', or supply an explicit workspace or command target.");
  return home;
}

function discoveryPath(): string {
  // Check before join(): a link followed by '..' need not name the lexical parent.
  const checked = (path: string) => {
    if (!isAbsolute(path) || path.split(process.platform === "win32" ? /[\\/]/ : "/").includes("..")) {
      fail("Use an absolute MCP discovery or workspace path without '..'. Unset SK_PATH for the default workspace.");
    }
    return path;
  };
  const explicit = process.env.SCRIPT_KIT_MCP_SERVER_JSON;
  if (explicit !== undefined) return checked(explicit);

  const configured = process.env.SK_PATH;
  if (configured === undefined) return join(checkedHome(), ".scriptkit", "server.json");
  const expanded = configured.replace(/\$\{([A-Za-z_][A-Za-z0-9_]*)\}|\$([A-Za-z_][A-Za-z0-9_]*)/g,
    (_, braced: string | undefined, bare: string | undefined) =>
      process.env[braced ?? bare ?? ""] ?? fail("Unknown variable in SK_PATH; use an absolute workspace path"),
  );
  const workspace = /^~(?=\/|$)/.test(configured) ? expanded.replace(/^~/, checkedHome) : expanded;
  return join(checked(workspace), "server.json");
}

function loadDiscovery(locationOnly=false): DiscoveryInfo | null {
  const path = locationOnly ? join(dirname(discoveryPath()),"server-location.json") : discoveryPath();
  if (!existsSync(path)) {
    return null;
  }
  let descriptor: number | undefined;
  try {
    const owner = lstatSync(path);
    if (!owner.isFile() || owner.isSymbolicLink()) {
      fail(`Script Kit MCP requires a private regular discovery file: ${path}`);
    }
    descriptor = openSync(path, fsConstants.O_RDONLY | (fsConstants.O_NOFOLLOW ?? 0));
    const opened = fstatSync(descriptor);
    if (!opened.isFile() || opened.dev !== owner.dev || opened.ino !== owner.ino) {
      fail(`Script Kit MCP requires a private regular discovery file: ${path}`);
    }
    if (process.platform !== "win32" && (opened.mode & 0o077) !== 0) {
      fail(`Script Kit MCP discovery requires owner-only permissions: ${path}`);
    }
    if (typeof process.getuid === "function" && opened.uid !== process.getuid()) {
      fail(`Script Kit MCP discovery must belong to the current user: ${path}`);
    }
    const text = readFileSync(descriptor, "utf8");
    let discovery: DiscoveryInfo;
    try {
      discovery = JSON.parse(text) as DiscoveryInfo;
    } catch {
      // A parser error can quote private file contents, including an unquoted bearer.
      return fail("Script Kit's MCP discovery file contains invalid JSON. Restart Script Kit to recreate it.");
    }
    if (!discovery || typeof discovery !== "object") {
      fail(`Script Kit MCP discovery must contain a private endpoint and token: ${path}`);
    }
    return discovery;
  } catch (error) {
    if (error instanceof CliFailure) throw error;
    return fail(`Failed to parse ${path}: ${error instanceof Error ? error.message : String(error)}`);
  } finally {
    if (descriptor !== undefined) closeSync(descriptor);
  }
}

function envValue(name: string): string | undefined {
  const value = process.env[name];
  if (value !== undefined && !value.trim()) fail(`${name} is empty. Supply a valid value; no operator credential was used.`);
  return value;
}

function reviewedLocalEndpoint(endpointOrBase: string): string {
  let endpoint: URL;
  try {
    endpoint = new URL(endpointOrBase);
  } catch {
    fail("A local Script Kit MCP endpoint must be a valid loopback HTTP /rpc URL.");
  }
  const loopback = endpoint.hostname === "localhost"
    || endpoint.hostname === "127.0.0.1"
    || endpoint.hostname === "[::1]";
  if (
    !loopback ||
    !["http:", "https:"].includes(endpoint.protocol) ||
    endpoint.username ||
    endpoint.password ||
    endpoint.search ||
    endpoint.hash ||
    !["/", "/rpc"].includes(endpoint.pathname)
  ) {
    fail("A local Script Kit MCP endpoint must be a loopback HTTP /rpc URL without credentials, query, or fragment.");
  }
  endpoint.pathname = "/rpc";
  return endpoint.toString();
}

function resolveEndpointAndToken(explicitToken?:string): { endpoint: string; token: string } {
  const endpointOverride = envValue("SCRIPT_KIT_MCP_ENDPOINT");
  const tokenOverride = explicitToken ?? envValue("SCRIPT_KIT_MCP_TOKEN");
  if (tokenOverride !== undefined && (tokenOverride.length > 4096 || !/^[A-Za-z0-9._~+/\-]+=*$/.test(tokenOverride))) {
    fail("The supplied MCP credential is invalid. No operator credential was used.");
  }
  // Validate an explicit destination before opening local bearer-token state.
  const reviewedOverride = endpointOverride ? reviewedLocalEndpoint(endpointOverride) : undefined;
  const discovery = reviewedOverride && tokenOverride ? null : loadDiscovery(tokenOverride !== undefined);
  const token = tokenOverride ?? discovery?.token;
  const endpointOrBase = reviewedOverride ?? discovery?.url;

  if (!endpointOrBase) {
    fail(
      `Missing MCP endpoint. Set SCRIPT_KIT_MCP_ENDPOINT or start Script Kit so ${discoveryPath()} exists.`,
    );
  }
  if (typeof token !== "string" || !token.trim() || /[\r\n]/.test(token)) {
    fail(
      `Missing MCP token. Set SCRIPT_KIT_MCP_TOKEN or start Script Kit so ${discoveryPath()} contains a token.`,
    );
  }

  if (typeof endpointOrBase !== "string") {
    fail("A local Script Kit MCP endpoint must be a valid loopback HTTP /rpc URL.");
  }
  const endpoint = reviewedOverride ?? reviewedLocalEndpoint(endpointOrBase);
  // An endpoint override alone must never move Kit's operator credential to another
  // listener. Compare normalized URLs, not merely “both loopback” or matching ports.
  if (reviewedOverride !== undefined && tokenOverride === undefined &&
      (typeof discovery?.url !== "string" || reviewedLocalEndpoint(discovery.url) !== reviewedOverride)) {
    fail("The endpoint override does not match the server that supplied the stored credential. Unset SCRIPT_KIT_MCP_ENDPOINT or supply a credential for that server.");
  }
  return { endpoint, token };
}

function mcpRequestTimeoutMs(): number {
  const configured = process.env.SCRIPT_KIT_MCP_TIMEOUT_MS;
  if (configured === undefined || configured === "") return 30_000;
  const timeout = Number(configured);
  if (!/^[1-9][0-9]*$/.test(configured) || !Number.isSafeInteger(timeout) || timeout > 120_000) {
    fail("SCRIPT_KIT_MCP_TIMEOUT_MS must be a positive whole duration no greater than 120000ms.");
  }
  return timeout;
}

function mcpMaxResponseBytes(): number {
  const configured = process.env.SCRIPT_KIT_MCP_MAX_RESPONSE_BYTES;
  if (configured === undefined || configured === "") return 16 * 1024 * 1024;
  const budget = Number(configured);
  if (!/^[1-9][0-9]*$/.test(configured) || !Number.isSafeInteger(budget) || budget > 64 * 1024 * 1024) {
    fail("SCRIPT_KIT_MCP_MAX_RESPONSE_BYTES must be a positive whole byte budget no greater than 67108864.");
  }
  return budget;
}

async function readBoundedMcpResponse(response: Response, maximumBytes: number): Promise<string> {
  const declared = response.headers.get("content-length");
  if (declared !== null && /^\d+$/.test(declared) && Number(declared) > maximumBytes) {
    await response.body?.cancel();
    fail(`Script Kit MCP exceeded its ${maximumBytes}-byte response safety budget.`);
  }

  const reader = response.body?.getReader();
  if (!reader) return "";
  const chunks: Uint8Array[] = [];
  let received = 0;
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      received += value.byteLength;
      if (received > maximumBytes) {
        await reader.cancel();
        fail(`Script Kit MCP exceeded its ${maximumBytes}-byte response safety budget.`);
      }
      chunks.push(value);
    }
  } finally {
    reader.releaseLock();
  }
  return new TextDecoder().decode(Buffer.concat(chunks, received));
}

function safeMcpDiagnostic(message: string, token: string): string {
  return message
    .split(token).join("[REDACTED]")
    .replace(/\bBearer\s+[^\s,;"'}]+/gi, "Bearer [REDACTED]")
    .slice(0, 512);
}

const installUsage = "Usage: scriptkit install-command [target-path]. Omit the target for ~/.local/bin/scriptkit. Use a filename without '..'.";
const pathParts = (p: string) => p.split(process.platform === "win32" ? /[\\/]/ : "/");

function commandTarget(args: string[]): string {
  if (!args.length) {
    return join(checkedHome(), ".local", "bin", "scriptkit");
  }
  if (args.length !== 1) fail(installUsage);
  const [raw] = args;
  const parts = pathParts(raw), name = parts.at(-1);
  if (!raw.trim() || raw.startsWith("-") || raw.includes("\0") || parts.includes("..") || !name || name === ".") {
    fail(installUsage);
  }
  // No parent traversal remains. Preserve spaces in the name; a blank argument is not omission.
  return resolve(raw);
}

type FileId = { dev: bigint; ino: bigint };
const sameFile = (a: FileId, b: FileId) => a.dev === b.dev && a.ino === b.ino;

function commandPointsTo(target: string, source: string, id: FileId): boolean {
  try {
    const link = lstatSync(target, { bigint: true });
    if (!link.isSymbolicLink()) return false;
    const payload = readlinkSync(target);
    // Let the filesystem walk the link as the shell does. join()/resolve() would cancel a
    // directory link followed by '..' before the OS could follow it.
    const referenced = isAbsolute(payload) ? payload : dirname(target) + sep + payload;
    const leaf = lstatSync(referenced, { bigint: true });
    return leaf.isFile() && sameFile(leaf, id) && sameFile(statSync(target, { bigint: true }), id)
      // Another name for a hard-linked inode will not follow a later atomic Kit upgrade.
      && realpathSync.native(target) === source && sameFile(lstatSync(target, { bigint: true }), link);
  } catch {
    return false;
  }
}

function installCommand(args: string[]): CliResult | string {
  if (args.length === 1 && ["--help", "-h"].includes(args[0])) return installUsage;
  const target = commandTarget(args);
  const url = new URL(import.meta.url);
  if (url.protocol !== "file:") fail("The current CLI is not running from a file path.");
  const source = realpathSync.native(fileURLToPath(url));
  const fd = openSync(source, fsConstants.O_RDONLY | (fsConstants.O_NOFOLLOW ?? 0) | (fsConstants.O_NONBLOCK ?? 0));
  try {
    const file = fstatSync(fd, { bigint: true });
    if (!file.isFile()) fail("The current CLI is not a regular file.");
    const needsExecute = (file.mode & 0o100n) === 0n;
    if (needsExecute && file.nlink !== 1n) {
      fail("The CLI has another hard link; refusing to change its executable permissions.");
    }
    mkdirSync(dirname(target), { recursive: true });
    const existing = lstatSync(target, { throwIfNoEntry: false });
    if (existing && !commandPointsTo(target, source, file)) {
      fail(`Refusing to replace a different command at ${target}. Choose an unused target path.`);
    }
    if (needsExecute) fchmodSync(fd, Number(file.mode & 0o777n) | 0o100);
    if (!sameFile(statSync(source, { bigint: true }), file)) {
      fail("The CLI file changed. Check the source before installing the command.");
    }
    if (!existing) {
      try { symlinkSync(source, target); }
      catch (error) {
        // A competing installer might have won. Verify its entry below; never replace or retry.
        if ((error as NodeJS.ErrnoException).code !== "EEXIST") throw error;
      }
    }
    if (!commandPointsTo(target, source, file) || !(fstatSync(fd, { bigint: true }).mode & 0o100n)) {
      fail("The command link could not be verified. Inspect the target before trying again.");
    }
    return {
      success: true,
      data: {
        command: "scriptkit", target, source,
        note: "This link names the current CLI. Use command -v scriptkit to check which command your shell will run.",
      },
    };
  } finally {
    closeSync(fd);
  }
}

export async function rpc(method: string, params: unknown, explicitToken?:string): Promise<unknown> {
  const scopedFile=envValue("SCRIPT_KIT_AGENT_CREDENTIAL");
  if(explicitToken===undefined && scopedFile) explicitToken=(await import("./agent-bridge-cli.ts")).credentialToken(scopedFile);
  const { endpoint, token } = resolveEndpointAndToken(explicitToken);
  const timeoutMs = mcpRequestTimeoutMs();
  const maximumResponseBytes = mcpMaxResponseBytes();
  const requestId = `script-kit-mcp-cli-${crypto.randomUUID()}`;
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  let response: Response;
  let text: string;
  try {
    response = await fetch(endpoint, {
      method: "POST",
      redirect: "error",
      signal: controller.signal,
      headers: {
        authorization: `Bearer ${token}`,
        "content-type": "application/json",
      },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: requestId,
        method,
        params,
      }),
    });
    text = await readBoundedMcpResponse(response, maximumResponseBytes);
  } catch (error) {
    if (controller.signal.aborted) {
      fail(`Script Kit MCP ${method} timed out after ${timeoutMs}ms.`);
    }
    const detail = error instanceof Error ? error.message : String(error);
    fail(`Script Kit MCP ${method} request failed: ${safeMcpDiagnostic(detail, token)}`);
  } finally {
    clearTimeout(timer);
  }

  let payload: unknown;
  try {
    payload = text ? JSON.parse(text) : null;
  } catch {
    fail(`MCP server returned non-JSON HTTP ${response.status}.`);
  }

  if (!response.ok) {
    fail(`MCP server returned HTTP ${response.status}.`);
  }

  if (
    !payload ||
    typeof payload !== "object" ||
    Array.isArray(payload) ||
    (payload as Record<string, unknown>).jsonrpc !== "2.0" ||
    (payload as Record<string, unknown>).id !== requestId
  ) {
    fail(`Script Kit MCP ${method} returned an invalid JSON-RPC response.`);
  }
  const envelope = payload as Record<string, unknown>;
  const hasResult = Object.hasOwn(envelope, "result");
  const hasError = Object.hasOwn(envelope, "error");
  if (hasResult === hasError) {
    fail(`Script Kit MCP ${method} returned an invalid JSON-RPC response.`);
  }
  if (hasError) {
    const error = envelope.error;
    if (
      !error ||
      typeof error !== "object" ||
      !Number.isSafeInteger((error as Record<string, unknown>).code) ||
      typeof (error as Record<string, unknown>).message !== "string" ||
      !(error as Record<string, unknown>).message
    ) {
      fail(`Script Kit MCP ${method} returned an invalid JSON-RPC response.`);
    }
    const typedError = error as { code: number; message: string };
    fail(`Script Kit MCP ${method} failed (${typedError.code}): ${safeMcpDiagnostic(typedError.message, token)}`);
  }

  return payload;
}

export async function runMcpCli(argv: string[]): Promise<CliResult | string> {
  const [rawCommand, ...rest] = argv;
  if (!rawCommand || rawCommand === "--help" || rawCommand === "-h") {
    return usage();
  }

  if (["--skill","grant-agent","install-skill","inspect-command","agent"].includes(rawCommand)) {
    const bridge=await import("./agent-bridge-cli.ts");
    if(rawCommand==='--skill')return bridge.agentDoc();
    if(rawCommand==='agent')return {success:true,data:await bridge.agent(rest,rpc)};
    const operatorOnly = ["grant-agent", "inspect-command"].includes(rawCommand);
    if (operatorOnly && (process.env.SCRIPT_KIT_AGENT_CREDENTIAL !== undefined || process.env.SCRIPT_KIT_MCP_TOKEN !== undefined)) {
      fail("Use an operator shell, not an agent credential context");
    }
    if (process.env.SCRIPT_KIT_MCP_SERVER_JSON !== undefined || (operatorOnly && process.env.SCRIPT_KIT_MCP_ENDPOINT !== undefined)) {
      fail("Use SK_PATH rather than an endpoint override for operator commands");
    }
    return {success:true,data:bridge.operator(argv,dirname(discoveryPath()))};
  }
  if (rawCommand === "install-command") {
    return installCommand(rest);
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
    if ((data as {result?:{isError?:boolean}}).result?.isError) {
      let code="tool_error";try {const v=JSON.parse((data as any).result.content[0].text);if(typeof v.error?.code==="string"&&/^[a-z_]{1,64}$/.test(v.error.code))code=v.error.code;}catch{}
      fail(`Script Kit tool ${first} refused the call: ${code}`);
    }
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
