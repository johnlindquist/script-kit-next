// Probe: visible main-list rows + enter action for spine fragments.
import { Driver } from "../devtools/driver.ts";

const FRAGMENTS = [
  "@files",
  "@file:",
  "@file:read",
  "@screenshot",
  "@clipboard",
  "@clipboard:",
  "@history",
  "@history:",
  "@notes:",
  "@project:",
  "@scripts:",
];

const driver = await Driver.launch({
  sandboxHome: true,
  binary: "target/debug/script-kit-gpui",
});

const receipt: Record<string, unknown> = {};
try {
  for (const fragment of FRAGMENTS) {
    await driver.setFilterAndWait(fragment);
    await new Promise((r) => setTimeout(r, 600));
    const state: any = await driver.getState();
    const pf = state.mainWindowPreflight ?? {};
    receipt[fragment] = {
      visibleChoiceCount: state.visibleChoiceCount,
      selectedValue: state.selectedValue,
      rows: (pf.visibleResults ?? []).map((r: any) => ({
        key: r.stableKey,
        role: r.role,
        type: r.typeLabel,
        desc: r.description,
      })),
      enterAction: pf.enterAction
        ? {
            kind: pf.enterAction.kind,
            label: pf.enterAction.label,
            subject: pf.enterAction.subject,
          }
        : null,
      rowFingerprint: pf.visibleRowFingerprint,
    };
    await driver.setFilterAndWait("");
  }
} finally {
  await driver.close();
}
console.log(JSON.stringify(receipt, null, 2));
