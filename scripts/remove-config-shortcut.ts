#!/usr/bin/env bun
/**
 * Compatibility wrapper for shortcut recorder removals.
 *
 * Usage:
 *   bun scripts/remove-config-shortcut.ts <command_id>
 */

import { spawnSync } from "node:child_process";

const cliPath = new URL("./config-cli.ts", import.meta.url).pathname;
const result = spawnSync(
  process.execPath,
  [cliPath, "remove-command-shortcut", ...process.argv.slice(2)],
  { stdio: "inherit" },
);

process.exit(result.status ?? 1);
