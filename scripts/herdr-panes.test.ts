// macOS-only worker integration, with a synthetic responder in a private directory.
// Run directly: bun test ./scripts/herdr-panes.test.ts. No Herdr or Kit app is started.
import { expect, test } from "bun:test";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

test.skipIf(process.platform !== "darwin")(
  "only the offering workspace can pair, and every read needs a fresh response for that workspace",
  async () => {
    const dir = mkdtempSync(join(realpathSync(tmpdir()), "kit-pane-protocol-"));
    const kit = join(dir, "kit"),
      other = join(dir, "other-kit"),
      home = join(dir, "herdr");
    let timer: ReturnType<typeof setInterval> | undefined;
    try {
      for (const p of [kit, join(kit, "db"), other, join(other, "db"), home]) {
        mkdirSync(p, { mode: 0o700 });
      }
      const at = async (workspace: string, ...args: string[]) => {
        const p = Bun.spawn([
          process.execPath,
          "--config=/dev/null",
          "--no-env-file",
          "--no-install",
          "--no-macros",
          fileURLToPath(new URL("./mcp-cli.ts", import.meta.url)),
          "panes",
          ...args,
        ], {
          cwd: dir,
          env: {
            SK_PATH: workspace,
            HOME: dir,
            TMPDIR: dir,
            PATH: "/usr/bin:/bin",
            BUN_RUNTIME_TRANSPILER_CACHE_PATH: "0",
          },
          stdout: "pipe",
          stderr: "ignore",
        });
        const [code, text] = await Promise.all([
          p.exited,
          new Response(p.stdout).text(),
        ]);
        return { code, reply: JSON.parse(text) };
      };
      const cli = (...args: string[]) => at(kit, ...args);
      const offer = (await cli("offer", home, "sp1")).reply.data;
      const share = join(home, "kit-pane-share-v1");
      expect(offer.state).toBe("offered");
      const grant = JSON.parse(readFileSync(join(share, "grant.json"), "utf8"));
      writeFileSync(
        offer.marker,
        JSON.stringify({ ...grant, state: "listening" }),
        { mode: 0o600 },
      );
      let last = "", bad = "";
      timer = setInterval(() => {
        let q;
        try {
          q = JSON.parse(readFileSync(join(share, "in/request.json"), "utf8"));
        } catch {
          return;
        }
        if (
          q.op !== "list" || q.nonce === last || q.workspace !== grant.workspace
        ) return;
        last = q.nonce;
        const reply = {
          v: grant.v,
          workspace: bad === "workspace" ? "f".repeat(64) : grant.workspace,
          source: q.source,
          space: "sp1",
          nonce: q.nonce,
          state: "ready",
          rows: [{
            pane: "p1",
            tab: "tab1",
            terminal: "t3",
            pid: 123,
            createdMs: 456,
            title: bad === "title" ? "Untrusted title" : "Pane 3",
            status: "blocked",
            focused: false,
            zoomed: false,
          }],
          more: false,
        };
        writeFileSync(
          join(share, "out", `${q.source}-${q.nonce}.json`),
          JSON.stringify(reply),
          { flag: "wx", mode: 0o600 },
        );
      }, 8);
      const foreign = await at(other, "pair", offer.marker, "sp1");
      expect(foreign.code).toBe(1);
      expect(foreign.reply.data).toEqual({ state: "offerWorkspaceMismatch" });
      expect(readFileSync(join(share, "in/request.json"), "utf8")).toBe("{}");
      expect(existsSync(join(other, "db/herdr-panes-v1"))).toBe(false);
      expect(grant.workspace).toBe((await cli("status")).reply.data.workspace);
      const first = await cli("pair", offer.marker, "sp1");
      expect(first.code).toBe(0);
      expect(first.reply.data.state).toBe("paired");
      const paired = first.reply.data.pairId;
      const read = await cli("list", "needsAttention", "attentionFirst", "1");
      expect(read.reply.data.rows[0].title).toBe("Pane 3");
      expect(read.reply.data.pairId).toBe(paired);
      bad = "workspace";
      const crossed = await cli("list");
      expect(crossed.code).toBe(1);
      expect(crossed.reply.data).toEqual({ state: "unverifiedSource" });
      bad = "title";
      const refused = await cli("list");
      expect(refused.code).toBe(1);
      expect(refused.reply.data).toEqual({ state: "unverifiedSource" });
      expect(JSON.stringify(refused.reply)).not.toContain("Untrusted title");
      expect((await cli("status")).reply.data.state).toBe("paired");
      // Old enrollments can be retired, but cannot authorize a fresh read.
      const file = join(kit, "db/herdr-panes-v1/pair.json");
      const saved = JSON.parse(readFileSync(file, "utf8"));
      writeFileSync(file, JSON.stringify({ ...saved, v: 1 }));
      const lastRequest = readFileSync(join(share, "in/request.json"));
      expect((await cli("status")).reply.data).toMatchObject({
        state: "renewalRequired",
        pairId: paired,
      });
      expect((await cli("list")).reply.data).toEqual({
        state: "renewalRequired",
      });
      expect(readFileSync(join(share, "in/request.json"))).toEqual(lastRequest);
      expect((await cli("unpair", paired)).reply.data.state).toBe("unpaired");
      expect((await cli("status")).reply.data.state).toBe("notPaired");
    } finally {
      clearInterval(timer);
      rmSync(dir, { recursive: true, force: true });
    }
  },
  12000,
);
