import { describe, expect, test } from "bun:test";
import { prepareValidatedReceipt } from "./lib/receipt-schema.ts";
import {
  buildBindingsReceipt,
  runBindingsPipeline,
  runCoverageNegativeControls,
} from "./surfaces.ts";

async function passingInventory() {
  const pipeline = await runBindingsPipeline();
  const negatives = runCoverageNegativeControls(pipeline.build.set, null);
  return buildBindingsReceipt(pipeline, negatives);
}

describe("executable PF-009 surface coverage binding inventory", () => {
  test("all 49 mappings produce a valid static receipt without runtime claims", async () => {
    const candidate = await passingInventory();
    const prepared = prepareValidatedReceipt(
      "devtools.coverage.bindings",
      candidate,
    );
    expect(prepared.exitCode).toBe(0);
    expect(prepared.receipt.disposition).toBe("EVALUABLE_PASS");
    expect(prepared.receipt.evidenceClass).toBe("STATIC_INVENTORY");
    expect((prepared.receipt.catalogBinding as Record<string, unknown>).taskId).toBe(
      "PF-009",
    );
    expect((prepared.receipt.bindings as unknown[]).length).toBe(49);
    const summary = prepared.receipt.summary as Record<string, unknown>;
    expect(summary.staticDirectBindingCount).toBeGreaterThan(0);
    expect(summary.freshDirectRuntimeProofCount).toBe(0);
    expect(summary.runtimeProofDisposition).toBe("NOT_EVALUATED");
  });

  test("a static Direct relation cannot be counted as fresh runtime proof", async () => {
    const candidate = await passingInventory();
    candidate.summary.freshDirectRuntimeProofCount = 1;
    const prepared = prepareValidatedReceipt(
      "devtools.coverage.bindings",
      candidate,
    );
    expect(prepared.receipt.disposition).toBe("INVALID_SCHEMA");
    expect(JSON.stringify(prepared.receipt)).toContain(
      "cannot claim fresh direct runtime proof",
    );
  });

  test("root launcher resolves to its exact direct owner without runtime receipts", async () => {
    const pipeline = await runBindingsPipeline();
    const launcher = pipeline.build.set.bindings.find(
      (binding) => binding.contractKind === "ScriptList",
    );

    expect(launcher?.profileId).toBe("main");
    expect(launcher?.relation).toBe("Direct");
    expect(launcher?.evidenceGrade).toBe("Direct");
    expect(launcher?.missingPrimitiveIds).toEqual([]);

    const receipt = await passingInventory();
    expect(receipt.summary.freshDirectRuntimeProofCount).toBe(0);
  });

  test("high-traffic surfaces use exact production owners instead of inherited host coverage", async () => {
    const pipeline = await runBindingsPipeline();
    const expected = [
      ["ClipboardHistoryView", "clipboard-history"],
      ["BrowserHistoryView", "browser-history"],
      ["NotesBrowseView", "notes-browse"],
      ["FileSearchView", "file-search"],
      ["DayPage", "day-page"],
      ["CurrentAppCommandsView", "current-app-commands"],
      ["WebcamView", "webcam"],
    ] as const;

    // Past chats and ⌘K now open under ScriptList rather than a second window.
    expect(pipeline.build.set.bindings.some(b => b.appViewVariant === "AgentChatHistoryView")).toBe(false);
    expect(pipeline.build.set.aliases.find(b => b.alias === "actions-dialog")?.appViewVariant).toBe("ScriptList");
    expect(pipeline.build.set.bindings.find(b => b.appViewVariant === "ActionsDialog")?.supported).toBe(false);
    for (const [variant, owner] of expected) {
      const matches = pipeline.build.set.bindings.filter(
        (binding) => binding.appViewVariant === variant,
      );
      expect(matches.length).toBeGreaterThan(0);
      for (const binding of matches) {
        expect(binding.profileId).toBe(owner);
        expect(binding.relation).toBe("Direct");
        expect(binding.evidenceGrade).toBe("Direct");
        expect(binding.profileStatus).toBe("partial");
      }
    }

    const receipt = await passingInventory();
    expect(receipt.summary.freshDirectRuntimeProofCount).toBe(0);
  });

  test("missing owner validation or failed negative controls cannot pass", async () => {
    const invalidOwners = await passingInventory();
    invalidOwners.profileRegistry.validationErrorCount = 1;
    expect(
      prepareValidatedReceipt("devtools.coverage.bindings", invalidOwners)
        .receipt.disposition,
    ).toBe("INVALID_SCHEMA");

    const failedNegative = await passingInventory();
    failedNegative.negativeControls[0]!.pass = false;
    expect(
      prepareValidatedReceipt("devtools.coverage.bindings", failedNegative)
        .receipt.disposition,
    ).toBe("INVALID_SCHEMA");
  });

  test("mapping and contract census drift cannot masquerade as complete", async () => {
    const candidate = await passingInventory();
    candidate.census.actual.contractMappingCount = 53;
    expect(
      prepareValidatedReceipt("devtools.coverage.bindings", candidate)
        .receipt.disposition,
    ).toBe("INVALID_SCHEMA");
  });
});
