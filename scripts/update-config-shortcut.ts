#!/usr/bin/env bun
/**
 * Compatibility wrapper for shortcut recorder writes.
 *
 * Usage:
 *   bun scripts/update-config-shortcut.ts <command_id> <key> <cmd> <ctrl> <alt> <shift>
 */

import { spawnSync } from "node:child_process";

const cliPath = new URL("./config-cli.ts", import.meta.url).pathname;
const result = spawnSync(
  process.execPath,
  [cliPath, "set-command-shortcut", ...process.argv.slice(2)],
  { stdio: "inherit" },
);

process.exit(result.status ?? 1);
