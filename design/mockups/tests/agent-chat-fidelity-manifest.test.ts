import { describe, expect, test } from "bun:test";

const screenDir = new URL("../screens/agent-chat/", import.meta.url);

describe("Agent Chat fidelity manifest", () => {
  test("maps every marked HTML element exactly once", async () => {
    const [html, manifest] = await Promise.all([
      Bun.file(new URL("index.html", screenDir)).text(),
      Bun.file(new URL("fidelity-manifest.json", screenDir)).json(),
    ]);

    const screenMatch = html.match(/data-fidelity-screen="([^"]+)"/);
    const htmlIds = [...html.matchAll(/data-fidelity-id="([^"]+)"/g)].map(
      (match) => match[1],
    );
    const fidelityIds = manifest.elements.map(
      (mapping: { fidelityId: string }) => mapping.fidelityId,
    );
    const gpuiIds = manifest.elements.map(
      (mapping: { gpuiId: string }) => mapping.gpuiId,
    );

    expect(screenMatch?.[1]).toBe(manifest.screen.id);
    expect(manifest.screen).toEqual({
      id: "agent-chat",
      target: { automationId: "main", targetKind: "Main" },
      viewport: { width: 750, height: 480 },
    });
    expect(manifest.tolerance).toBe(0.5);
    expect(manifest.requirePaintMeasurement).toBe(true);
    expect(manifest.requireVisibleMeasurement).toBe(true);
    expect(manifest.imageDiff).toEqual({
      required: true,
      maxChangedPixelRatio: 0,
      requireSameSize: true,
      requireInputHashes: true,
      requireRedOsEvidence: true,
    });
    expect(manifest.elements).toHaveLength(13);
    expect(new Set(htmlIds).size).toBe(htmlIds.length);
    expect(new Set(fidelityIds).size).toBe(fidelityIds.length);
    expect(new Set(gpuiIds).size).toBe(gpuiIds.length);
    expect([...htmlIds].sort()).toEqual([...fidelityIds].sort());
  });
});
