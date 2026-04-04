import { describe, expect, it } from "bun:test";

const CLI_PATH = new URL("./config-cli.ts", import.meta.url).pathname;

async function runValidateChange(change: unknown) {
  const proc = Bun.spawn(
    ["bun", CLI_PATH, "validate-change", JSON.stringify(change)],
    {
      stdout: "pipe",
      stderr: "pipe",
      env: { ...process.env },
    },
  );

  const stdoutText = await new Response(proc.stdout).text();
  const stderrText = await new Response(proc.stderr).text();
  const exitCode = await proc.exited;

  return {
    exitCode,
    stdout: JSON.parse(stdoutText),
    stderr: stderrText.trim(),
  };
}

describe("config-cli validate-change", () => {
  it("accepts nested command field updates", async () => {
    const result = await runValidateChange({
      key: "commands.builtin/clipboard-history.hidden",
      value: true,
    });
    expect(result.exitCode).toBe(0);
    expect(result.stdout.success).toBe(true);
    expect(result.stdout.data).toEqual({
      valid: true,
      normalizedValue: true,
      errors: [],
      warnings: [],
    });
    expect(result.stderr).toContain('"msg":"config.validate_change"');
    expect(result.stderr).toContain(
      '"commandPath":{"commandId":"builtin/clipboard-history","fieldPath":"hidden"}',
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
    expect(result.stderr).toContain('"msg":"config.validate_change"');
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
    expect(result.stdout.errors[0].code).toBe("invalidCommandPath");
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
