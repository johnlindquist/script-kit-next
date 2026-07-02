/**
 * Shared runtime helpers for isolated Codex imp profiles.
 */
import {
  createHash,
} from "crypto";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  symlinkSync,
  writeFileSync,
} from "fs";
import { join } from "path";
import type { ImpConfig } from "./isolated.ts";

export interface PreparedCodexHome {
  /** Env vars to merge into the spawned Codex process. */
  extraEnv: Record<string, string>;
  hooksEnabled: boolean;
}

export function sourceCodexHome(realHome = process.env.HOME!): string {
  return process.env.CODEX_HOME || join(realHome, ".codex");
}

export function trustedProjectsConfig(configToml: string): string {
  const blocks: string[] = [];

  let currentHeader: string | undefined;
  let currentBody: string[] = [];

  const flush = () => {
    if (!currentHeader) return;
    const trustMatch = currentBody.join("\n").match(/^trust_level\s*=\s*"(trusted|untrusted)"\s*$/m);
    if (trustMatch) {
      blocks.push(`${currentHeader}\ntrust_level = "${trustMatch[1]}"`);
    }
  };

  for (const line of configToml.split(/\r?\n/)) {
    const projectHeader = line.match(/^\[projects\."((?:\\.|[^"\\])*)"\]\s*$/);
    if (projectHeader) {
      flush();
      currentHeader = `[projects."${projectHeader[1]}"]`;
      currentBody = [];
      continue;
    }

    if (/^\[/.test(line)) {
      flush();
      currentHeader = undefined;
      currentBody = [];
      continue;
    }

    if (currentHeader) currentBody.push(line);
  }
  flush();

  return blocks.length ? `${blocks.join("\n\n")}\n` : "";
}

function copyTrustedProjects(sourceHome: string, isolatedHome: string) {
  const configSrc = join(sourceHome, "config.toml");
  const configDst = join(isolatedHome, "config.toml");
  if (!existsSync(configSrc) || existsSync(configDst)) return;

  const config = trustedProjectsConfig(readFileSync(configSrc, "utf8"));
  if (config) writeFileSync(configDst, config);
}

function appendImpTuiDefaults(isolatedHome: string): void {
  const configPath = join(isolatedHome, "config.toml");
  const current = existsSync(configPath) ? readFileSync(configPath, "utf8") : "";
  const prefix = current.trim() ? `${current.replace(/\s*$/, "\n\n")}` : "";
  writeFileSync(configPath, `${prefix}[tui]\nshow_tooltips = false\n`, "utf8");
}

function symlinkAuth(sourceHome: string, isolatedHome: string) {
  const authSrc = join(sourceHome, "auth.json");
  const authDst = join(isolatedHome, "auth.json");
  if (existsSync(authSrc) && !existsSync(authDst)) {
    symlinkSync(authSrc, authDst);
  }
}

function shellQuote(value: string): string {
  return `'${value.replace(/'/g, `'\\''`)}'`;
}

function tomlEscape(value: string): string {
  return value.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
}

function canonicalJson(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(canonicalJson);
  if (value && typeof value === "object") {
    const out: Record<string, unknown> = {};
    for (const key of Object.keys(value).sort()) {
      out[key] = canonicalJson((value as Record<string, unknown>)[key]);
    }
    return out;
  }
  return value;
}

function versionForTomlShape(value: unknown): string {
  const serialized = JSON.stringify(canonicalJson(value));
  return `sha256:${createHash("sha256").update(serialized).digest("hex")}`;
}

function bundledHookTrustedHash(command: string): string {
  return versionForTomlShape({
    event_name: "user_prompt_submit",
    hooks: [
      {
        async: false,
        command,
        statusMessage: "Checking imp evolution feedback",
        timeout: 10,
        type: "command",
      },
    ],
  });
}

function appendBundledHookTrust(isolatedHome: string, hookKey: string, trustedHash: string): void {
  const configPath = join(isolatedHome, "config.toml");
  const current = existsSync(configPath) ? readFileSync(configPath, "utf8") : "";
  const prefix = current.trim() ? `${current.replace(/\s*$/, "\n\n")}` : "";
  writeFileSync(
    configPath,
    `${prefix}[hooks.state."${tomlEscape(hookKey)}"]\ntrusted_hash = "${tomlEscape(trustedHash)}"\n`,
    "utf8",
  );
}

function writeBundledUserPromptSubmitHook(config: ImpConfig, isolatedHome: string): boolean {
  if (!config.bundledUserPromptSubmitHookSource) return false;
  const hooksDir = join(isolatedHome, "hooks");
  const hookPath = join(hooksDir, "imps-plus-user-prompt-submit.ts");
  const hooksJsonPath = join(isolatedHome, "hooks.json");
  const command = `${shellQuote(process.execPath)} ${shellQuote(hookPath)}`;
  mkdirSync(hooksDir, { recursive: true });
  writeFileSync(hookPath, config.bundledUserPromptSubmitHookSource, "utf8");
  writeFileSync(
    hooksJsonPath,
    JSON.stringify(
      {
        hooks: {
          UserPromptSubmit: [
            {
              hooks: [
                {
                  type: "command",
                  command,
                  timeout: 10,
                  statusMessage: "Checking imp evolution feedback",
                },
              ],
            },
          ],
        },
      },
      null,
      2,
    ) + "\n",
    "utf8",
  );
  appendBundledHookTrust(
    isolatedHome,
    `${realpathSync(hooksJsonPath)}:user_prompt_submit:0:0`,
    bundledHookTrustedHash(command),
  );
  return true;
}

export function prepareIsolatedCodexHome(
  config: ImpConfig,
  isolatedHome: string,
  realHome = process.env.HOME!,
): PreparedCodexHome {
  mkdirSync(isolatedHome, { recursive: true });

  const sourceHome = sourceCodexHome(realHome);
  symlinkAuth(sourceHome, isolatedHome);
  copyTrustedProjects(sourceHome, isolatedHome);
  appendImpTuiDefaults(isolatedHome);
  const hooksEnabled = writeBundledUserPromptSubmitHook(config, isolatedHome);

  return { extraEnv: {}, hooksEnabled };
}
