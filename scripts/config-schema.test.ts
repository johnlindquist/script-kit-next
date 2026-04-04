import { describe, expect, it } from "bun:test";
import {
  COMMAND_ID_CATEGORIES,
  isValidCommandId,
  parseCommandConfigPath,
  parseCommandId,
  validateCommandConfigFieldValue,
  validateCommandConfigValue,
  validateCommandIdList,
  validateCommandsConfig,
  toDeeplink,
  fromDeeplink,
} from "./config-schema";

// =============================================================================
// parseCommandId
// =============================================================================

describe("parseCommandId", () => {
  it("parses canonical ids into category and identifier", () => {
    expect(parseCommandId("builtin/clipboard-history")).toEqual({
      category: "builtin",
      identifier: "clipboard-history",
    });
    expect(parseCommandId("scriptlet/abc-123")).toEqual({
      category: "scriptlet",
      identifier: "abc-123",
    });
  });

  it("rejects empty identifiers", () => {
    expect(parseCommandId("builtin/")).toBeNull();
    expect(parseCommandId("app/")).toBeNull();
    expect(parseCommandId("script/")).toBeNull();
    expect(parseCommandId("scriptlet/")).toBeNull();
  });

  it("exports all supported categories", () => {
    expect(COMMAND_ID_CATEGORIES).toEqual([
      "builtin",
      "app",
      "script",
      "scriptlet",
    ]);
  });
});

// =============================================================================
// validateCommandIdList
// =============================================================================

describe("validateCommandIdList", () => {
  it("accepts canonical slash-style ids", () => {
    expect(
      validateCommandIdList(
        ["builtin/quit-script-kit", "script/my-script"],
        "suggested.excludedCommands",
      ),
    ).toEqual([]);
  });

  it("rejects dash-style and empty ids", () => {
    const errors = validateCommandIdList(
      ["builtin-quit-script-kit", "builtin/"],
      "suggested.excludedCommands",
    );
    expect(errors).toHaveLength(2);
    expect(errors[0].code).toBe("invalidCommandId");
    expect(errors[0].path).toBe("suggested.excludedCommands[0]");
    expect(errors[1].path).toBe("suggested.excludedCommands[1]");
  });

  it("rejects non-array input", () => {
    const errors = validateCommandIdList("not-array", "path");
    expect(errors).toHaveLength(1);
    expect(errors[0].code).toBe("invalidType");
  });
});

// =============================================================================
// isValidCommandId
// =============================================================================

describe("isValidCommandId", () => {
  it("accepts canonical slash-style IDs", () => {
    expect(isValidCommandId("builtin/clipboard-history")).toBe(true);
    expect(isValidCommandId("app/com.apple.Safari")).toBe(true);
    expect(isValidCommandId("script/my-script")).toBe(true);
    expect(isValidCommandId("scriptlet/abc-123")).toBe(true);
  });

  it("rejects dash-style builtin IDs", () => {
    expect(isValidCommandId("builtin-clipboard-history")).toBe(false);
  });

  it("rejects bare identifiers", () => {
    expect(isValidCommandId("clipboard-history")).toBe(false);
  });

  it("rejects unknown categories", () => {
    expect(isValidCommandId("unknown/thing")).toBe(false);
    expect(isValidCommandId("agent/foo")).toBe(false);
  });
});

// =============================================================================
// validateCommandConfigValue
// =============================================================================

describe("validateCommandConfigValue", () => {
  it("accepts valid hidden-only config", () => {
    const errors = validateCommandConfigValue({ hidden: true }, "commands.builtin/foo");
    expect(errors).toEqual([]);
  });

  it("accepts valid shortcut config", () => {
    const errors = validateCommandConfigValue(
      { shortcut: { modifiers: ["meta", "shift"], key: "KeyV" } },
      "commands.builtin/foo",
    );
    expect(errors).toEqual([]);
  });

  it("accepts valid confirmationRequired config", () => {
    const errors = validateCommandConfigValue(
      { confirmationRequired: true },
      "commands.builtin/foo",
    );
    expect(errors).toEqual([]);
  });

  it("rejects non-object values", () => {
    const errors = validateCommandConfigValue("not-an-object", "commands.x");
    expect(errors.length).toBe(1);
    expect(errors[0].code).toBe("invalidType");
  });

  it("rejects unknown keys", () => {
    const errors = validateCommandConfigValue(
      { hidden: true, bogus: 42 },
      "commands.builtin/foo",
    );
    expect(errors.length).toBe(1);
    expect(errors[0].code).toBe("unknownCommandConfigKey");
  });

  it("rejects non-boolean hidden", () => {
    const errors = validateCommandConfigValue({ hidden: "yes" }, "commands.builtin/foo");
    expect(errors.length).toBe(1);
    expect(errors[0].code).toBe("invalidType");
  });

  it("rejects shortcut with missing modifiers", () => {
    const errors = validateCommandConfigValue(
      { shortcut: { key: "KeyV" } },
      "commands.builtin/foo",
    );
    expect(errors.some((e) => e.path.includes("modifiers"))).toBe(true);
  });

  it("rejects shortcut with invalid key code", () => {
    const errors = validateCommandConfigValue(
      { shortcut: { modifiers: ["meta"], key: "Nope" } },
      "commands.builtin/foo",
    );
    expect(errors.some((e) => e.code === "invalidKeyCode")).toBe(true);
  });
});

// =============================================================================
// validateCommandsConfig
// =============================================================================

describe("validateCommandsConfig", () => {
  it("accepts valid commands block with slash-style IDs", () => {
    const result = validateCommandsConfig({
      "builtin/clipboard-history": { hidden: true },
      "app/com.apple.Safari": {
        shortcut: { modifiers: ["meta", "shift"], key: "KeyS" },
      },
      "script/my-script": { confirmationRequired: true },
    });
    expect(result.valid).toBe(true);
    expect(result.errors).toEqual([]);
    expect(result.normalizedValue).toBeTruthy();
  });

  it("rejects dash-style builtin IDs with invalidCommandId code", () => {
    const result = validateCommandsConfig({
      "builtin-clipboard-history": { hidden: true },
    });
    expect(result.valid).toBe(false);
    expect(result.errors.length).toBe(1);
    expect(result.errors[0].code).toBe("invalidCommandId");
    expect(result.errors[0].path).toBe("commands.builtin-clipboard-history");
  });

  it("rejects non-object input", () => {
    const result = validateCommandsConfig("not-an-object");
    expect(result.valid).toBe(false);
    expect(result.errors[0].code).toBe("invalidType");
  });

  it("collects multiple errors", () => {
    const result = validateCommandsConfig({
      "bad-id": { hidden: true },
      "builtin/ok": { hidden: "yes" },
    });
    expect(result.valid).toBe(false);
    expect(result.errors.length).toBe(2);
  });

  it("accepts empty commands block", () => {
    const result = validateCommandsConfig({});
    expect(result.valid).toBe(true);
  });
});

// =============================================================================
// Deeplink roundtrip
// =============================================================================

describe("deeplink roundtrip", () => {
  it("converts command ID to deeplink and back", () => {
    const id = "builtin/clipboard-history";
    const deeplink = toDeeplink(id);
    expect(deeplink).toBe("scriptkit://commands/builtin/clipboard-history");
    const parsed = fromDeeplink(deeplink);
    expect(parsed).toBe(id);
  });

  it("rejects invalid deeplink prefix", () => {
    expect(fromDeeplink("kit://commands/builtin/foo")).toBeNull();
  });

  it("rejects deeplink with unknown category", () => {
    expect(fromDeeplink("scriptkit://commands/unknown/foo")).toBeNull();
  });

  it("rejects deeplink with empty identifier", () => {
    expect(fromDeeplink("scriptkit://commands/builtin/")).toBeNull();
  });
});

// =============================================================================
// Nested command config paths
// =============================================================================

describe("parseCommandConfigPath", () => {
  it("parses a whole command entry path", () => {
    expect(parseCommandConfigPath("commands.builtin/clipboard-history")).toEqual({
      commandId: "builtin/clipboard-history",
    });
  });

  it("parses nested command field paths", () => {
    expect(parseCommandConfigPath("commands.builtin/clipboard-history.hidden")).toEqual({
      commandId: "builtin/clipboard-history",
      fieldPath: "hidden",
    });
    expect(parseCommandConfigPath("commands.script/my-script.shortcut.key")).toEqual({
      commandId: "script/my-script",
      fieldPath: "shortcut.key",
    });
  });

  it("rejects invalid command ids in nested paths", () => {
    expect(parseCommandConfigPath("commands.builtin-clipboard-history.hidden")).toBeNull();
  });
});

describe("validateCommandConfigFieldValue", () => {
  it("accepts nested command field updates", () => {
    expect(
      validateCommandConfigFieldValue(
        "hidden",
        true,
        "commands.builtin/clipboard-history.hidden",
      ),
    ).toEqual([]);
    expect(
      validateCommandConfigFieldValue(
        "shortcut.key",
        "KeyV",
        "commands.builtin/clipboard-history.shortcut.key",
      ),
    ).toEqual([]);
  });

  it("rejects invalid nested command field values", () => {
    const errors = validateCommandConfigFieldValue(
      "shortcut.key",
      "Nope",
      "commands.builtin/clipboard-history.shortcut.key",
    );
    expect(errors).toHaveLength(1);
    expect(errors[0].code).toBe("invalidKeyCode");
  });
});
