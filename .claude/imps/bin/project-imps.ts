#!/usr/bin/env bun
import { existsSync, readFileSync } from "fs";
import { join } from "path";
import { allImps, impsRoot } from "../lib/project-config.ts";

const [cmd = "list", name] = process.argv.slice(2);

if (cmd === "list") {
  for (const imp of allImps()) {
    const lessonPath = join(impsRoot, "lessons", "local", `${imp.name}.lessons.md`);
    const lessons = existsSync(lessonPath) ? "lessons" : "clean";
    console.log(`${imp.name}\t${imp.phase}\t${imp.permission}\t${lessons}\t${imp.summary}`);
  }
  process.exit(0);
}

if (cmd === "lessons") {
  if (!name) {
    console.error("Usage: project-imps lessons <imp-name>");
    process.exit(1);
  }
  const lessonPath = join(impsRoot, "lessons", "local", `${name}.lessons.md`);
  if (!existsSync(lessonPath)) {
    console.log(`${name}: no local lessons`);
    process.exit(0);
  }
  console.log(readFileSync(lessonPath, "utf8"));
  process.exit(0);
}

if (cmd === "paths") {
  for (const imp of allImps()) {
    console.log(`\n${imp.name}`);
    for (const glob of imp.ownerGlobs) console.log(`  ${glob}`);
  }
  process.exit(0);
}

console.error("Usage: project-imps list | lessons <imp-name> | paths");
process.exit(1);
