import { readFileSync } from "node:fs";
import { join } from "node:path";

const root = process.cwd();
const moduleSource = readFileSync(join(root, "src/storybook/mod.rs"), "utf8");
const browserSource = readFileSync(join(root, "src/storybook/browser.rs"), "utf8");
const storybookDocs = readFileSync(join(root, "removed-docs"), "utf8");

for (const token of [
  "StorybookWindowRegistry",
  "register_primary",
  "register_child",
  "unregister_primary",
  "unregister_child",
  "should_quit_after_close",
]) {
  if (!moduleSource.includes(token) && !browserSource.includes(token)) {
    throw new Error(`missing Storybook lifecycle token ${token}`);
  }
}

for (const token of ["Theme Reveal Synchronization", "Storybook Window Lifecycle"]) {
  if (!storybookDocs.includes(token)) {
    throw new Error(`missing Storybook removed-docs section ${token}`);
  }
}

console.log(JSON.stringify({ ok: true, scenario: "storybook_lifecycle_theme" }));
