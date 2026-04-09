#!/usr/bin/env bun

import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname } from "node:path";

type ExampleMirror = {
  source: string;
  target: string;
};

const MIRRORS: ExampleMirror[] = [
  {
    source: "kit-init/extensions/acp-chat/main.md",
    target: "kit-init/extensions/examples/acp-chat.md",
  },
  {
    source: "kit-init/extensions/custom-actions/main.md",
    target: "kit-init/extensions/examples/custom-actions.md",
  },
  {
    source: "kit-init/extensions/custom-actions/main.actions.md",
    target: "kit-init/extensions/examples/custom-actions.actions.md",
  },
  {
    source: "kit-init/extensions/notes/main.md",
    target: "kit-init/extensions/examples/notes.md",
  },
];

function isEnoent(error: unknown): boolean {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    (error as { code?: string }).code === "ENOENT"
  );
}

async function readUtf8IfExists(path: string): Promise<string | null> {
  try {
    return await readFile(path, "utf8");
  } catch (error) {
    if (isEnoent(error)) return null;
    throw error;
  }
}

async function syncMirror(
  mirror: ExampleMirror,
  checkOnly: boolean
): Promise<boolean> {
  const nextText = await readFile(mirror.source, "utf8");
  const prevText = await readUtf8IfExists(mirror.target);
  const changed = prevText !== nextText;

  if (changed && !checkOnly) {
    await mkdir(dirname(mirror.target), { recursive: true });
    await writeFile(mirror.target, nextText, "utf8");
  }

  console.log(
    JSON.stringify({
      type: "extension_example_sync",
      ok: !changed || !checkOnly,
      mode: checkOnly ? "check" : "write",
      source: mirror.source,
      target: mirror.target,
      changed,
      bytes: nextText.length,
    })
  );

  return changed;
}

async function main(): Promise<void> {
  const checkOnly = process.argv.includes("--check");
  let changedCount = 0;

  for (const mirror of MIRRORS) {
    if (await syncMirror(mirror, checkOnly)) changedCount += 1;
  }

  if (checkOnly && changedCount > 0) {
    process.exitCode = 1;
  }
}

await main();
