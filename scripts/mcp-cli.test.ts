import { afterEach, describe, expect, it } from "bun:test";
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdtempSync,
  readlinkSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { runMcpCli } from "./mcp-cli";

const originalFetch = globalThis.fetch;
// These fixtures never preconnect or open a socket, including with Bun's extended fetch type.
const stubResponse = (respond: () => Promise<Response>) => Object.assign(respond, {
  preconnect() { throw Error("Unexpected preconnect in MCP fixture"); },
});
let server: { url: URL; stop: (force?: boolean) => void } | null = null;

afterEach(() => {
  server?.stop(true);
  server = null;
  globalThis.fetch = originalFetch;
});

function startMockMcp(handler: (body: any) => any) {
  const previousFetch = globalThis.fetch;
  const url = new URL("http://127.0.0.1:43129/");
  globalThis.fetch = (async (input: RequestInfo | URL, init?: RequestInit) => {
      const request = new Request(input, init);
      if (new URL(request.url).pathname !== "/rpc") {
        return new Response("not found", { status: 404 });
      }
      if (request.headers.get("authorization") !== "Bearer test-token") {
        return Response.json({ error: "unauthorized" }, { status: 401 });
      }
      const body = await request.json();
      return Response.json(handler(body));
  }) as typeof fetch;
  server = {
    url,
    stop() {
      globalThis.fetch = previousFetch;
    },
  };
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
    { mode: 0o600 },
  );
  return {
    dir,
    env: {
      SCRIPT_KIT_MCP_SERVER_JSON: serverJson,
    },
  };
}

describe("workspace path identity", () => {
  const local = { SCRIPT_KIT_MCP_ENDPOINT: "", SCRIPT_KIT_MCP_SERVER_JSON: "", SCRIPT_KIT_MCP_TOKEN: "", SCRIPT_KIT_AGENT_CREDENTIAL: "" };

  it("honors significant spaces in connection files, workspace names and grant registrations", async () => {
    const { mkdirSync } = await import("node:fs");
    const mock = startMockMcp(body => ({ jsonrpc: "2.0", id: body.id, result: { tools: [] } }));
    const { dir } = discoveryEnv(mock.url.origin);
    try {
      const selected = join(dir, "Kit "), neighbor = join(dir, "Kit");
      const connection = (token: string) => JSON.stringify({ url: mock.url.origin, token });
      for (const root of [selected, neighbor]) {
        mkdirSync(root, { mode: 0o700 });
        writeFileSync(join(root, "server.json"), connection(root === selected ? "test-token" : "neighbor-token"), { mode: 0o600 });
      }
      const explicit = join(neighbor, "server.json ");
      writeFileSync(explicit, connection("test-token"), { mode: 0o600 });
      for (const env of [{ ...local, SK_PATH: selected }, { ...local, SCRIPT_KIT_MCP_SERVER_JSON: explicit }]) {
        expect(await runCli(["mcp", "tools"], env)).toMatchObject({ success: true });
      }
      await runCli(["grant-agent", "reader", join(dir, "reader"), "notes:quiet"], { ...local, SK_PATH: selected });
      expect(existsSync(join(selected, "agents/reader.json"))).toBe(true);
      expect(existsSync(join(neighbor, "agents/reader.json"))).toBe(false);
    } finally { rmSync(dir, { recursive: true, force: true }); }
  });

  it("refuses an ambiguous workspace, discovery file or new credential before contact or issuance", async () => {
    const { mkdirSync } = await import("node:fs");
    let calls = 0;
    const mock = startMockMcp(body => { calls++; return { jsonrpc: "2.0", id: body.id, result: {} }; });
    const { dir } = discoveryEnv(mock.url.origin);
    try {
      const logical = join(dir, "logical"), physical = join(dir, "physical"), alias = join(logical, "via");
      for (const path of [logical, physical, join(physical, "anchor"), join(physical, "kit"), join(logical, "kit")]) mkdirSync(path, { mode: 0o700 });
      symlinkSync(join(physical, "anchor"), alias, "dir");
      const ambiguous = alias + "/../kit"; // join() would erase the very distinction under test.
      for (const base of [logical, physical]) {
        copyFileSync(join(dir, "server.json"), join(base, "kit/server.json"));
        writeFileSync(join(base, "kit/server-location.json"), JSON.stringify({ url: mock.url.origin }), { mode: 0o600 });
      }
      for (const env of [{ ...local, SK_PATH: ambiguous }, { ...local, SK_PATH: "$KIT_TEST_ROOT", KIT_TEST_ROOT: ambiguous },
        { ...local, SCRIPT_KIT_MCP_SERVER_JSON: ambiguous + "/server.json", SCRIPT_KIT_MCP_TOKEN: "test-token" }]) {
        await expect(runCli(["mcp", "tools"], env)).rejects.toThrow("without '..'");
      }
      await expect(runCli(["grant-agent", "reader", ambiguous + "/reader", "notes:quiet"], { ...local, SK_PATH: physical })).rejects.toThrow("without '..'");
      expect(calls).toBe(0);
      for (const base of [logical, physical]) expect(existsSync(join(base, "kit/reader"))).toBe(false);
      expect(existsSync(join(physical, "agents/reader.json"))).toBe(false);
    } finally { rmSync(dir, { recursive: true, force: true }); }
  });
});

describe("scoped credential destinations", () => {
  async function fixture(run: (root: string, priv: string, grant: (name: string, file: string) => Promise<unknown>) => Promise<void>) {
    const { mkdirSync } = await import("node:fs");
    const dir = realpathSync(mkdtempSync(join(tmpdir(), "kit-grant-")));
    const root = join(dir, "workspace"), priv = join(dir, "private");
    try {
      for (const path of [root, priv]) mkdirSync(path, { mode: 0o700 });
      const env = { SK_PATH: root, SCRIPT_KIT_AGENT_CREDENTIAL: "", SCRIPT_KIT_MCP_TOKEN: "", SCRIPT_KIT_MCP_SERVER_JSON: "" };
      await run(root, priv, (name, file) => runCli(["grant-agent", name, file, "notes:quiet"], env));
    } finally { rmSync(dir, { recursive: true, force: true }); }
  }

  it("never mints a scoped bearer as Kit's operator token, script, or registration", () => fixture(async (root, priv, grant) => {
    const { mkdirSync } = await import("node:fs");
    const scripts = join(root, "plugins/main/scripts"), agents = join(root, "agents");
    mkdirSync(scripts, { recursive: true, mode: 0o700 }); mkdirSync(agents, { mode: 0o700 });
    symlinkSync(join(root, "plugins/main"), join(priv, "alias"), "dir");
    symlinkSync(join(root, ".."), join(priv, "parent-alias"), "dir");
    mkdirSync(join(root, "themes"), { mode: 0o700 });
    mkdirSync(join(priv, "nested"), { mode: 0o700 });
    symlinkSync(priv, join(root, "themes", "linked"), "dir");
    for (const dest of [join(scripts, "reader.ts"), join(priv, "alias/scripts/reader.ts"), join(root, "agent-token"),
      join(priv, "parent-alias/workspace/agent-token"), join(root, "AGENT-TOKEN"), join(root, "theme.json"),
      join(root, "themes/linked/nested/theme.json"), join(agents, "reader.json")]) {
      await expect(grant("reader", dest)).rejects.toThrow(/credential destination/i);
      expect(existsSync(dest)).toBe(false);
      expect(existsSync(join(agents, "reader.json"))).toBe(false);
    }
  }));

  it("recognizes linked plugin script folders even when the grant names their outside location", () => fixture(async (root, priv, grant) => {
    const { mkdirSync, readFileSync } = await import("node:fs");
    const container = join(root, "plugins"), main = join(container, "main"), external = join(priv, "external");
    const project = join(priv, "project"), secondary = join(project, "scripts"), sibling = join(priv, "external-private");
    for (const path of [container, main, external, project, secondary, sibling]) mkdirSync(path, { mode: 0o700 });
    symlinkSync(external, join(main, "scripts"), "dir"); symlinkSync(project, join(container, "work"), "dir");
    writeFileSync(join(priv, "readme"), "not a plugin"); symlinkSync(join(priv, "readme"), join(container, "readme"));
    for (const dir of [external, secondary]) {
      const dest = join(dir, "reader.ts");
      await expect(grant("reader", dest)).rejects.toThrow(/credential destination/i);
      expect(existsSync(dest)).toBe(false); expect(existsSync(join(root, "agents/reader.json"))).toBe(false);
    }
    const program = "// Agent effect: Say hello\nconsole.log('hello')\n";
    writeFileSync(join(external, "hello.ts"), program);
    expect(await runCli(["inspect-command", "hello"], { SK_PATH: root, SCRIPT_KIT_MCP_SERVER_JSON: "" }))
      .toMatchObject({ success: true, data: { source: program } });
    await grant("reader", join(sibling, "reader.ts"));
    expect(readFileSync(join(external, "hello.ts"), "utf8")).toBe(program);
    expect(existsSync(join(root, "agents/reader.json"))).toBe(true);
  }));

  it("does not make an individual linked launcher script readable as an agent credential", () => fixture(async (root, priv, grant) => {
    const { lstatSync, mkdirSync } = await import("node:fs");
    const scripts = join(root, "plugins/work/scripts"), dest = join(priv, "note-reader");
    mkdirSync(scripts, { recursive: true, mode: 0o700 });
    symlinkSync(dest, join(priv, "alias")); symlinkSync(join(priv, "alias"), join(scripts, "linked.js"));
    await expect(grant("reader", dest)).rejects.toThrow(/credential destination/i);
    expect(existsSync(dest)).toBe(false); expect(existsSync(join(root, "agents/reader.json"))).toBe(false);
    expect(lstatSync(join(scripts, "linked.js")).isSymbolicLink()).toBe(true);
    await grant("reader", join(priv, "unlinked"));
    expect(existsSync(join(scripts, "linked.js"))).toBe(false);
  }));

  it("refuses an unreadable script catalog without issuing or replacing anything", () => fixture(async (root, priv, grant) => {
    const { lstatSync, mkdirSync } = await import("node:fs");
    const main = join(root, "plugins/main"), scripts = join(main, "scripts"), dest = join(priv, "reader");
    mkdirSync(main, { recursive: true, mode: 0o700 }); symlinkSync("scripts", scripts);
    await expect(grant("reader", dest)).rejects.toThrow(/cannot verify Kit’s linked scripts/i);
    expect(lstatSync(scripts).isSymbolicLink()).toBe(true);
    expect(existsSync(dest)).toBe(false); expect(existsSync(join(root, "agents/reader.json"))).toBe(false);
  }));

  it("does not complete a dangling operator-token link by issuing a scoped credential", () => fixture(async (root, priv, grant) => {
    const { lstatSync } = await import("node:fs");
    const dest = join(priv, "reader"); symlinkSync(dest, join(root, "agent-token"));
    await expect(grant("reader", dest)).rejects.toThrow(/operator token/i);
    expect(existsSync(dest)).toBe(false);
    expect(existsSync(join(root, "agents/reader.json"))).toBe(false);
    expect(lstatSync(join(root, "agent-token")).isSymbolicLink()).toBe(true);
  }));

  it("preserves both destinations on collisions and registers only the hash on success", () => fixture(async (root, priv, grant) => {
    const { lstatSync, mkdirSync, readFileSync } = await import("node:fs");
    const { createHash } = await import("node:crypto");
    const reg = join(root, "agents"), dest = join(priv, "reader"), grantPath = join(reg, "reader.json");
    mkdirSync(reg, { mode: 0o700 }); symlinkSync(join(priv, "missing"), grantPath);
    await expect(grant("reader", dest)).rejects.toThrow();
    expect(existsSync(dest)).toBe(false); expect(lstatSync(grantPath).isSymbolicLink()).toBe(true);
    rmSync(grantPath); writeFileSync(dest, "keep this file", { mode: 0o600 });
    await expect(grant("reader", dest)).rejects.toThrow();
    expect(readFileSync(dest, "utf8")).toBe("keep this file"); expect(existsSync(grantPath)).toBe(false);
    rmSync(dest); await grant("reader", dest);
    const raw = readFileSync(dest, "utf8").trim(), record = readFileSync(grantPath, "utf8");
    expect(lstatSync(dest).mode & 0o777).toBe(0o600);
    expect(record.includes(raw)).toBe(false);
    expect(JSON.parse(record)).toEqual({ tokenHash: createHash("sha256").update(raw).digest("hex"), scopes: ["notes:quiet"] });
    await expect(grant("reader", join(priv, "another"))).rejects.toThrow();
    expect(existsSync(join(priv, "another"))).toBe(false); expect(readFileSync(grantPath, "utf8") === record).toBe(true);
  }));
});

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
      expect(result).toMatchObject({success:true});
      expect((result as any).data.result.tools[0].name).toBe("kit/trigger_builtin");
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("uses the same SK_PATH workspace as the running Script Kit host", async () => {
    const mock = startMockMcp((body) => ({
      jsonrpc: "2.0",
      id: body.id,
      result: { tools: [{ name: "workspace-owned-tool" }] },
    }));
    const { dir } = discoveryEnv(mock.url.origin);
    try {
      const result = await runCli(["mcp", "tools"], {
        SK_PATH: dir,
        SCRIPT_KIT_MCP_SERVER_JSON: "",
        SCRIPT_KIT_MCP_ENDPOINT: "",
        SCRIPT_KIT_MCP_TOKEN: "",
      });
      expect(typeof result).toBe("object");
      expect((result as any).data.result.tools[0].name).toBe("workspace-owned-tool");
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it.each([
    "$SCRIPT_KIT_MCP_FIXTURE_ROOT",
    "${SCRIPT_KIT_MCP_FIXTURE_ROOT}",
  ])("expands host-style SK_PATH variables before private discovery: %s", async (root) => {
    const mock = startMockMcp((body) => ({
      jsonrpc: "2.0",
      id: body.id,
      result: { tools: [{ name: "expanded-workspace-tool" }] },
    }));
    const { dir } = discoveryEnv(mock.url.origin);
    try {
      const result = await runCli(["mcp", "tools"], {
        SK_PATH: root,
        SCRIPT_KIT_MCP_FIXTURE_ROOT: dir,
        SCRIPT_KIT_MCP_SERVER_JSON: "",
        SCRIPT_KIT_MCP_ENDPOINT: "",
        SCRIPT_KIT_MCP_TOKEN: "",
      });
      expect(typeof result).toBe("object");
      expect((result as any).data.result.tools[0].name).toBe("expanded-workspace-tool");
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
    expect(result).toMatchObject({success:true});
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
    expect(result).toMatchObject({success:true});
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

  it.each([
    "https://malicious.example/rpc",
    "http://127.0.0.1.malicious.example/rpc",
    "http://user:secret@127.0.0.1:43129/rpc",
    "ftp://127.0.0.1:43129/rpc",
    "http://127.0.0.1:43129/not-script-kit",
    "http://127.0.0.1:43129/rpc?forward=malicious",
    "http://127.0.0.1:43129/rpc#credentials",
  ])("refuses unsafe MCP endpoint %s before sending the local bearer token", async (endpoint) => {
    let requestCount = 0;
    globalThis.fetch = stubResponse(async () => {
      requestCount += 1;
      return Response.json({ jsonrpc: "2.0", id: "forged", result: {} });
    });

    await expect(runCli(["tools"], {
      SCRIPT_KIT_MCP_ENDPOINT: endpoint,
      SCRIPT_KIT_MCP_TOKEN: "private-local-bearer-token",
    })).rejects.toThrow("local Script Kit MCP endpoint");
    expect(requestCount).toBe(0);
  });

  it("pairs a discovered credential with its exact normalized endpoint before fetching", async () => {
    let dispatched = 0;
    const mock = startMockMcp(body => {
      dispatched++;
      return { jsonrpc: "2.0", id: body.id, result: { tools: [] } };
    });
    const { dir, env } = discoveryEnv(mock.url.origin);
    const implicit = { ...env, SCRIPT_KIT_MCP_TOKEN: "", SCRIPT_KIT_AGENT_CREDENTIAL: "" };
    try {
      for (const url of ["http://127.0.0.1:43130/rpc", "http://localhost:43129/rpc", "https://127.0.0.1:43129/rpc"]) {
        await expect(runCli(["mcp", "tools"], { ...implicit, SCRIPT_KIT_MCP_ENDPOINT: url }))
          .rejects.toThrow("does not match the server that supplied the stored credential");
      }
      writeFileSync(join(dir, "server.json"), JSON.stringify({ token: "test-token" }));
      await expect(runCli(["mcp", "tools"], { ...implicit, SCRIPT_KIT_MCP_ENDPOINT: mock.url.origin }))
        .rejects.toThrow("does not match the server that supplied the stored credential");
      expect(dispatched).toBe(0);
      writeFileSync(join(dir, "server.json"), JSON.stringify({ url: mock.url.origin, token: "test-token" }));
      const r = await runCli(["mcp", "tools"], { ...implicit, SCRIPT_KIT_MCP_ENDPOINT: mock.url.origin + "/rpc" });
      expect(r).toMatchObject({ success: true });
      expect(dispatched).toBe(1);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("refuses a remote endpoint supplied by private server discovery", async () => {
    const { dir, env } = discoveryEnv("https://malicious.example");
    let requestCount = 0;
    globalThis.fetch = stubResponse(async () => {
      requestCount += 1;
      return Response.json({ jsonrpc: "2.0", id: "forged", result: {} });
    });
    try {
      await expect(runCli(["tools"], env)).rejects.toThrow("local Script Kit MCP endpoint");
      expect(requestCount).toBe(0);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it.each(["server.json", "server-location.json"])("does not expose malformed private %s contents", async name => {
    const { dir, env } = discoveryEnv("http://127.0.0.1:43129");
    const canary = "PrivateFixture" + crypto.randomUUID().replaceAll("-", "");
    let requests = 0;
    startMockMcp(body => { requests++; return { jsonrpc: "2.0", id: body.id, result: {} }; });
    try {
      writeFileSync(join(dir, name), '{"token":' + canary + '}', { mode: 0o600 });
      const message = await runCli(["mcp", "tools"], { ...env, SCRIPT_KIT_MCP_ENDPOINT: "",
        SCRIPT_KIT_AGENT_CREDENTIAL: "", SCRIPT_KIT_MCP_TOKEN: name === "server.json" ? "" : "scoped-fixture" })
        .then(() => "", (err: unknown) => err instanceof Error ? err.message : "");
      // Assert on facts, not the raw failure: a failing regression must not print the canary.
      expect({ disclosed: message.includes(canary), malformed: message.includes("invalid JSON"),
        recovery: message.includes("Restart Script Kit"), requests }).toEqual({
        disclosed: false, malformed: true, recovery: true, requests: 0,
      });
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("refuses symlinked bearer-token discovery before reading its external owner", async () => {
    const { dir } = discoveryEnv("http://127.0.0.1:43129");
    const alias = join(dir, "aliased-server.json");
    symlinkSync(join(dir, "server.json"), alias);
    try {
      await expect(runCli(["tools"], {
        SCRIPT_KIT_MCP_SERVER_JSON: alias,
      })).rejects.toThrow("private regular discovery file");
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("refuses bearer-token discovery readable by other users", async () => {
    const { dir, env } = discoveryEnv("http://127.0.0.1:43129");
    chmodSync(join(dir, "server.json"), 0o644);
    try {
      await expect(runCli(["tools"], env)).rejects.toThrow("owner-only permissions");
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("reports JSON-RPC failures as failures instead of successful MCP calls", async () => {
    const mock = startMockMcp((body) => ({
      jsonrpc: "2.0",
      id: body.id,
      error: { code: -32601, message: "tool is unavailable" },
    }));
    await expect(runCli(["tools"], {
      SCRIPT_KIT_MCP_ENDPOINT: mock.url.origin,
      SCRIPT_KIT_MCP_TOKEN: "test-token",
    })).rejects.toThrow("tool is unavailable");
  });

  it.each(["localhost", "[::1]"])(
    "preserves supported loopback MCP endpoint %s",
    async (host) => {
      startMockMcp((body) => ({ jsonrpc: "2.0", id: body.id, result: { tools: [] } }));
      const result = await runCli(["tools"], {
        SCRIPT_KIT_MCP_ENDPOINT: `http://${host}:43129`,
        SCRIPT_KIT_MCP_TOKEN: "test-token",
      });
      expect(typeof result).toBe("object");
      expect((result as any).success).toBe(true);
    },
  );

  it("never follows a redirect with the private local bearer token", async () => {
    globalThis.fetch = (async (input: RequestInfo | URL, init?: RequestInit) => {
      expect(init?.redirect).toBe("error");
      const body = await new Request(input, init).json() as { id: string };
      return Response.json({ jsonrpc: "2.0", id: body.id, result: {} });
    }) as typeof fetch;
    const result = await runCli(["tools"], {
      SCRIPT_KIT_MCP_ENDPOINT: "http://127.0.0.1:43129/rpc",
      SCRIPT_KIT_MCP_TOKEN: "private-local-bearer-token",
    });
    expect(typeof result).toBe("object");
  });

  it.each(["non-json", "http-error", "json-rpc-error"])(
    "never exposes the local bearer credential in %s diagnostics",
    async (mode) => {
      const secret = "private-local-bearer-token-should-never-appear";
      globalThis.fetch = (async (input: RequestInfo | URL, init?: RequestInit) => {
        if (mode === "non-json") {
          return new Response(`upstream echoed ${secret}`, { status: 502 });
        }
        if (mode === "http-error") {
          return Response.json({ error: `Bearer ${secret}` }, { status: 401 });
        }
        const body = await new Request(input, init).json() as { id: string };
        return Response.json({
          jsonrpc: "2.0",
          id: body.id,
          error: { code: -32001, message: `auth failed for Bearer ${secret}` },
        });
      }) as typeof fetch;

      let failure: unknown;
      try {
        await runCli(["tools"], {
          SCRIPT_KIT_MCP_ENDPOINT: "http://127.0.0.1:43129/rpc",
          SCRIPT_KIT_MCP_TOKEN: secret,
        });
      } catch (error) {
        failure = error;
      }
      expect(failure).toBeInstanceOf(Error);
      expect((failure as Error).message).not.toContain(secret);
    },
  );

  it.each([
    ["wrong-id", (id: string) => ({ jsonrpc: "2.0", id: `${id}-stale`, result: {} })],
    ["wrong-protocol", (id: string) => ({ jsonrpc: "1.0", id, result: {} })],
    ["missing-outcome", (id: string) => ({ jsonrpc: "2.0", id })],
    ["contradictory-outcome", (id: string) => ({ jsonrpc: "2.0", id, result: {}, error: { code: 1, message: "failed" } })],
  ])("refuses mismatched or malformed JSON-RPC response %s", async (_name, response) => {
    const mock = startMockMcp((body) => response(body.id));
    await expect(runCli(["tools"], {
      SCRIPT_KIT_MCP_ENDPOINT: mock.url.origin,
      SCRIPT_KIT_MCP_TOKEN: "test-token",
    })).rejects.toThrow("invalid JSON-RPC response");
  });

  it("bounds an unresponsive MCP request without using a socket or leaving a timer", async () => {
    globalThis.fetch = ((_input: RequestInfo | URL, init?: RequestInit) =>
      new Promise<Response>((_resolve, reject) => {
        init?.signal?.addEventListener("abort", () => {
          reject(new DOMException("The operation was aborted", "AbortError"));
        }, { once: true });
      })) as typeof fetch;

    const started = performance.now();
    await expect(runCli(["tools"], {
      SCRIPT_KIT_MCP_ENDPOINT: "http://127.0.0.1:43129/rpc",
      SCRIPT_KIT_MCP_TOKEN: "test-token",
      SCRIPT_KIT_MCP_TIMEOUT_MS: "25",
    })).rejects.toThrow("timed out after 25ms");
    expect(performance.now() - started).toBeLessThan(1_000);
  });

  it("rejects a declared oversized MCP response before accepting its body", async () => {
    globalThis.fetch = stubResponse(async () => new Response("x".repeat(257), {
      headers: { "content-length": "257" },
    }));
    await expect(runCli(["tools"], {
      SCRIPT_KIT_MCP_ENDPOINT: "http://127.0.0.1:43129/rpc",
      SCRIPT_KIT_MCP_TOKEN: "test-token",
      SCRIPT_KIT_MCP_MAX_RESPONSE_BYTES: "256",
    })).rejects.toThrow("256-byte response safety budget");
  });

  it("cancels an oversized streamed MCP response without buffering unbounded bytes", async () => {
    let cancelled = false;
    let pullCount = 0;
    globalThis.fetch = stubResponse(async () => new Response(new ReadableStream<Uint8Array>({
      pull(controller) {
        pullCount += 1;
        controller.enqueue(new Uint8Array(pullCount === 1 ? 128 : 129));
        if (pullCount >= 8) controller.close();
      },
      cancel() { cancelled = true; },
    })));
    await expect(runCli(["tools"], {
      SCRIPT_KIT_MCP_ENDPOINT: "http://127.0.0.1:43129/rpc",
      SCRIPT_KIT_MCP_TOKEN: "test-token",
      SCRIPT_KIT_MCP_MAX_RESPONSE_BYTES: "256",
    })).rejects.toThrow("256-byte response safety budget");
    expect(cancelled).toBe(true);
  });

  it.each(["0", "-1", "1.5", "67108865", "invalid"])(
    "refuses invalid MCP response budget %s before fetching",
    async (budget) => {
      let requestCount = 0;
      globalThis.fetch = stubResponse(async () => {
        requestCount += 1;
        return Response.json({});
      });
      await expect(runCli(["tools"], {
        SCRIPT_KIT_MCP_ENDPOINT: "http://127.0.0.1:43129/rpc",
        SCRIPT_KIT_MCP_TOKEN: "test-token",
        SCRIPT_KIT_MCP_MAX_RESPONSE_BYTES: budget,
      })).rejects.toThrow("SCRIPT_KIT_MCP_MAX_RESPONSE_BYTES");
      expect(requestCount).toBe(0);
    },
  );

  it.each(["0", "-1", "1.5", "120001", "invalid"])(
    "refuses invalid MCP request timeout %s before fetching",
    async (timeout) => {
      let requestCount = 0;
      globalThis.fetch = stubResponse(async () => {
        requestCount += 1;
        return Response.json({});
      });
      await expect(runCli(["tools"], {
        SCRIPT_KIT_MCP_ENDPOINT: "http://127.0.0.1:43129/rpc",
        SCRIPT_KIT_MCP_TOKEN: "test-token",
        SCRIPT_KIT_MCP_TIMEOUT_MS: timeout,
      })).rejects.toThrow("SCRIPT_KIT_MCP_TIMEOUT_MS");
      expect(requestCount).toBe(0);
    },
  );

  it("installs the real file from a path containing spaces, # and %, without replacing someone else's command", async () => {
    const dir = mkdtempSync(join(tmpdir(), "Kit #100% "));
    const source = join(dir, "personal cli.ts"), destination = join(dir, "scriptkit");
    try {
      copyFileSync(new URL("./mcp-cli.ts", import.meta.url), source);
      const install = () => Bun.spawnSync([process.execPath, source, "install-command", destination]);
      expect(install().exitCode).toBe(0);
      expect(realpathSync(destination)).toBe(realpathSync(source));
      expect(install().exitCode).toBe(0);
      rmSync(destination);
      const foreign = join(dir,"missing-unrelated-command"); symlinkSync(foreign,destination);
      expect(install().exitCode).not.toBe(0);
      expect(readlinkSync(destination)).toBe(foreign);
    } finally { rmSync(dir,{recursive:true,force:true}); }
  });

  it("does not certify a different executable reached through a link followed by '..'", async () => {
    const { linkSync, lstatSync, mkdirSync, statSync } = await import("node:fs");
    const root = realpathSync(mkdtempSync(join(tmpdir(), "kit-command-identity-")));
    try {
      const owned = join(root, "Kit #1"), foreign = join(root, "another"), bin = join(root, "bin");
      for (const dir of [owned, join(foreign, "nested"), bin]) mkdirSync(dir, { recursive: true });
      const source = join(owned, "scriptkit"), other = join(foreign, "scriptkit"), target = join(bin, "scriptkit");
      copyFileSync(new URL("./mcp-cli.ts", import.meta.url), source);
      writeFileSync(other, "a different program");
      symlinkSync(join(foreign, "nested"), join(owned, "detour"));
      const spelling = owned + "/detour/../scriptkit";
      symlinkSync(spelling, target);
      const install = () => Bun.spawnSync([process.execPath, source, "install-command", target]);
      expect(statSync(target).ino).toBe(statSync(other).ino);
      expect(statSync(target).ino).not.toBe(statSync(source).ino);
      expect(install().exitCode).not.toBe(0);
      expect(readlinkSync(target)).toBe(spelling);
      expect(await Bun.file(other).text()).toBe("a different program");
      // The same inode under a second filename will not follow a future Kit upgrade.
      rmSync(target); rmSync(other); linkSync(source, other); symlinkSync(other, target);
      expect(statSync(target).ino).toBe(statSync(source).ino);
      expect(install().exitCode).not.toBe(0);
      // A genuine direct relative link remains valid and isn't replaced.
      rmSync(target); symlinkSync("../Kit #1/scriptkit", target);
      const id = lstatSync(target).ino;
      expect(install().exitCode).toBe(0);
      expect(lstatSync(target).ino).toBe(id);
    } finally { rmSync(root, { recursive: true, force: true }); }
  });

  it("keeps explicit command names exact and never installs for a malformed request or help", () => {
    const home = realpathSync(mkdtempSync(join(tmpdir(), "kit-command-target-")));
    try {
      const source = join(home, "cli.ts"), target = join(home, " command "), defaultLink = join(home, ".local/bin/scriptkit");
      copyFileSync(new URL("./mcp-cli.ts", import.meta.url), source);
      const invoke = (...args: string[]) => Bun.spawnSync([process.execPath, source, "install-command", ...args], { cwd: home, env: { HOME: home, PATH: "/usr/bin:/bin" } });
      expect(invoke(target).exitCode).toBe(0);
      expect(readlinkSync(target)).toBe(source);
      expect(existsSync(target.trim())).toBe(false);
      for (const args of [[""], ["   "], ["unused/../misdirected"], ["folder/"], ["unused/target", "extra"], ["--wrong-flag"]]) {
        expect(invoke(...args).exitCode).not.toBe(0);
      }
      const help = invoke("--help");
      expect(help.exitCode).toBe(0);
      expect(help.stdout.toString()).toContain("install-command [target-path]");
      for (const p of [".local", "unused", "misdirected", "folder", "--help", "--wrong-flag"]) {
        expect(existsSync(join(home, p))).toBe(false);
      }
      expect(invoke().exitCode).toBe(0);
      expect(readlinkSync(defaultLink)).toBe(source);
    } finally { rmSync(home, { recursive: true, force: true }); }
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
