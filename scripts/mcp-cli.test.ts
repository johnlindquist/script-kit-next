import { afterEach, describe, expect, it } from "bun:test";
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  readlinkSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { runMcpCli } from "./mcp-cli";

import { agent, operator } from "./agent-bridge-cli";

const stockLauncherRules = { pinnedText:"muted", pinnedWeight:"regular", selectionFirst:null, clipboardFirst:null, homeMode:"standard" };

describe("scoped launcher author", () => {
  it("requires explicit grants, pins the operation to the selected verb, and never retries a refusal", async () => {
    const dir=mkdtempSync(join(tmpdir(),"kit-launcher-client-")),file=join(dir,"credential");
    const previous=process.env.SCRIPT_KIT_AGENT_CREDENTIAL;
    try {
      expect(()=>operator(["grant-agent","wild",file,"launcher:*"],dir)).toThrow();
      expect(existsSync(file)).toBe(false);
      operator(["grant-agent","reader",file,"launcher:read"],dir);
      expect(JSON.parse(readFileSync(join(dir,"agents/reader.json"),"utf8")).scopes).toEqual(["launcher:read"]);
      process.env.SCRIPT_KIT_AGENT_CREDENTIAL=file;
      const seen:any[]=[];
      const rpc=async(method:string, params:any, token:string)=>{
        expect(method).toBe("tools/call"); expect(token).toBe(readFileSync(file,"utf8").trim());seen.push(params);
        const denied=params.name==="kit/launcher_apply";
        const state={workspace:"a".repeat(64),revision:"stock",enabled:true,declared:stockLauncherRules,effective:stockLauncherRules,undo:null};
        const data=params.arguments.op==="preview"?{scope:"preview",wouldStore:{...state,enabled:false}}:{scope:"stored",safeMode:false,state};
        return {result:{isError:denied,content:[{type:"text",text:JSON.stringify(denied?{ok:false,error:{code:"scope_denied"}}:{ok:true,data})}]}};
      };
      const body=(v:unknown)=>(async function*(){yield JSON.stringify(v);})();
      await expect(agent(["launcher","inspect",""],rpc,body(null))).rejects.toThrow("Usage:");
      expect(await agent(["launcher","inspect"],rpc,body(null))).toMatchObject({scope:"stored",state:{revision:"stock"}});
      await agent(["launcher","preview"],rpc,body({change:{op:"disable"}}));
      await expect(agent(["launcher","preview"],rpc,body({op:"apply",change:{op:"disable"}}))).rejects.toThrow("Expected one JSON field");
      await expect(agent(["launcher","apply"],rpc,body({request:{requestId:"fixture"}}))).rejects.toThrow("scope_denied");
      expect(seen).toEqual([{name:"kit/launcher_read",arguments:{op:"inspect"}},{name:"kit/launcher_read",arguments:{op:"preview",change:{op:"disable"}}},{name:"kit/launcher_apply",arguments:{op:"apply",request:{requestId:"fixture"}}}]);
    } finally {if(previous===undefined)delete process.env.SCRIPT_KIT_AGENT_CREDENTIAL;else process.env.SCRIPT_KIT_AGENT_CREDENTIAL=previous;rmSync(dir,{recursive:true,force:true});}
  });
});

describe("launcher receipt identity", () => {
  const id = "9b78c807-2234-4b67-a504-ff7952cab7dd", other = "8ea40dac-f986-4b15-83a5-bd19ff64e4bc";
  const workspace = "a".repeat(64), first = "b".repeat(64), later = "c".repeat(64);
  const state = (revision: string, enabled = true) => ({ workspace, revision, enabled, declared:stockLauncherRules, effective:stockLauncherRules, undo:id });
  const request = { workspace, expectedRevision: "stock", requestId: id.toUpperCase(), change: { op: "disable" } };
  const success = { requestId: id, appliedRevision: first, replayed: false, state: state(first, false) };
  async function exercise(fn: (send: (verb: string, body: unknown, data: unknown) => Promise<unknown>) => Promise<void>) {
    const old = process.env.SCRIPT_KIT_MCP_TOKEN, credential = process.env.SCRIPT_KIT_AGENT_CREDENTIAL;
    delete process.env.SCRIPT_KIT_AGENT_CREDENTIAL; process.env.SCRIPT_KIT_MCP_TOKEN = "synthetic-receipt-test";
    try {
      await fn(async (verb, body, data) => {
        let calls = 0;
        try {
          return await agent(["launcher",verb],async () => {
            calls++;
            return {result:{content:[{type:"text",text:JSON.stringify({ok:true,data})}]}};
          },(async function*(){if(body!==null)yield JSON.stringify(body);})());
        } finally { expect(calls).toBe(1); }
      });
    } finally {
      if(old===undefined)delete process.env.SCRIPT_KIT_MCP_TOKEN;else process.env.SCRIPT_KIT_MCP_TOKEN=old;
      if(credential!==undefined)process.env.SCRIPT_KIT_AGENT_CREDENTIAL=credential;
    }
  }
  it("does not report a stored change for another request, workspace or revision", async () => {
    await exercise(async send => {
      for (const bad of [{...success,requestId:other},{...success,state:{...state(first),workspace:later}},
        {...success,state:state(later)},{...success,replayed:"true"},{...success,appliedRevision:"stock"}]) {
        await expect(send("apply",{request},{scope:"stored",result:bad})).rejects.toThrow(/status/i);
      }
      await expect(send("apply",{request},{scope:"preview",result:success})).rejects.toThrow(/status/i);
      await expect(send("apply",{request:{...request,expectedRevision:first}},{scope:"stored",result:success})).rejects.toThrow(/status/i);
    });
  });
  it("accepts an original commit and a replay whose current state has since advanced", async () => {
    await exercise(async send => {
      for(const result of [success,{...success,replayed:true,state:state(later)}]) {
        const data={scope:"stored",result};expect(await send("apply",{request},data)).toEqual(data);
      }
    });
  });
  it("compares the requested list with the proposed or just-committed declaration, not a later replay snapshot", async () => {
    await exercise(async send => {
      const list = { source:"kit.noteTitles", filter:"pinned", sort:"title", detail:"lastChanged", enter:"openNote" };
      const view = { id:"reading", title:"Reading list", limit:8, list };
      const change = { op:"putView", view }, remove = { op:"removeView", id:view.id };
      const exact = { list, limit:8, title:view.title, id:view.id }; // JSON member order is not identity.
      const wider = { ...view, list:{ ...list, filter:"all" } };
      const preview = (views: unknown) => ({scope:"preview",wouldStore:{...state(first),views}});
      expect(await send("preview",{change},preview([exact]))).toEqual(preview([exact]));
      for (const views of [[wider],[],[view,view],null]) await expect(send("preview",{change},preview(views))).rejects.toThrow();
      for (const views of [[view],[null],null]) await expect(send("preview",{change:remove},preview(views))).rejects.toThrow();
      const empty={scope:"preview",wouldStore:state(first)};
      expect(await send("preview",{change:remove},empty)).toEqual(empty);
      const reply = (views: unknown[], replayed: boolean, now = first) => ({scope:"stored",result:{...success,replayed,state:{...state(now),views}}});
      for (const replayed of [false,true]) await expect(send("apply",{request:{...request,change}},reply([wider],replayed))).rejects.toThrow(/status/i);
      await expect(send("apply",{request:{...request,change:remove}},reply([view],false))).rejects.toThrow(/status/i);
      for (const result of [reply([exact],false),reply([],true,later)]) expect(await send("apply",{request:{...request,change}},result)).toEqual(result);
    });
  });
  it("checks the requested switch or rule while distinguishing inactive rules from effective ones", async () => {
    await exercise(async send => {
      for (const enabled of [true,false]) {
        const change={op:enabled?"enable":"disable"}, mismatch=state(first,!enabled);
        await expect(send("preview",{change},{scope:"preview",wouldStore:mismatch})).rejects.toThrow();
        await expect(send("apply",{request:{...request,change}},{scope:"stored",result:{...success,state:mismatch}})).rejects.toThrow(/status/i);
      }
      for (const [target,field,value] of [
        ["launcher.header.pinned-label.textRole","pinnedText","primary"], ["launcher.header.pinned-label.weight","pinnedWeight","semiboldWhileDictating"],
        ["launcher.home.mode","homeMode","minimal"],
        ["launcher.subject.selection.firstAction","selectionFirst","note"], ["launcher.subject.clipboard.firstAction","clipboardFirst",null],
      ] as const) {
        const change = {op:"edit",edit:{target,value}};
        const proposed=(enabled:boolean,declared:Record<string,unknown>,effective:Record<string,unknown>)=>({scope:"preview",wouldStore:{...state(first,enabled),declared,effective}});
        const set = {...stockLauncherRules,[field]:value};
        await expect(send("preview",{change},proposed(false,{},set))).rejects.toThrow();
        await expect(send("preview",{change},proposed(true,set,{}))).rejects.toThrow();
        for(const r of [proposed(false,set,stockLauncherRules),proposed(true,set,set)]) expect(await send("preview",{change},r)).toEqual(r);
      }
    });
  });
  it("distinguishes a preview, stored state, running window, status and explanation", async () => {
    await exercise(async send => {
      const stored={scope:"stored",safeMode:false,state:state(first)};
      const preview={scope:"preview",wouldStore:state(first,false)};
      expect(await send("inspect",null,stored)).toEqual(stored);
      expect(await send("preview",{change:{op:"disable"}},preview)).toEqual(preview);
      expect(await send("live",null,{scope:"runningWindow",available:false})).toEqual({scope:"runningWindow",available:false});
      await expect(send("inspect",null,preview)).rejects.toThrow();
      await expect(send("preview",{change:{op:"disable"}},stored)).rejects.toThrow();
      await expect(send("live",null,stored)).rejects.toThrow();
      await expect(send("live",null,{scope:"runningWindow",available:true})).rejects.toThrow();
      const status={requestId:id,storedRevision:later,status:"committed"};
      expect(await send("status",{requestId:id.toUpperCase()},status)).toEqual(status);
      await expect(send("status",{requestId:other},status)).rejects.toThrow();
      await expect(send("why",{target:"launcher.header.pinned-label.weight"},{target:"launcher.header.pinned-label.textRole",origin:"stock",revision:first})).rejects.toThrow();
    });
  });
});

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

async function runCli(args: string[], env: Record<string, string | undefined>) {
  const previous = new Map<string, string | undefined>();
  for (const [key, value] of Object.entries(env)) {
    previous.set(key, process.env[key]);
    if (value === undefined) delete process.env[key];
    else process.env[key] = value;
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

async function launcherToolFixture(route: string, tool: string, args: {op: string; [key: string]: unknown}, result: unknown) {
  let calls = 0;
  const check = (params: unknown) => { calls++; expect(params).toEqual({name:tool,arguments:args}); };
  const mock = startMockMcp(wire => { check(wire.params); return {jsonrpc:"2.0",id:wire.id,result}; });
  const previous = process.env.SCRIPT_KIT_MCP_TOKEN, file = process.env.SCRIPT_KIT_AGENT_CREDENTIAL;
  delete process.env.SCRIPT_KIT_AGENT_CREDENTIAL;
  process.env.SCRIPT_KIT_MCP_TOKEN = "test-token";
  try {
    if (route === "named") return await runCli(["mcp","call",tool,JSON.stringify(args)],{SCRIPT_KIT_MCP_ENDPOINT:mock.url.href});
    const { op, ...input } = args;
    return await agent(["launcher",op],async (method, params) => {
      expect(method).toBe("tools/call"); check(params); return {result};
    },(async function*(){yield JSON.stringify(input);})());
  } finally {
    mock.stop();
    if (previous === undefined) delete process.env.SCRIPT_KIT_MCP_TOKEN; else process.env.SCRIPT_KIT_MCP_TOKEN=previous;
    if (file !== undefined) process.env.SCRIPT_KIT_AGENT_CREDENTIAL=file;
    expect(calls).toBe(1);
  }
}

describe("effective launcher rules and the next Undo", () => {
  const requestId = "dc2a64d7-7a79-47c6-b47a-72f9cb917873", tip = "9d8732a9-a4da-46aa-879a-bc50dd396cf9";
  const workspace = "a".repeat(64), appliedRevision = "b".repeat(64), newer = "c".repeat(64);
  const declared = {...stockLauncherRules,pinnedWeight:"semiboldWhileDictating",selectionFirst:"note"};
  const state = { workspace, revision:appliedRevision, declared, effective:stockLauncherRules, enabled:false, undo:requestId };
  for (const route of ["typed","named"]) {
    const call = (verb:string, change:unknown, snapshot:unknown, replayed=false) => {
      const request = {workspace,expectedRevision:"stock",requestId,change};
      const args = verb === "apply" ? {op:verb,request} : {op:verb,change};
      const result = {requestId,appliedRevision,replayed,state:snapshot};
      const body = {ok:true,data:verb === "apply" ? {scope:"stored",result} : {scope:"preview",wouldStore:snapshot}};
      return launcherToolFixture(route,verb === "apply" ? "kit/launcher_apply" : "kit/launcher_read",args,
        {structuredContent:body,isError:false,content:[{type:"text",text:JSON.stringify(body)}]});
    };
    it(route+": the stock workspace cannot name a change to Undo", async () => {
      const inspect = (undo:string|null) => {
        const body = {ok:true,data:{scope:"stored",safeMode:false,state:{...state,revision:"stock",enabled:true,
          declared:stockLauncherRules,effective:stockLauncherRules,undo}}};
        return launcherToolFixture(route,"kit/launcher_read",{op:"inspect"},{content:[{type:"text",text:JSON.stringify(body)}]});
      };
      await expect(inspect(tip)).rejects.toMatchObject({code:"launcher_read_unverified"});
      expect(await inspect(null)).toBeDefined();
    });
    it(route+": disabled declarations are kept, not effective; enabling restores them", async () => {
      for (const verb of ["preview","apply"]) {
        const code = verb === "apply" ? "launcher_apply_unverified" : "launcher_read_unverified";
        await expect(call(verb,{op:"disable"},{...state,effective:declared})).rejects.toMatchObject({code});
        await expect(call(verb,{op:"enable"},{...state,enabled:true})).rejects.toMatchObject({code});
        expect(await call(verb,{op:"disable"},state)).toBeDefined();
        expect(await call(verb,{op:"enable"},{...state,enabled:true,effective:declared})).toBeDefined();
      }
    });
    it(route+": a new save names itself for Undo; a later replay describes the current tip", async () => {
      for (const replayed of [false,true]) {
        await expect(call("apply",{op:"disable"},{...state,undo:tip},replayed)).rejects.toMatchObject({code:"launcher_apply_unverified"});
        expect(await call("apply",{op:"disable"},state,replayed)).toBeDefined();
      }
      expect(await call("apply",{op:"disable"},{...state,revision:newer,undo:tip,enabled:true,effective:declared},true)).toBeDefined();
    });
    it(route+": Undo preview names the current tip; the save consumes it", async () => {
      const change = {op:"undo",of:tip.toUpperCase()};
      await expect(call("preview",change,state)).rejects.toMatchObject({code:"launcher_read_unverified"});
      expect(await call("preview",change,{...state,undo:tip})).toBeDefined();
      for (const undo of [tip,requestId,undefined,"not-a-uuid"])
        await expect(call("apply",change,{...state,undo})).rejects.toMatchObject({code:"launcher_apply_unverified"});
      for (const undo of [null,"8544bdb9-2443-4b14-a38d-759614f04a6e"])
        expect(await call("apply",change,{...state,undo})).toBeDefined();
    });
  }
});

describe("reconciling a launcher save by its original request ID", () => {
  const requestId = "9d8732a9-a4da-46aa-879a-bc50dd396cf9";
  const args = {op:"status",requestId:requestId.toUpperCase()};
  const status = (value: string, revision: string) => ({ok:true,data:{requestId,status:value,storedRevision:revision}});
  const rev = "a".repeat(64), committed = status("committed",rev), unknown = status("unknown",rev);
  const reply = (body: unknown, extra = {}) => ({content:[{type:"text",text:JSON.stringify(body)}],...extra});
  for (const route of ["typed","named"]) {
    const read = (result: unknown) => launcherToolFixture(route,"kit/launcher_read",args,result);
    it(route + ": cannot confirm a save from a conflicting reply, foreign UUID or empty active history", async () => {
      const denied = {ok:false,error:{code:"scope_denied"}};
      for (const bad of [reply(committed,{structuredContent:unknown}),reply(status("committed","stock")),
        reply({...committed,data:{...committed.data,requestId:"4da13210-6fb8-48b1-b178-c3e9678dda6f"}}),
        reply({...committed,error:denied.error}),reply(committed,{isError:true}),reply(denied,{isError:false}),
        reply(denied,{isError:true,structuredContent:committed}),reply(committed,{isError:"false"})]) {
        await expect(read(bad)).rejects.toMatchObject({code:"launcher_read_unverified",message:expect.stringContaining("status receipt")});
      }
    });
    it(route + ": preserves valid history statuses, the response shape, and an actual read refusal", async () => {
      for (const body of [committed,unknown,status("unknown","stock"),status("notInActiveHistory","stock"),status("notInActiveHistory",rev)]) {
        const result = reply(body,{isError:false,structuredContent:body});
        expect(await read(result)).toEqual(route==="typed" ? body.data : {success:true,data:{jsonrpc:"2.0",id:expect.any(String),result}});
      }
      await expect(read(reply({ok:false,error:{code:"scope_denied"}},{isError:true}))).rejects.toThrow("scope_denied");
      await expect(read(reply({ok:false,error:{code:"private_fixture_diagnostic"}},{isError:true}))).rejects.toThrow(/^Launcher read refused$/);
    });
  }
});

describe("launcher save outcomes", () => {
  const request = { workspace: "a".repeat(64), expectedRevision: "stock", requestId: "dc2a64d7-7a79-47c6-b47a-72f9cb917873", change: {op:"disable"} };
  const args = {op:"apply",request};
  const revision = "b".repeat(64);
  const stored = { scope:"stored",result:{ requestId:request.requestId,appliedRevision:revision,replayed:false,
    state:{workspace:request.workspace,revision,enabled:false,declared:stockLauncherRules,effective:stockLauncherRules,undo:request.requestId} } };
  const reply = (isError: unknown, body: unknown) => ({isError,content:[{type:"text",text:JSON.stringify(body)}]});
  it("reports a local routing refusal before any request goes out", async () => {
    let calls = 0;
    startMockMcp(() => { calls++; throw Error("unexpected dispatch"); });
    await expect(runCli(["mcp","call","kit/launcher_apply",JSON.stringify(args)], {
      SCRIPT_KIT_MCP_ENDPOINT:"https://not-local.example/rpc", SCRIPT_KIT_MCP_TOKEN:"test-token",
    })).rejects.toMatchObject({code:"mcp_request_not_sent"});
    expect(calls).toBe(0);
  });
  for (const route of ["typed","named"]) {
    const send = (result: unknown) => launcherToolFixture(route,"kit/launcher_apply",args,result);
    it(`${route}: a post-save error, contradictory flags or another request's receipt cannot settle this write`, async () => {
      for (const r of [reply(true,{ok:false,error:{code:"storage_unavailable"}}),reply(true,{ok:false,error:{code:"stale_revision"}}),
        reply(false,{ok:false,error:{code:"scope_denied"}}),reply(true,{ok:true,data:stored}),reply("false",{ok:true,data:stored}),
        {...reply(false,{ok:true,data:stored}),structuredContent:{ok:false,error:{code:"scope_denied"}}},
        reply(false,{ok:true,data:stored,error:{code:"storage_unavailable"}}),reply(false,{ok:true,data:{...stored,result:{...stored.result,requestId:"00000000-0000-4000-8000-000000000000"}}})]) {
        await expect(send(r)).rejects.toMatchObject({code:"launcher_apply_unverified",message:expect.stringContaining("original requestId")});
      }
      const error = await send(reply(true,{ok:false,error:{code:"private_fixture_label"}})).then(()=>null, e=>e);
      expect(error.code).toBe("launcher_apply_unverified"); expect(error.message).not.toContain("private_fixture_label");
    });
    it(`${route}: keeps an actual scope refusal distinct and preserves the successful response format`, async () => {
      await expect(send(reply(true,{ok:false,error:{code:"scope_denied"}}))).rejects.toThrow("scope_denied");
      const ok = reply(false,{ok:true,data:stored});
      expect(await send(ok)).toEqual(route==="typed" ? stored : {success:true,data:{jsonrpc:"2.0",id:expect.any(String),result:ok}});
    });
  }
});

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

describe("agent skill installation", () => {
  it("does not turn a directory link and '..' into a different skills directory", async () => {
    const { mkdirSync, readFileSync } = await import("node:fs");
    const root = realpathSync(mkdtempSync(join(tmpdir(), "kit-skill-destination-")));
    const env = { SK_PATH: root, HOME: root, SCRIPT_KIT_MCP_ENDPOINT: undefined, SCRIPT_KIT_MCP_SERVER_JSON: undefined,
      SCRIPT_KIT_MCP_TOKEN: undefined, SCRIPT_KIT_AGENT_CREDENTIAL: undefined };
    try {
      const neighbor = join(root, "skills"), actual = join(root, "another", "skills"), inner = join(root, "another", "inner");
      for (const p of [neighbor, actual, inner]) mkdirSync(p, { recursive: true });
      symlinkSync(inner, join(root, "through"), "dir"); writeFileSync(join(root, "not-a-dir"), "file");
      for (const component of ["through", "missing", "not-a-dir"]) {
        await expect(runCli(["install-skill", root + "/" + component + "/../skills"], env)).rejects.toThrow("without '..'");
        expect(existsSync(join(neighbor, "script-kit"))).toBe(false);
        expect(existsSync(join(actual, "script-kit"))).toBe(false);
      }
      const alias = join(root, "agent skills 資料 "), dir = join(actual, "script-kit"), file = join(dir, "SKILL.md");
      symlinkSync(actual, alias, "dir");
      expect(await runCli(["install-skill", alias], env)).toEqual({ success: true, data: { installed: dir } });
      expect(readFileSync(file)).toEqual(readFileSync(new URL("../skills/script-kit/SKILL.md", import.meta.url)));
      writeFileSync(file, "My existing skill");
      await expect(runCli(["install-skill", alias], env)).rejects.toThrow("not replaced");
      expect(readFileSync(file, "utf8")).toBe("My existing skill");
    } finally { rmSync(root, { recursive: true, force: true }); }
  });

  it("checks the skill source before reserving the destination and preserves its exact UTF-8 bytes", async () => {
    const { mkdirSync, readFileSync } = await import("node:fs");
    const root = realpathSync(mkdtempSync(join(tmpdir(), "kit-skill-source-")));
    try {
      const bin = join(root, "bin"), share = join(root, "share", "external-agent"), directory = join(root, "skills");
      for (const p of [bin, share, directory]) mkdirSync(p, { recursive: true });
      const entry = join(bin, "scriptkit.ts"), src = join(share, "SKILL.md"), dest = join(directory, "script-kit");
      copyFileSync(new URL("./mcp-cli.ts", import.meta.url), entry);
      copyFileSync(new URL("./agent-bridge-cli.ts", import.meta.url), join(bin, "agent-bridge-cli.ts"));
      const install = (target = directory, ...flags: string[]) => Bun.spawnSync([process.execPath, "--config=/dev/null", "--no-env-file", "--no-install", "--no-macros", ...flags, entry, "install-skill", target],
        { cwd: root, env: { HOME: root, SK_PATH: root, PATH: "/usr/bin:/bin" } });
      const checkRefusal = () => {
        const r = install(); expect(r.exitCode).not.toBe(0); expect(existsSync(dest)).toBe(false);
        expect(r.stdout.toString()).toContain("No skill was installed");
      };
      checkRefusal();
      const marker = join(root, "unrelated-file"); writeFileSync(marker, "Not agent instructions");
      symlinkSync(marker, src); checkRefusal(); rmSync(src);
      writeFileSync(src, Buffer.from([0x96])); checkRefusal();
      const original = Buffer.from("\ufeff---\r\nname: cafe\u0301 🧪\r\n---\r\n");
      writeFileSync(src, original); expect(install().exitCode).toBe(0);
      expect(readFileSync(join(dest, "SKILL.md"))).toEqual(original);

      // Another writer can keep the same inode and byte count. Change the real file
      // immediately after the installer's write, without touching the test runner's fs.
      const racing = join(root, "racing-skills"), hook = join(root, "writer.cjs"), written = join(racing, "script-kit", "SKILL.md");
      mkdirSync(racing);
      writeFileSync(hook,
        `const fs = require('node:fs'), put = fs.writeFileSync.bind(fs), p = ${JSON.stringify(written)};
        fs.writeFileSync = (fd, ...args) => { const result = put(fd, ...args);
          if (fs.existsSync(p) && (fd === p || (typeof fd === 'number' && fs.fstatSync(fd).ino === fs.statSync(p).ino)))
            put(p, Buffer.alloc(fs.statSync(p).size, 88));
          return result; }; require('node:module').syncBuiltinESMExports();`);
      const contested = install(racing, "--preload", hook);
      expect(contested.exitCode).not.toBe(0); expect(contested.stdout.toString()).toContain("Could not confirm");
      expect(readFileSync(written)).toEqual(Buffer.alloc(original.length, 88));
    } finally { rmSync(root, { recursive: true, force: true }); }
  });
});

describe("workspace path identity", () => {
  const local = { SCRIPT_KIT_MCP_ENDPOINT: undefined, SCRIPT_KIT_MCP_SERVER_JSON: undefined, SCRIPT_KIT_MCP_TOKEN: undefined, SCRIPT_KIT_AGENT_CREDENTIAL: undefined };

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

describe("local pane pairing", () => {
  it.skipIf(process.platform !== "darwin")("withdrawal retires only the observed provider grant, even when the exchange pathname is reused", async () => {
    const { linkSync, lstatSync, renameSync, unlinkSync } = await import("node:fs");
    const dir = mkdtempSync(join(realpathSync(tmpdir()), "kit-withdraw-"));
    try {
      const kit = join(dir, "kit"), other = join(dir, "other"), home = join(dir, "herdr");
      for (const p of [kit, other, home]) mkdirSync(p, { mode: 0o700 });
      const run = (root: string, env: Record<string, string>, ...args: string[]) => {
        const child = Bun.spawnSync([process.execPath, "--config=/dev/null", "--no-env-file", "--no-install", "--no-macros",
          join(import.meta.dir, "mcp-cli.ts"), "panes", ...args], {
          cwd: dir,
          env: { HOME: dir, TMPDIR: dir, SK_PATH: root, PATH: "/usr/bin:/bin", SCRIPT_KIT_NONINTERACTIVE: "1", ...env },
        });
        const reply = JSON.parse(child.stdout.toString());
        expect(child.exitCode).toBe(reply.success ? 0 : 1);
        return reply.data;
      };
      const cli = (...args: string[]) => run(kit, {}, ...args);
      const exchange = join(home, "kit-pane-share-v1"), grant = join(exchange, "grant.json");
      expect(cli("offer-status", home).state).toBe("noOffer");
      expect(existsSync(exchange)).toBe(false);
      const first = cli("offer", home, "first");
      expect(first.offerId).toMatch(/^[a-f0-9]{64}$/);
      expect(cli("offer-status", home)).toEqual(first);
      renameSync(exchange, join(home, "earlier-offer"));
      const second = cli("offer", home, "second");
      const original = readFileSync(grant), inode = lstatSync(grant).ino;
      expect(cli("withdraw", home, first.offerId).state).toBe("offerChanged");
      expect(run(other, {}, "withdraw", home, second.offerId).state).toBe("offerWorkspaceMismatch");
      expect(run(kit, { SCRIPT_KIT_AGENT_CREDENTIAL: "" }, "withdraw", home, second.offerId).state).toBe("agentContextDenied");
      linkSync(grant, join(dir, "other-name"));
      expect(cli("withdraw", home, second.offerId).state).toBe("unverifiedSource");
      unlinkSync(join(dir, "other-name"));
      expect(readFileSync(grant)).toEqual(original);
      const withdrawn = run(kit, { SCRIPT_KIT_PERSONAL_SAFE_MODE: "1" }, "withdraw", home, second.offerId);
      expect(withdrawn).toMatchObject({ state: "withdrawn", offerId: second.offerId, space: "second", workspace: first.workspace });
      expect(withdrawn).not.toHaveProperty("marker");
      expect(lstatSync(grant).ino).toBe(inode);
      expect(JSON.parse(readFileSync(grant, "utf8"))).toEqual({ ...JSON.parse(original.toString()), capability: null });
      const changed = lstatSync(grant, { bigint: true }).mtimeNs;
      expect(cli("offer-status", home)).toEqual(withdrawn);
      expect(cli("withdraw", home, second.offerId)).toEqual(withdrawn);
      expect(lstatSync(grant, { bigint: true }).mtimeNs).toBe(changed);
      expect(cli("offer", home, "new").state).toBe("offerExists");
      expect(readFileSync(join(exchange, "in/request.json"), "utf8")).toBe("{}");
      expect(existsSync(join(kit, "db"))).toBe(false);
    } finally { rmSync(dir, { recursive: true, force: true }); }
  });
  it("does not let a scoped credential offer a space or reach the request file", async () => {
    const root = realpathSync(mkdtempSync(join(tmpdir(), "kit-pane-author-")));
    try {
      for (const args of [["offer",root,"sp1"],["pair",join(root,"out"),"sp1"],["list"],["unpair","a".repeat(32)]]) {
        await expect(runCli(["panes", ...args], {SK_PATH:root, SCRIPT_KIT_AGENT_CREDENTIAL:join(root,"not-a-credential")})).rejects.toThrow("agentContextDenied");
      }
      expect(existsSync(join(root,"kit-pane-share-v1"))).toBe(false);
      expect(existsSync(join(root,"db"))).toBe(false);
    } finally { rmSync(root,{recursive:true,force:true}); }
  });
  it.skipIf(process.platform !== "darwin")("creates only a new offer; a foreign space in a marker cannot pair", async () => {
    const root = realpathSync(mkdtempSync(join(tmpdir(), "kit-pane-pair-")));
    const kit = join(root,"kit"), home=join(root,"source"), env={SK_PATH:kit};
    for(const p of [kit,join(kit,"db"),home])mkdirSync(p,{mode:0o700});
    try {
      expect(await runCli(["panes","status"],env)).toMatchObject({success:true,data:{state:"notPaired"}});
      const result = await runCli(["panes","offer",home,"sp1"],env) as {data:{marker:string;source:string}};
      const grant = join(home,"kit-pane-share-v1/grant.json"), original=readFileSync(grant);
      await expect(runCli(["panes","offer",home,"sp2"],env)).rejects.toThrow("offerExists");
      expect(readFileSync(grant)).toEqual(original);
      writeFileSync(result.data.marker,JSON.stringify({v:1,state:"listening",source:result.data.source,space:"sp2",capability:"herdr.panes.metadata"}),{mode:0o600});
      await expect(runCli(["panes","pair",result.data.marker,"sp1"],env)).rejects.toThrow("unverifiedSource");
      expect(readFileSync(join(home,"kit-pane-share-v1/in/request.json"),"utf8")).toBe("{}");
      expect(existsSync(join(kit,"db/herdr-panes-v1/pair.json"))).toBe(false);
    } finally { rmSync(root,{recursive:true,force:true}); }
  });
});

describe("scoped credential destinations", () => {
  async function fixture(run: (root: string, priv: string, grant: (name: string, file: string) => Promise<unknown>) => Promise<void>) {
    const { mkdirSync } = await import("node:fs");
    const dir = realpathSync(mkdtempSync(join(tmpdir(), "kit-grant-")));
    const root = join(dir, "workspace"), priv = join(dir, "private");
    try {
      for (const path of [root, priv]) mkdirSync(path, { mode: 0o700 });
      const env = { SK_PATH: root, SCRIPT_KIT_AGENT_CREDENTIAL: undefined, SCRIPT_KIT_MCP_TOKEN: undefined, SCRIPT_KIT_MCP_SERVER_JSON: undefined };
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
    expect(await runCli(["inspect-command", "hello"], { SK_PATH: root, SCRIPT_KIT_MCP_SERVER_JSON: undefined }))
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
        SCRIPT_KIT_MCP_SERVER_JSON: undefined,
        SCRIPT_KIT_MCP_ENDPOINT: undefined,
        SCRIPT_KIT_MCP_TOKEN: undefined,
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
        SCRIPT_KIT_MCP_SERVER_JSON: undefined,
        SCRIPT_KIT_MCP_ENDPOINT: undefined,
        SCRIPT_KIT_MCP_TOKEN: undefined,
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
    const implicit = { ...env, SCRIPT_KIT_MCP_TOKEN: undefined, SCRIPT_KIT_AGENT_CREDENTIAL: undefined };
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
      const message = await runCli(["mcp", "tools"], { ...env, SCRIPT_KIT_MCP_ENDPOINT: undefined,
        SCRIPT_KIT_AGENT_CREDENTIAL: undefined, SCRIPT_KIT_MCP_TOKEN: name === "server.json" ? undefined : "scoped-fixture" })
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

  it.skipIf(process.platform === "win32")("keeps project startup code and dotenv out of the shell command", async () => {
    const { mkdirSync } = await import("node:fs");
    const root = realpathSync(mkdtempSync(join(tmpdir(), "kit-cli-startup-")));
    try {
      const home = join(root, "home"), workspace = join(home, ".scriptkit"), other = join(root, "other Kit");
      const bin = join(root, "commands"), caller = join(root, "unrelated project");
      const source = join(workspace, "bin/scriptkit");
      for (const dir of [bin, caller, join(workspace, "bin")]) mkdirSync(dir, { recursive: true, mode: 0o700 });
      for (const [kit, effect] of [[workspace, "original"], [other, "other"]] as const) {
        const scripts = join(kit, "plugins/main/scripts");
        mkdirSync(scripts, { recursive: true, mode: 0o700 });
        writeFileSync(join(scripts, "example.ts"), "// Agent effect: " + effect + "\n", { mode: 0o600 });
      }
      copyFileSync(new URL("./mcp-cli.ts", import.meta.url), source);
      chmodSync(source, 0o700);
      copyFileSync(new URL("./agent-bridge-cli.ts", import.meta.url), join(workspace, "bin/agent-bridge-cli.ts"));
      symlinkSync(realpathSync(process.execPath), join(bin, "bun"));
      const env = { HOME: home, PATH: bin + ":/usr/bin:/bin", TMPDIR: root, SCRIPT_KIT_NONINTERACTIVE: "1", CI: "1" };
      const install = Bun.spawnSync([source, "install-command", join(bin, "scriptkit")], { cwd: home, env });
      expect(install.exitCode).toBe(0);
      const marker = join(caller, "ran-before-cli");
      writeFileSync(join(caller, "project.ts"), "import fs from 'node:fs'; fs.writeFileSync('ran-before-cli', 'ran');\n");
      writeFileSync(join(caller, "bunfig.toml"), 'preload = ["./project.ts"]\n');
      writeFileSync(join(caller, ".env"), "SK_PATH=" + JSON.stringify(other) + "\n");
      const run = (args: string[], overrides: Record<string, string> = {}) => {
        const child = Bun.spawnSync(["/bin/sh", "-c", 'exec scriptkit "$@"', "shell", ...args], {
          cwd: caller, env: { ...env, ...overrides },
        });
        expect(child.exitCode).toBe(0);
        return child.stdout.toString();
      };
      expect(run(["--help"])).toContain("Script Kit command line");
      const inspect = ["inspect-command", "example"];
      const fromDefault = JSON.parse(run(inspect)).data.effect;
      const fromExport = JSON.parse(run(inspect, { SK_PATH: other })).data.effect;
      expect({ projectRan: existsSync(marker), fromDefault, fromExport }).toEqual({
        projectRan: false, fromDefault: "original", fromExport: "other",
      });
    } finally { rmSync(root, { recursive: true, force: true }); }
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

describe("agent text input", () => {
  it("rejects malformed stdin before calling Kit and preserves literal Unicode", () => {
    const bridge = JSON.stringify(new URL("./agent-bridge-cli.ts", import.meta.url).href);
    const probe = `
      import { agent } from ${bridge};
      let calls = 0;
      const rpc = async (_, params) => {
        calls++;
        const text = JSON.stringify({ body: params.arguments.body });
        return { result: { content: [{ type: "text", text }] } };
      };
      try {
        const { body } = await agent(["note", "save", "00000000-0000-4000-8000-000000000001"], rpc);
        console.log(JSON.stringify({ calls, body }));
      } catch (error) { console.log(JSON.stringify({ calls, error: error.message })); }
    `;
    const read = (input: Uint8Array) => {
      const home = mkdtempSync(join(tmpdir(), "kit-stdin-"));
      try {
        const child = Bun.spawnSync([process.execPath, "--config=/dev/null", "--no-install", "--no-env-file", "--no-macros", "-e", probe], {
          stdin: input, timeout: 4000,
          env: { SCRIPT_KIT_MCP_TOKEN: "fixture-only", HOME: home, TMPDIR: home, CI: "1", SCRIPT_KIT_NONINTERACTIVE: "1" },
        });
        expect(child.exitCode).toBe(0);
        return JSON.parse(child.stdout.toString());
      } finally { rmSync(home, { recursive: true, force: true }); }
    };
    expect(read(Uint8Array.of(0x61, 0xc3, 0x28))).toEqual({ calls: 0, error: "Input must be valid UTF-8. No request was sent." });
    const text = "\uFEFFRésumé 🧪 \uFFFD\r\n";
    expect(read(new TextEncoder().encode(text))).toEqual({ calls: 1, body: text });
  });
});

describe("explicit connection context", () => {
  it("does not borrow an operator identity or endpoint when a selected credential or route is blank", async () => {
    const { dir } = discoveryEnv("http://127.0.0.1:43129");
    let calls = 0;
    startMockMcp(body => { calls++; return { jsonrpc: "2.0", id: body.id, result: { tools: [] } }; });
    try {
      writeFileSync(join(dir, "server-location.json"), JSON.stringify({ url: "http://127.0.0.1:43129" }), { mode: 0o600 });
      for (const key of ["SCRIPT_KIT_AGENT_CREDENTIAL", "SCRIPT_KIT_MCP_TOKEN", "SCRIPT_KIT_MCP_ENDPOINT", "SCRIPT_KIT_MCP_SERVER_JSON"]) {
        for (const value of ["", "   "]) {
          await expect(runCli(["mcp", "tools"], { SK_PATH: dir, [key]: value })).rejects.toThrow();
        }
      }
      await expect(runCli(["agent", "discover"], { SK_PATH: dir, SCRIPT_KIT_AGENT_CREDENTIAL: "", SCRIPT_KIT_MCP_TOKEN: "test-token" })).rejects.toThrow(/credential/i);
      expect(calls).toBe(0);
    } finally { rmSync(dir, { recursive: true, force: true }); }
  });
  it("cannot issue a grant or inspect script source from an empty agent credential context", async () => {
    const root = realpathSync(mkdtempSync(join(tmpdir(), "kit-empty-agent-")));
    try {
      for (const key of ["SCRIPT_KIT_AGENT_CREDENTIAL", "SCRIPT_KIT_MCP_TOKEN"]) {
        const env = { SK_PATH: root, [key]: "" };
        await expect(runCli(["grant-agent", "reader", join(root, "new-secret"), "notes:quiet"], env)).rejects.toThrow("operator shell");
        await expect(runCli(["inspect-command", "example"], env)).rejects.toThrow("operator shell");
        expect(existsSync(join(root, "agents"))).toBe(false);
        expect(existsSync(join(root, "new-secret"))).toBe(false);
      }
    } finally { rmSync(root, { recursive: true, force: true }); }
  });
});
