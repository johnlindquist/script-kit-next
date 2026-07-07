import { describe, expect, it } from "bun:test";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const CLI_PATH = new URL("./config-cli.ts", import.meta.url).pathname;
const TEMPLATE_PATH = new URL(
  "../kit-init/config-template.ts",
  import.meta.url,
).pathname;

async function runValidateChange(
  change: unknown,
  env: Record<string, string> = {},
) {
  const proc = Bun.spawn(
    ["bun", CLI_PATH, "validate-change", JSON.stringify(change)],
    {
      stdout: "pipe",
      stderr: "pipe",
      env: { ...process.env, ...env },
    },
  );

  const stdoutText = await new Response(proc.stdout).text();
  const stderrText = await new Response(proc.stderr).text();
  const exitCode = await proc.exited;

  return {
    exitCode,
    stdout: JSON.parse(stdoutText),
    stderrText: stderrText.trim(),
  };
}

async function runCli(args: string[], env: Record<string, string> = {}) {
  const proc = Bun.spawn(["bun", CLI_PATH, ...args], {
    stdout: "pipe",
    stderr: "pipe",
    env: { ...process.env, ...env },
  });
  const stdoutText = await new Response(proc.stdout).text();
  const stderrText = await new Response(proc.stderr).text();
  const exitCode = await proc.exited;
  return {
    exitCode,
    stdout: stdoutText.trim() ? JSON.parse(stdoutText) : null,
    stderrText: stderrText.trim(),
  };
}

describe("config-cli validate-change", () => {
  it("accepts nested command field updates", async () => {
    const result = await runValidateChange(
      {
        key: "commands.builtin/clipboard-history.hidden",
        value: true,
      },
      { SCRIPT_KIT_CONFIG_DEBUG: "1" },
    );
    expect(result.exitCode).toBe(0);
    expect(result.stdout.success).toBe(true);
    expect(result.stdout.data).toEqual({
      valid: true,
      normalizedValue: true,
      errors: [],
      warnings: [],
    });
    expect(result.stderrText).toContain('"event":"validate_change_command_path"');
    expect(result.stderrText).toContain(
      '"commandId":"builtin/clipboard-history"',
    );
  });

  it("rejects invalid nested shortcut key values", async () => {
    const result = await runValidateChange({
      key: "commands.builtin/clipboard-history.shortcut.key",
      value: "Nope",
    });
    expect(result.exitCode).toBe(1);
    expect(result.stdout.success).toBe(false);
    expect(result.stdout.valid).toBe(false);
    expect(result.stdout.errors[0].code).toBe("invalidKeyCode");
  });

  it("rejects dash-style ids inside suggested.excludedCommands", async () => {
    const result = await runValidateChange({
      key: "suggested.excludedCommands",
      value: ["builtin-quit-script-kit"],
    });
    expect(result.exitCode).toBe(1);
    expect(result.stdout.success).toBe(false);
    expect(result.stdout.errors[0].code).toBe("invalidCommandId");
    expect(result.stdout.errors[0].path).toBe(
      "suggested.excludedCommands[0]",
    );
  });

  it("rejects empty command identifiers", async () => {
    const result = await runValidateChange({
      key: "commands.builtin/",
      value: { hidden: true },
    });
    expect(result.exitCode).toBe(1);
    expect(result.stdout.success).toBe(false);
    expect(result.stdout.errors[0].code).toBe("invalidCommandId");
  });

  it("accepts valid suggested.excludedCommands", async () => {
    const result = await runValidateChange({
      key: "suggested.excludedCommands",
      value: ["builtin/quit-script-kit"],
    });
    expect(result.exitCode).toBe(0);
    expect(result.stdout.success).toBe(true);
  });
});

// =============================================================================
// Command ID contract (with debug logging verification)
// =============================================================================

describe("config-cli validate-change command id contract", () => {
  it("accepts canonical nested command paths", async () => {
    const result = await runValidateChange(
      { key: "commands.builtin/clipboard-history.hidden", value: true },
      { SCRIPT_KIT_CONFIG_DEBUG: "1" },
    );
    expect(result.exitCode).toBe(0);
    expect(result.stdout.data.valid).toBe(true);
    expect(result.stderrText).toContain('"event":"validate_change_command_path"');
    expect(result.stderrText).toContain('"commandId":"builtin/clipboard-history"');
  });

  it("rejects dash-style nested command ids with invalidCommandId", async () => {
    const result = await runValidateChange(
      { key: "commands.builtin-clipboard-history.hidden", value: true },
      { SCRIPT_KIT_CONFIG_DEBUG: "1" },
    );
    expect(result.exitCode).toBe(1);
    expect(result.stdout.valid).toBe(false);
    expect(result.stdout.errors).toEqual([
      {
        path: "commands.builtin-clipboard-history.hidden",
        code: "invalidCommandId",
        message: "Invalid command id: builtin-clipboard-history",
      },
    ]);
    expect(result.stderrText).toContain('"event":"validate_change_invalid_command_id"');
  });

  it("rejects empty commands keys with invalidCommandPath", async () => {
    const result = await runValidateChange(
      { key: "commands.", value: {} },
      { SCRIPT_KIT_CONFIG_DEBUG: "1" },
    );
    expect(result.exitCode).toBe(1);
    expect(result.stdout.valid).toBe(false);
    expect(result.stdout.errors[0].code).toBe("invalidCommandPath");
    expect(result.stderrText).toContain('"event":"validate_change_invalid_command_path"');
  });

  it("rejects legacy suggested.excludedCommands ids", async () => {
    const result = await runValidateChange(
      { key: "suggested.excludedCommands", value: ["builtin-quit-script-kit"] },
      { SCRIPT_KIT_CONFIG_DEBUG: "1" },
    );
    expect(result.exitCode).toBe(1);
    expect(result.stdout.valid).toBe(false);
    expect(result.stdout.errors).toEqual([
      {
        path: "suggested.excludedCommands[0]",
        code: "invalidCommandId",
        message: "Invalid command id: builtin-quit-script-kit",
      },
    ]);
    expect(result.stderrText).toContain('"event":"validate_change_command_id_list"');
  });

  it("accepts canonical suggested.excludedCommands ids", async () => {
    const result = await runValidateChange(
      {
        key: "suggested.excludedCommands",
        value: ["builtin/quit-script-kit", "script/my-script"],
      },
      { SCRIPT_KIT_CONFIG_DEBUG: "1" },
    );
    expect(result.exitCode).toBe(0);
    expect(result.stdout.data.valid).toBe(true);
    expect(result.stderrText).toContain('"event":"validate_change_command_id_list"');
  });
});

describe("config-cli command shortcut writers", () => {
  it("sets command shortcuts through config.ts commands", async () => {
    const dir = mkdtempSync(join(tmpdir(), "kit-config-cli-"));
    const configPath = join(dir, "config.ts");
    writeFileSync(
      configPath,
      `import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: { modifiers: ["meta"], key: "Semicolon" }
} satisfies Config;
`,
    );

    try {
      const result = await runCli(
        [
          "set-command-shortcut",
          "script/main:do-in-current-app",
          "1",
          "true",
          "false",
          "false",
          "false",
        ],
        { SCRIPT_KIT_CONFIG_PATH: configPath },
      );

      expect(result.exitCode).toBe(0);
      const content = readFileSync(configPath, "utf8");
      expect(content).toContain('"script/main:do-in-current-app"');
      expect(content).toContain('"key":"Digit1"');
      expect(content).toContain('"modifiers":["meta"]');
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("removes only shortcut fields and preserves command flags", async () => {
    const dir = mkdtempSync(join(tmpdir(), "kit-config-cli-"));
    const configPath = join(dir, "config.ts");
    writeFileSync(
      configPath,
      `import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: { modifiers: ["meta"], key: "Semicolon" },
  commands: {
    "builtin/clipboard-history": { shortcut: {"modifiers":["meta"],"key":"KeyV"}, hidden: true },
  },
} satisfies Config;
`,
    );

    try {
      const result = await runCli(
        ["remove-command-shortcut", "builtin/clipboard-history"],
        { SCRIPT_KIT_CONFIG_PATH: configPath },
      );

      expect(result.exitCode).toBe(0);
      const content = readFileSync(configPath, "utf8");
      expect(content).toContain('"builtin/clipboard-history": { hidden: true }');
      expect(content).not.toContain("shortcut");
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
});

// =============================================================================
// Comment-aware `set` writer (regression: commented template examples)
// =============================================================================
//
// config-template.ts ships many commented example blocks whose key names
// collide with real settable keys (e.g. `// aiHotkeyEnabled: true,`). The
// writer must locate keys against real code only: a key that exists only in
// comments is inserted as a new real property and the comment stays intact.

describe("config-cli set skips commented examples", () => {
  function withTempConfig(content: string): { dir: string; configPath: string } {
    const dir = mkdtempSync(join(tmpdir(), "kit-config-cli-"));
    const configPath = join(dir, "config.ts");
    writeFileSync(configPath, content);
    return { dir, configPath };
  }

  it("inserts a real key when it only exists as a commented template example", async () => {
    const { dir, configPath } = withTempConfig(readFileSync(TEMPLATE_PATH, "utf8"));

    try {
      const result = await runCli(["set", "aiHotkeyEnabled", "false"], {
        SCRIPT_KIT_CONFIG_PATH: configPath,
      });
      expect(result.exitCode).toBe(0);
      expect(result.stdout.success).toBe(true);

      const content = readFileSync(configPath, "utf8");
      // The commented example is untouched...
      expect(content).toContain("// aiHotkeyEnabled: true,");
      // ...and a real (uncommented) property was inserted.
      expect(content).toMatch(/^\s*aiHotkeyEnabled: false/m);

      const get = await runCli(["get", "aiHotkeyEnabled"], {
        SCRIPT_KIT_CONFIG_PATH: configPath,
      });
      expect(get.exitCode).toBe(0);
      expect(get.stdout.data.value).toBe(false);

      const validate = await runCli(["validate"], {
        SCRIPT_KIT_CONFIG_PATH: configPath,
      });
      expect(validate.exitCode).toBe(0);
      expect(validate.stdout.data.valid).toBe(true);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("creates a nested parent without corrupting its commented example", async () => {
    const { dir, configPath } = withTempConfig(readFileSync(TEMPLATE_PATH, "utf8"));

    try {
      const result = await runCli(["set", "dictation.saveHistory", "false"], {
        SCRIPT_KIT_CONFIG_PATH: configPath,
      });
      expect(result.exitCode).toBe(0);
      expect(result.stdout.success).toBe(true);

      const content = readFileSync(configPath, "utf8");
      // The commented dictation example is untouched...
      expect(content).toContain('// dictation: { selectedDeviceId: "usb-mic" },');
      // ...and a real dictation block was inserted with the new value.
      expect(content).toMatch(/^\s*dictation: \{\n\s*saveHistory: false\n\s*\}/m);

      const get = await runCli(["get", "dictation.saveHistory"], {
        SCRIPT_KIT_CONFIG_PATH: configPath,
      });
      expect(get.exitCode).toBe(0);
      expect(get.stdout.data.value).toBe(false);

      const validate = await runCli(["validate"], {
        SCRIPT_KIT_CONFIG_PATH: configPath,
      });
      expect(validate.exitCode).toBe(0);
      expect(validate.stdout.data.valid).toBe(true);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("updates the real key, not a commented duplicate of the same key", async () => {
    const { dir, configPath } = withTempConfig(
      `import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: { modifiers: ["meta"], key: "Semicolon" },
  // editorFontSize: 14,
  editorFontSize: 16,
} satisfies Config;
`,
    );

    try {
      const result = await runCli(["set", "editorFontSize", "18"], {
        SCRIPT_KIT_CONFIG_PATH: configPath,
      });
      expect(result.exitCode).toBe(0);

      const content = readFileSync(configPath, "utf8");
      expect(content).toContain("// editorFontSize: 14,");
      expect(content).toContain("\n  editorFontSize: 18,");

      const validate = await runCli(["validate"], {
        SCRIPT_KIT_CONFIG_PATH: configPath,
      });
      expect(validate.exitCode).toBe(0);
      expect(validate.stdout.data.valid).toBe(true);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("updates a real nested key while a commented sibling example survives", async () => {
    const { dir, configPath } = withTempConfig(
      `import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: { modifiers: ["meta"], key: "Semicolon" },
  // padding: { top: 99 },
  padding: { top: 8, left: 12 },
} satisfies Config;
`,
    );

    try {
      const result = await runCli(["set", "padding.top", "16"], {
        SCRIPT_KIT_CONFIG_PATH: configPath,
      });
      expect(result.exitCode).toBe(0);

      const content = readFileSync(configPath, "utf8");
      expect(content).toContain("// padding: { top: 99 },");
      expect(content).toContain("padding: { top: 16, left: 12 }");

      const validate = await runCli(["validate"], {
        SCRIPT_KIT_CONFIG_PATH: configPath,
      });
      expect(validate.exitCode).toBe(0);
      expect(validate.stdout.data.valid).toBe(true);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("appends a nested child after a trailing comment inside the parent object", async () => {
    const { dir, configPath } = withTempConfig(
      `import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: { modifiers: ["meta"], key: "Semicolon" },
  dictation: {
    selectedDeviceId: "usb-mic"
    // saveHistory: false,
  },
} satisfies Config;
`,
    );

    try {
      const result = await runCli(["set", "dictation.livePreview", "false"], {
        SCRIPT_KIT_CONFIG_PATH: configPath,
      });
      expect(result.exitCode).toBe(0);

      const content = readFileSync(configPath, "utf8");
      // Comma lands after the real property, not inside the comment.
      expect(content).toContain('selectedDeviceId: "usb-mic",');
      expect(content).toContain("// saveHistory: false,");
      expect(content).toMatch(/^\s*livePreview: false/m);

      const validate = await runCli(["validate"], {
        SCRIPT_KIT_CONFIG_PATH: configPath,
      });
      expect(validate.exitCode).toBe(0);
      expect(validate.stdout.data.valid).toBe(true);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
});
