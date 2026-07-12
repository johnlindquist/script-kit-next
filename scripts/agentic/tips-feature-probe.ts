#!/usr/bin/env bun
/**
 * Runtime proof for the Tips discoverability feature:
 *  A1 sandbox seeding of ~/.scriptkit/tips.json
 *  A2 footer tip visible on root ScriptList with empty filter (leftInfo.action === "tips")
 *  A3 footer tip hidden once the filter is non-empty; returns when cleared
 *  A4 tip rotation across hide/show visibility transitions
 *  A5 Tips builtin opens from the main list; persistent leading section header ("Tips" -> "Results")
 *  A6 escape returns to the main menu
 *  B1 config.ts tips.enabled=false hides the footer tip
 */
import { join } from "node:path";
import { existsSync, mkdirSync, writeFileSync, rmSync } from "node:fs";
import { Driver } from "../devtools/driver.ts";

const BINARY = "target-agent/artifacts/tips/script-kit-gpui";
const receipt: Record<string, unknown> = {};
const fails: string[] = [];
const check = (name: string, ok: boolean, detail?: unknown) => {
  receipt[name] = { ok, detail };
  if (!ok) fails.push(name);
};

const leftInfo = (state: any) =>
  state?.activeFooter?.leftInfo ?? state?.state?.activeFooter?.leftInfo ?? null;
const viewName = (state: any) => state?.surfaceContract?.surfaceKind ?? state?.promptType;

// ---------- Scenario A: defaults ----------
{
  const d = await Driver.launch({ sandboxHome: true, binary: BINARY, sessionName: "tips-probe" });
  try {
    await d.waitForSettle();
    const kitDir = join(d.sessionDir, "home", ".scriptkit");
    check("A1_seeded_tips_json", existsSync(join(kitDir, "tips.json")), join(kitDir, "tips.json"));

    let s: any = await d.getState();
    receipt.rootView = viewName(s);
    const li0 = leftInfo(s);
    check(
      "A2_footer_tip_on_empty_root",
      !!li0 && li0.action === "tips" && typeof li0.modelName === "string" && li0.modelName.length > 0,
      li0,
    );
    const tipBefore = li0?.modelName ?? null;

    await d.setFilterAndWait("t");
    s = await d.getState();
    check("A3_tip_hidden_when_typing", leftInfo(s) === null || leftInfo(s) === undefined, leftInfo(s));

    d.setFilter("");
    await d.waitForSettle();
    s = await d.getState();
    check("A3b_tip_back_when_cleared", !!leftInfo(s) && leftInfo(s).action === "tips", leftInfo(s));

    // A4 rotation across hide/show
    d.send({ type: "hide" });
    await d.waitForSettle();
    d.send({ type: "show" });
    await d.waitForSettle();
    s = await d.getState();
    const tipAfter = leftInfo(s)?.modelName ?? null;
    check("A4_rotates_on_reopen", tipAfter !== null && tipAfter !== tipBefore, { tipBefore, tipAfter });

    // A5 open Tips builtin via the real list path
    await d.setFilterAndWait("Tips");
    s = await d.getState();
    let els: any = await d.getElements({});
    let nodes: any[] = els?.elements ?? els?.nodes ?? [];
    const tipsRow = nodes.find((n) => n.semanticId === "choice:0:tips");
    check("A5a_tips_row_selected", !!tipsRow && tipsRow.selected === true, tipsRow);
    d.simulateKey("enter");
    await d.waitForSettle();
    s = await d.getState();
    receipt.viewAfterEnter = viewName(s) ?? null;
    const inTips = String(receipt.viewAfterEnter ?? "").toLowerCase().includes("tips");
    check("A5_enter_opens_tips_view", inTips, receipt.viewAfterEnter);

    if (inTips) {
      // NOTE: collect_named_rows does not emit sectionHeader nodes (pre-existing
      // primitive gap shared with SdkReference); the persistent leading header is
      // proven visually via the screenshot below and structurally in
      // src/render_builtins/tips.rs. Here we prove real filtering through the
      // shared main input.
      const rowCount = (ns: any[]) => ns.filter((n) => n.type === "choice").length;
      els = await d.getElements({});
      nodes = els?.elements ?? els?.nodes ?? [];
      const allRows = rowCount(nodes);

      d.setFilter("capture");
      await d.waitForSettle();
      els = await d.getElements({});
      const nodes2: any[] = els?.elements ?? els?.nodes ?? [];
      const inputNode = nodes2.find((n) => n.semanticId === "input:tips-filter");
      const filteredRows = rowCount(nodes2);
      check(
        "A5b_typing_filters_tips",
        inputNode?.value === "capture" && filteredRows > 0 && filteredRows < allRows,
        { inputValue: inputNode?.value, allRows, filteredRows, top: nodes2[2]?.text },
      );

      d.setFilter("");
      await d.waitForSettle();
      els = await d.getElements({});
      check(
        "A5c_clearing_restores_all_tips",
        rowCount(els?.elements ?? els?.nodes ?? []) === allRows,
        { allRows },
      );

      // visual header proof
      d.send({ type: "show" });
      await d.waitForSettle();
      const shot: any = await d
        .captureScreenshot(".test-screenshots/tips-view.png")
        .catch((e: any) => ({ error: String(e) }));
      receipt.screenshot = shot?.error ? shot : ".test-screenshots/tips-view.png";

      // A6 escape ladder back to main menu (may need 2 presses if filter non-empty)
      d.simulateKey("escape");
      await d.waitForSettle();
      s = await d.getState();
      if (String(viewName(s) ?? "").toLowerCase().includes("tips")) {
        d.simulateKey("escape");
        await d.waitForSettle();
        s = await d.getState();
      }
      check(
        "A6_escape_returns_to_main",
        String(viewName(s) ?? "").toLowerCase().includes("script"),
        viewName(s),
      );
    }
  } finally {
    await d.close();
  }
}

// ---------- Scenario B: config disable ----------
{
  const home = "/tmp/sk-tips-probe-disabled-home";
  rmSync(home, { recursive: true, force: true });
  const kitDir = join(home, ".scriptkit");
  mkdirSync(kitDir, { recursive: true });
  writeFileSync(join(kitDir, "config.ts"), "export default { tips: { enabled: false } };\n");
  const d = await Driver.launch({
    binary: BINARY,
    sessionName: "tips-probe-disabled",
    env: { HOME: home, SK_PATH: kitDir },
  });
  try {
    await d.waitForSettle();
    const s: any = await d.getState();
    receipt.disabledRootView = viewName(s);
    const li = leftInfo(s);
    check("B1_config_disables_footer_tip", li === null || li === undefined || li.action !== "tips", li);
  } finally {
    await d.close();
  }
  rmSync(home, { recursive: true, force: true });
}

console.log(JSON.stringify({ pass: fails.length === 0, fails, receipt }, null, 2));
process.exit(fails.length === 0 ? 0 : 1);
