import { afterEach, describe, expect, it } from "bun:test";
import { existsSync, mkdtempSync, readlinkSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { runMcpCli } from "./mcp-cli";

let server: ReturnType<typeof Bun.serve> | null = null;

afterEach(() => {
  server?.stop(true);
  server = null;
});

function startMockMcp(handler: (body: any) => any) {
  server = Bun.serve({
    port: 0,
    fetch: async (request) => {
      if (new URL(request.url).pathname !== "/rpc") {
        return new Response("not found", { status: 404 });
      }
      if (request.headers.get("authorization") !== "Bearer test-token") {
        return Response.json({ error: "unauthorized" }, { status: 401 });
      }
      const body = await request.json();
      return Response.json(handler(body));
    },
  });
  return server;
}

async function runCli(args: string[], env: Record<string, string>) {
  const previous = new Map<string, string | undefined>();
  for (const [key, value] of Object.entries(env)) {
    previous.set(key, process.env[key]);
    process.env[key] = value;
  }
  try {
    return await runMcpCli(args);
  } finally {
    for (const [key, value] of previous) {
      if (value === undefined) {
        delete process.env[key];
      } else {
        process.env[key] = value;
      }
    }
  }
}

function discoveryEnv(baseUrl: string) {
  const dir = mkdtempSync(join(tmpdir(), "script-kit-mcp-cli-"));
  const serverJson = join(dir, "server.json");
  writeFileSync(
    serverJson,
    JSON.stringify({
      url: baseUrl,
      token: "test-token",
      version: "test",
      capabilities: { tools: true },
    }),
  );
  return {
    dir,
    env: {
      SCRIPT_KIT_MCP_SERVER_JSON: serverJson,
    },
  };
}

describe("mcp-cli", () => {
  it("prints product-oriented top-level help", async () => {
    const result = await runMcpCli(["--help"]);
    expect(typeof result).toBe("string");
    expect(result).toContain("scriptkit mcp tools");
    expect(result).toContain("scriptkit install-command");
    expect(result).toContain("~/.scriptkit/server.json");
  });

  it("prints mcp subcommand help", async () => {
    const result = await runMcpCli(["mcp", "--help"]);
    expect(typeof result).toBe("string");
    expect(result).toContain("Script Kit MCP commands");
    expect(result).toContain("SCRIPT_KIT_MCP_ENDPOINT");
  });

  it("lists tools through discovery server.json", async () => {
    const mock = startMockMcp((body) => {
      expect(body.method).toBe("tools/list");
      return {
        jsonrpc: "2.0",
        id: body.id,
        result: { tools: [{ name: "kit/trigger_builtin" }] },
      };
    });
    const { dir, env } = discoveryEnv(mock.url.origin);
    try {
      const result = await runCli(["mcp", "tools"], env);
      expect(typeof result).toBe("object");
      expect(result.success).toBe(true);
      expect((result as any).data.result.tools[0].name).toBe("kit/trigger_builtin");
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("calls tools with JSON arguments and bearer auth", async () => {
    const mock = startMockMcp((body) => {
      expect(body.method).toBe("tools/call");
      expect(body.params).toEqual({
        name: "kit/trigger_builtin",
        arguments: { builtinId: "builtin/clipboard-history" },
      });
      return {
        jsonrpc: "2.0",
        id: body.id,
        result: { content: [{ type: "text", text: "{\"ok\":true}" }] },
      };
    });
    const result = await runCli(
      [
        "call",
        "kit/trigger_builtin",
        JSON.stringify({ builtinId: "builtin/clipboard-history" }),
      ],
      {
        SCRIPT_KIT_MCP_ENDPOINT: `${mock.url.origin}/rpc`,
        SCRIPT_KIT_MCP_TOKEN: "test-token",
      },
    );
    expect(typeof result).toBe("object");
    expect(result.success).toBe(true);
    expect((result as any).data.result.content[0].text).toBe("{\"ok\":true}");
  });

  it("keeps direct mcp command aliases for repo-local workflows", async () => {
    const mock = startMockMcp((body) => {
      expect(body.method).toBe("tools/list");
      return {
        jsonrpc: "2.0",
        id: body.id,
        result: { tools: [] },
      };
    });
    const result = await runCli(["tools"], {
      SCRIPT_KIT_MCP_ENDPOINT: mock.url.origin,
      SCRIPT_KIT_MCP_TOKEN: "test-token",
    });
    expect(typeof result).toBe("object");
    expect(result.success).toBe(true);
  });

  it("reads resources", async () => {
    const mock = startMockMcp((body) => {
      expect(body.method).toBe("resources/read");
      expect(body.params).toEqual({ uri: "kit://trigger-builtins" });
      return {
        jsonrpc: "2.0",
        id: body.id,
        result: { contents: [{ uri: "kit://trigger-builtins", text: "ids" }] },
      };
    });
    const result = await runCli(["read", "kit://trigger-builtins"], {
      SCRIPT_KIT_MCP_ENDPOINT: mock.url.origin,
      SCRIPT_KIT_MCP_TOKEN: "test-token",
    });
    expect(typeof result).toBe("object");
    expect((result as any).data.result.contents[0].uri).toBe("kit://trigger-builtins");
  });

  it("installs a scriptkit command symlink at a chosen target", async () => {
    const dir = mkdtempSync(join(tmpdir(), "script-kit-command-"));
    const target = join(dir, "scriptkit");
    try {
      const result = await runMcpCli(["install-command", target]);
      expect(typeof result).toBe("object");
      expect((result as any).success).toBe(true);
      expect(existsSync(target)).toBe(true);
      expect(readlinkSync(target)).toContain("mcp-cli.ts");
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
});
