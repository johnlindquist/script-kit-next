import { readFileSync } from "node:fs";
import { join } from "node:path";

const root = process.cwd();
const playground = readFileSync(
  join(root, "src/storybook/context_picker_popup_playground/mod.rs"),
  "utf8",
);
const acpPicker = readFileSync(join(root, "src/ai/acp/picker_popup.rs"), "utf8");

for (const token of [
  "acp_context_picker_item_to_inline_picker_row",
  "InlinePickerRow",
  "inline_picker_normalize_selected_index",
  "render_soft_compact_picker_row",
  "SOFT_COMPACT_PICKER_ROW_HEIGHT",
]) {
  if (!playground.includes(token) && !acpPicker.includes(token)) {
    throw new Error(`missing context picker parity token ${token}`);
  }
}

if (playground.includes("context_picker_row::render")) {
  throw new Error("Storybook context picker must not call context_picker_row::render");
}

console.log(JSON.stringify({ ok: true, scenario: "storybook_context_picker_parity" }));
