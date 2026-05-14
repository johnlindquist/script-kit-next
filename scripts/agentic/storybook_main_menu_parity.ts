import { readFileSync } from "node:fs";
import { join } from "node:path";

const root = process.cwd();
const mainMenu = readFileSync(join(root, "src/storybook/main_menu_variations/mod.rs"), "utf8");

const required = [
  "populated-results",
  "empty-results",
  "selected-row",
  "bottom-of-list-footer-safe-reveal",
  "frontmost-app-paste",
  "acp-ready-footer",
  "acp-not-ready-footer",
];

for (const id of required) {
  if (!mainMenu.includes(id)) throw new Error(`missing main-menu variant ${id}`);
}

for (const token of [
  "StorybookRepresentation::LiveSurface",
  "ProductionMainMenuFixture",
  "render_script_list::render_main_menu_from_inputs",
  "StorybookFooterSnapshot",
  "activeFooter",
  "execute_script_by_path",
  "SCRIPT_READY receipt missing",
]) {
  if (!mainMenu.includes(token)) throw new Error(`missing parity token ${token}`);
}

if (mainMenu.includes("MockDesignOnly")) {
  throw new Error("main-menu primary catalog must not use MockDesignOnly");
}

console.log(JSON.stringify({ ok: true, scenario: "storybook_main_menu_parity", required }));
