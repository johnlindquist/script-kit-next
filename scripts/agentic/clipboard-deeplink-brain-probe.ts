#!/usr/bin/env bun

import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { Database } from "bun:sqlite";
import { Driver } from "../devtools/driver";

const binary =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/clipboard-deeplink-brain/script-kit-gpui";

function runText(command: string[], input?: string): Promise<string> {
  return new Promise((resolve, reject) => {
    const proc = Bun.spawn(command, {
      stdin: input === undefined ? "ignore" : "pipe",
      stdout: "pipe",
      stderr: "pipe",
    });
    if (input !== undefined) {
      proc.stdin.write(input);
      proc.stdin.end();
    }
    proc.exited.then(async (code) => {
      const stdout = await new Response(proc.stdout).text();
      const stderr = await new Response(proc.stderr).text();
      if (code !== 0) {
        reject(new Error(`${command.join(" ")} failed (${code}): ${stderr}`));
        return;
      }
      resolve(stdout);
    });
  });
}

async function setClipboard(text: string): Promise<void> {
  await runText(["/usr/bin/pbcopy"], text);
}

async function getClipboard(): Promise<string> {
  try {
    return await runText(["/usr/bin/pbpaste"]);
  } catch {
    return "";
  }
}

function readDayPages(home: string): string {
  const daysDir = join(home, ".scriptkit", "brain", "days");
  if (!existsSync(daysDir)) return "";
  return readdirSync(daysDir)
    .filter((name) => name.endsWith(".md"))
    .sort()
    .map((name) => readFileSync(join(daysDir, name), "utf8"))
    .join("\n---DAY---\n");
}

function fragmentCount(home: string): number {
  const fragmentsDir = join(home, ".scriptkit", "brain", "fragments");
  if (!existsSync(fragmentsDir)) return 0;
  return readdirSync(fragmentsDir).filter((name) => name.endsWith(".md")).length;
}

async function waitFor<T>(
  label: string,
  fn: () => T | undefined,
  timeoutMs = 10_000,
): Promise<T> {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    const value = fn();
    if (value !== undefined) return value;
    await Bun.sleep(100);
  }
  throw new Error(`Timed out waiting for ${label}`);
}

const previousClipboard = await getClipboard();
const driver = await Driver.launch({
  binary,
  sandboxHome: true,
  sessionName: "clipboard-deeplink-brain-probe",
  readyTimeoutMs: 30_000,
  defaultTimeoutMs: 10_000,
});

try {
  const state = await driver.getState({ timeoutMs: 10_000 });
  const sandboxHome = join(driver.sessionDir, "home");
  const copiedUrl = `https://clipboard-deeplink-runtime.example/${Date.now()}`;

  await setClipboard(copiedUrl);

  const row = await waitFor("clipboard db row", () => {
    const dbPath = join(sandboxHome, ".scriptkit", "db", "clipboard-history.sqlite");
    if (!existsSync(dbPath)) return undefined;
    const db = new Database(dbPath, { readonly: true });
    try {
      return db
        .query<{
          id: string;
          brain_kept: number;
          kept_url_day: string | null;
        }, [string]>(
          "SELECT id, brain_kept, kept_url_day FROM history WHERE content = ?",
        )
        .get(copiedUrl) ?? undefined;
    } finally {
      db.close();
    }
  });

  const expectedUri = `kit://clipboard-history?id=${row.id}`;
  const dayContent = await waitFor("raw-free brain day deeplink", () => {
    const content = readDayPages(sandboxHome);
    if (content.includes(expectedUri)) return content;
    return undefined;
  });

  const receipt = {
    ok: true,
    sessionDir: driver.sessionDir,
    binary,
    stateType: state.type,
    promptType: state.promptType ?? state.state?.promptType ?? null,
    copiedUrlLength: copiedUrl.length,
    clipboardEntryId: row.id,
    brainKept: row.brain_kept === 1,
    keptUrlDaySet: typeof row.kept_url_day === "string" && row.kept_url_day.length > 0,
    expectedUri,
    dayContainsExpectedUri: dayContent.includes(expectedUri),
    dayContainsRawUrl: dayContent.includes(copiedUrl),
    dayContainsGenericLabel: dayContent.includes("[Clipboard entry]("),
    fragmentCount: fragmentCount(sandboxHome),
    driverStats: driver.stats,
  };

  if (!receipt.brainKept) throw new Error("clipboard row was not marked brain_kept");
  if (!receipt.keptUrlDaySet) throw new Error("kept_url_day was not recorded");
  if (!receipt.dayContainsExpectedUri) throw new Error("day page missing clipboard deeplink");
  if (receipt.dayContainsRawUrl) throw new Error("day page leaked raw clipboard URL");
  if (!receipt.dayContainsGenericLabel) throw new Error("day page missing generic label");
  if (receipt.fragmentCount !== 0) throw new Error("clipboard sediment created a fragment");

  console.log(JSON.stringify(receipt, null, 2));
} finally {
  await setClipboard(previousClipboard);
  await driver.close();
}
