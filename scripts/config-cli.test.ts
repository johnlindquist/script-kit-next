import { describe, expect, it } from "bun:test";

const CLI_PATH = new URL("./config-cli.ts", import.meta.url).pathname;

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
