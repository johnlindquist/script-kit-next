import { describe, expect, test } from "bun:test";
import { resolveAgentChatTargetFromList } from "../agent_chat-mention";

describe("resolveAgentChatTargetFromList", () => {
  test("prefers the embedded ai target over main agentChatChat", () => {
    const result = resolveAgentChatTargetFromList({
      targets: [
        { automationId: "main", windowKind: "Main", semanticSurface: "agentChatChat" },
        { automationId: "ai", windowKind: "Ai", semanticSurface: "agentChatChat" },
      ],
    });

    expect(result.target).toEqual({ type: "id", id: "ai" });
    expect(result.selected?.automationId).toBe("ai");
  });

  test("falls back to detached Agent Chat", () => {
    const result = resolveAgentChatTargetFromList({
      targets: [
        { automationId: "agentChatDetached:demo", windowKind: "AgentChatDetached", semanticSurface: "agentChatChat" },
      ],
    });

    expect(result.target).toEqual({ type: "id", id: "agentChatDetached:demo" });
  });

  test("fails closed when no Agent Chat target is present", () => {
    const result = resolveAgentChatTargetFromList({
      targets: [{ automationId: "main", windowKind: "Main", semanticSurface: "scriptList" }],
    });

    expect(result.target).toBeNull();
    expect(result.candidates).toEqual([]);
  });
});
