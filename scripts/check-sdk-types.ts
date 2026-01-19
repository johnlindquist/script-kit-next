#!/usr/bin/env bun
/**
 * SDK Type Check Script
 * Runs TypeScript type checking on kit-sdk.ts to prevent type regressions.
 *
 * Usage:
 *   bun scripts/check-sdk-types.ts          # Standard check
 *   bun scripts/check-sdk-types.ts --strict # Enable strict mode (more errors)
 *
 * Exit codes: 0 = pass, 1 = type errors found
 */

import { spawnSync } from "node:child_process";

const SDK_PATH = "scripts/kit-sdk.ts";

// Check for --strict flag
const strictMode = process.argv.includes("--strict");

// TypeScript compiler options for the SDK
// These match the options needed for the SDK to type-check correctly:
// - ES2022 for modern JS features (Promise, Map, Set, etc.)
// - Node types for process, Buffer, etc.
// - Bundler resolution for module imports
const TSC_OPTIONS = [
  "--noEmit",
  "--lib", "ES2022",
  "--target", "ES2022",
  "--types", "node",
  "--moduleResolution", "bundler",
  "--module", "ES2022",
  "--skipLibCheck", // Skip checking node_modules types
  ...(strictMode ? ["--strict"] : []),
];

function main() {
  const modeLabel = strictMode ? " (strict mode)" : "";
  console.log(`Checking SDK types: ${SDK_PATH}${modeLabel}`);
  console.log("─".repeat(50));

  // Run tsc with --noEmit to check types without producing output
  const result = spawnSync(
    "./node_modules/.bin/tsc",
    [...TSC_OPTIONS, SDK_PATH],
    { encoding: "utf-8", stdio: ["inherit", "pipe", "pipe"] }
  );

  const stdout = result.stdout?.trim();
  const stderr = result.stderr?.trim();

  if (result.status === 0) {
    console.log("✓ SDK types check passed (0 errors)");
    process.exit(0);
  }

  console.error("✗ SDK types check FAILED\n");

  if (stdout) {
    // Count errors
    const errorLines = stdout.split("\n").filter((line) => line.includes("error TS"));
    console.error(`Found ${errorLines.length} type error(s):\n`);
    console.error(stdout);
  }

  if (stderr) {
    console.error(stderr);
  }

  process.exit(1);
}

main();
