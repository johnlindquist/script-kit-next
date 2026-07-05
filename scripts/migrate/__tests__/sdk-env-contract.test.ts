import { describe, expect, test } from "bun:test";
import { join } from "node:path";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";

/**
 * Black-box contract the migration engine depends on: v2's env() accepts the
 * v1 config argument ({ hint, secret } or a bare string) and forwards it on
 * the wire, so ports keep v1 env() calls verbatim instead of degrading them.
 * Regression-locks the env v1-compat change in scripts/kit-sdk.ts.
 */

const REPO = join(import.meta.dir, "..", "..", "..");
const SDK = join(REPO, "scripts", "kit-sdk.ts");

async function firstEnvMessage(scriptBody: string): Promise<Record<string, unknown>> {
  const dir = mkdtempSync(join(tmpdir(), "sk-env-contract-"));
  const scriptPath = join(dir, "probe.ts");
  writeFileSync(scriptPath, scriptBody);
  const proc = Bun.spawn(["bun", "--preload", SDK, scriptPath], {
    cwd: REPO,
    env: { ...process.env, SK_PATH: dir },
    stdout: "pipe",
    stderr: "pipe",
  });
  const killer = setTimeout(() => proc.kill(), 10_000);
  const reader = proc.stdout.getReader();
  const decoder = new TextDecoder();
  let buffer = "";
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      buffer += decoder.decode(value, { stream: true });
      let nl: number;
      while ((nl = buffer.indexOf("\n")) !== -1) {
        const line = buffer.slice(0, nl).trim();
        buffer = buffer.slice(nl + 1);
        if (!line) continue;
        try {
          const msg = JSON.parse(line);
          if (msg?.type === "env") {
            clearTimeout(killer);
            proc.kill();
            return msg;
          }
        } catch {}
      }
    }
  } catch {}
  clearTimeout(killer);
  proc.kill();
  throw new Error("no env message observed");
}

describe("SDK env() v1 config compatibility (runs the real SDK)", () => {
  test("options object forwards hint as prompt and explicit secret", async () => {
    const msg = await firstEnvMessage(
      'await env("MY_PLAIN_VALUE", { hint: "from the dashboard", secret: true });\n',
    );
    expect(msg.key).toBe("MY_PLAIN_VALUE");
    expect(msg.prompt).toBe("from the dashboard");
    expect(msg.secret).toBe(true);
  }, 20_000);

  test("bare string second arg becomes the prompt; key-name inference still masks", async () => {
    const msg = await firstEnvMessage(
      'await env("MY_API_KEY", "Paste your API key");\n',
    );
    expect(msg.prompt).toBe("Paste your API key");
    expect(msg.secret).toBe(true); // "key" substring inference preserved
  }, 20_000);

  test("no config keeps today's shape", async () => {
    const msg = await firstEnvMessage('await env("MY_PLAIN_VALUE");\n');
    expect(msg.key).toBe("MY_PLAIN_VALUE");
    expect(msg.secret).toBe(false);
  }, 20_000);
});
