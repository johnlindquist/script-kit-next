import "@scriptkit/sdk";

export const metadata = {
  name: "SDK UI Automation Probe",
  description: "Verifies getState/getElements/waitFor/batch",
};

await arg("Pick a fruit", ["apple", "banana", "pear"]);

const before = await getState();
if (!before.windowVisible) {
  throw new Error("expected Script Kit window to be visible");
}

await setInput("app");

const wait = await waitFor("choicesRendered", { timeout: 2000 });
if (!wait.success) {
  throw new Error(`waitFor timeout: ${wait.error?.message ?? "unknown"}`);
}

const elements = await getElements(10);
const hasInput = elements.elements.some(
  (el) => el.semanticId === "input:filter"
);
const hasChoice = elements.elements.some((el) => el.type === "choice");
if (!hasInput || !hasChoice) {
  throw new Error(`unexpected element snapshot: ${JSON.stringify(elements)}`);
}

const tx = await batch(
  [
    { type: "setInput", text: "apple" },
    { type: "waitFor", condition: "choicesRendered", timeout: 2000 },
  ],
  { timeout: 2500 }
);
if (!tx.success) {
  throw new Error(`batch failed: ${JSON.stringify(tx)}`);
}

console.log(
  JSON.stringify({
    ok: true,
    promptType: before.promptType,
    focusedSemanticId: elements.focusedSemanticId,
    totalCount: elements.totalCount,
    truncated: elements.truncated,
    warnings: elements.warnings,
  })
);

exit(0);
