#!/usr/bin/env bun
import { spawnSync } from "child_process";
import { basename, join } from "path";
import { allImps, impsRoot, routePrompt } from "../lib/project-config.ts";

const args = process.argv.slice(2);
const which = args.includes("--which");
const list = args.includes("--list") || args.includes("-l");
const prompt = args.filter((arg) => arg !== "--which" && arg !== "--list" && arg !== "-l").join(" ");

if (list) {
  for (const imp of allImps()) {
    console.log(`${imp.name}\t${imp.phase}\t${imp.permission}\t${imp.summary}`);
  }
  process.exit(0);
}

if (!prompt) {
  console.error("Usage: project-imp [--which] <task prompt>");
  process.exit(1);
}

const routed = routePrompt(prompt);

if (which) {
  console.log(routed.map((imp) => imp.name).join("\n"));
  process.exit(0);
}

const [primary, ...secondary] = routed;
if (secondary.length) {
  console.error(`project-imp: primary ${primary.name}; also consider ${secondary.map((imp) => imp.name).join(", ")}`);
}

const command = join(impsRoot, "imps", primary.name);
const result = spawnSync(command, [prompt], {
  cwd: join(impsRoot, "..", ".."),
  stdio: "inherit",
  env: process.env,
});

if (result.error) {
  console.error(`${basename(command)}: ${result.error.message}`);
  process.exit(1);
}
process.exit(result.status ?? 0);
