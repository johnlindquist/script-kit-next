import { describe, expect, test } from "bun:test";
import { resolveAcpTargetFromList } from "../acp-mention";

describe("resolveAcpTargetFromList", () => {
  test("prefers the embedded ai target over main acpChat", () => {
    const result = resolveAcpTargetFromList({
      targets: [
        { automationId: "main", windowKind: "Main", semanticSurface: "acpChat" },
        { automationId: "ai", windowKind: "Ai", semanticSurface: "acpChat" },
      ],
    });

    expect(result.target).toEqual({ type: "id", id: "ai" });
    expect(result.selected?.automationId).toBe("ai");
  });

  test("falls back to detached ACP", () => {
    const result = resolveAcpTargetFromList({
      targets: [
        { automationId: "acpDetached:demo", windowKind: "AcpDetached", semanticSurface: "acpChat" },
      ],
    });

    expect(result.target).toEqual({ type: "id", id: "acpDetached:demo" });
  });

  test("fails closed when no ACP target is present", () => {
    const result = resolveAcpTargetFromList({
      targets: [{ automationId: "main", windowKind: "Main", semanticSurface: "scriptList" }],
    });

    expect(result.target).toBeNull();
    expect(result.candidates).toEqual([]);
  });
});
