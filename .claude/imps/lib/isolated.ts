/**
 * Shared helper for creating fully isolated Codex SDK agents.
 *
 * Default mode is streaming (shows all events as they happen).
 * Use --quiet for buffered one-shot output.
 */

import { Codex, type CodexOptions, type ThreadOptions } from "@openai/codex-sdk";
import { rmSync, unlinkSync, writeFileSync } from "fs";
import { spawn } from "child_process";
import { ensureWarmImp, runViaWarmImp, serveImp } from "./imp.ts";
import {
  applyLessonOverlay,
  prepareIsolatedCodexHome,
  type SelfImproveConfig,
} from "./codex-runtime.ts";
import { createSelfImproveObserver } from "./self-improve.ts";

export interface ImpConfig {
  name: string;
  model?: string;
  reasoningEffort?: string;
  baseInstructions: string;
  developerInstructions: string;
  sandboxMode?: "read-only" | "workspace-write" | "danger-full-access";
  extraEnv?: Record<string, string>;
  /** Opt in to the Stop-hook self-improvement loop (see lib/codex-runtime.ts). */
  selfImprove?: SelfImproveConfig;
}

export function createIsolatedCodex(rawConfig: ImpConfig) {
  const config = applyLessonOverlay(rawConfig);
  const realHome = process.env.HOME!;
  const isolatedHome = `/tmp/codex-imp-${config.name}-${process.pid}`;
  // Symlinks auth, and (when self-improvement is enabled) writes the hook config.
  const runtime = prepareIsolatedCodexHome(config, isolatedHome, realHome);

  const model = config.model || process.env.CODEX_IMP_MODEL || process.env.CODEX_PROFILE_MODEL || "gpt-5.3-codex-spark";

  const codex = new Codex({
    env: {
      PATH: process.env.PATH || "/usr/local/bin:/usr/bin:/bin",
      HOME: realHome,
      CODEX_HOME: isolatedHome,
      ...config.extraEnv,
      ...runtime.extraEnv,
    },
    config: {
      base_instructions: config.baseInstructions,
      developer_instructions: config.developerInstructions,
      model_reasoning_effort: config.reasoningEffort || "low",
      show_raw_agent_reasoning: true,

      skills: { include_instructions: false },
      include_apps_instructions: false,
      include_environment_context: false,
      include_collaboration_mode_instructions: false,
      include_permissions_instructions: false,

      project_doc_max_bytes: 0,
      memories: { use_memories: false },
      mcp_servers: {},
      web_search: "disabled",

      // See appserver.ts: bypass hook trust so user hooks run non-interactively.
      bypass_hook_trust: runtime.hooksEnabled,

      features: {
        plugins: false,
        hooks: runtime.hooksEnabled,
        memories: false,
        apps: false,
        image_generation: false,
        tool_search: false,
        tool_suggest: false,
      },
    },
  });

  const startThread = (overrides?: Partial<ThreadOptions>) =>
    codex.startThread({
      workingDirectory: process.cwd(),
      skipGitRepoCheck: true,
      sandboxMode: config.sandboxMode || "danger-full-access",
      approvalPolicy: "never",
      ...overrides,
    });

  const cleanup = () => {
    try {
      rmSync(isolatedHome, { recursive: true, force: true });
    } catch {}
  };

  return { codex, startThread, cleanup, model, isolatedHome };
}

function buildInteractiveFlags(config: ImpConfig, hooksOn = false): string[] {
  const model = config.model || process.env.CODEX_IMP_MODEL || process.env.CODEX_PROFILE_MODEL || "gpt-5.3-codex-spark";
  return [
    "--dangerously-bypass-approvals-and-sandbox",
    "--disable", "plugins",
    ...(hooksOn
      ? ["-c", "features.hooks=true", "-c", "bypass_hook_trust=true"]
      : ["--disable", "hooks"]),
    "--disable", "memories",
    "--disable", "apps",
    "--disable", "image_generation",
    "--disable", "tool_search",
    "--disable", "tool_suggest",
    "-c", "skills.include_instructions=false",
    "-c", "include_apps_instructions=false",
    "-c", "include_environment_context=false",
    "-c", "include_collaboration_mode_instructions=false",
    "-c", "include_permissions_instructions=false",
    "-c", "project_doc_max_bytes=0",
    "-c", "memories.use_memories=false",
    "-c", "mcp_servers={}",
    "-c", 'web_search="disabled"',
    "-c", `model_reasoning_effort="${config.reasoningEffort || "low"}"`,
    "-m", model,
  ];
}

function tomlEscape(s: string): string {
  return s.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
}

/** Prompt suffix pointing the imp at piped-stdin data saved to a temp file. */
export function stdinPromptSuffix(stdinFilePath: string): string {
  return `

[piped input] Data was piped to this command on stdin and saved to the file: ${stdinFilePath}
Treat that file as the input for this task. Read it with the narrowest command (head, sed -n, jq, rg). Do not ask the user to provide the data again.`;
}

export function parseArgs(argv: string[]) {
  const args = argv.slice(2);
  const interactive = args.includes("-i") || args.includes("--interactive");
  const quiet = args.includes("-q") || args.includes("--quiet");
  const help = args.includes("--help") || args.includes("-h");
  // --daemon is a back-compat alias from before the imp rename.
  const serve = args.includes("--serve") || args.includes("--daemon");
  const noWarm = args.includes("--no-warm");
  // --effort <none|minimal|low|medium|high|xhigh>: per-turn reasoning override (warm imp path)
  const effortIdx = args.findIndex((a) => a === "--effort");
  const effort = effortIdx !== -1 ? args[effortIdx + 1] : undefined;
  const flags = ["-q", "--quiet", "-i", "--interactive", "--help", "-h", "--serve", "--daemon", "--no-warm"];
  // Drop the value following --effort only when --effort is actually present
  // (effortIdx === -1 would otherwise make effortIdx+1 === 0 and strip the first prompt word).
  const effortValueIdx = effortIdx !== -1 ? effortIdx + 1 : -1;
  const prompt = args
    .filter((a, i) => !flags.includes(a) && a !== "--effort" && i !== effortValueIdx)
    .join(" ");
  return { interactive, quiet, help, serve, noWarm, effort, prompt, noArgs: args.length === 0 };
}

// Renders streaming app-server JSON-RPC notifications (warm imp path).
// Answer tokens stream to stdout; reasoning/commands/output go to stderr.
function renderAppServerNotif(method: string, params: any) {
  switch (method) {
    case "item/agentMessage/delta":
      process.stdout.write(params?.delta ?? "");
      break;
    case "item/reasoning/textDelta":
    case "item/reasoning/summaryTextDelta":
      process.stderr.write(`\x1b[2;3m${params?.delta ?? ""}\x1b[0m`);
      break;
    case "item/started":
      if (params?.item?.type === "commandExecution") {
        process.stderr.write(`\x1b[2m$ ${params.item.command}\x1b[0m\n`);
      }
      break;
    case "item/commandExecution/outputDelta":
      if (params?.delta) process.stderr.write(`\x1b[2m${params.delta}\x1b[0m`);
      break;
    case "item/completed":
      if (params?.item?.type === "commandExecution" && params.item.exitCode && params.item.exitCode !== 0) {
        process.stderr.write(`\x1b[31m→ exit ${params.item.exitCode}\x1b[0m\n`);
      }
      break;
    case "turn/plan/updated":
      if (Array.isArray(params?.plan)) {
        for (const step of params.plan) {
          const mark = step.status === "completed" ? "✓" : "○";
          process.stderr.write(`\x1b[2m  ${mark} ${step.step ?? step.text ?? ""}\x1b[0m\n`);
        }
      }
      break;
  }
}

function renderEvent(event: any) {
  if (event.type === "item.started") {
    const item = event.item;
    if (item.type === "command_execution") {
      process.stderr.write(`\x1b[2m$ ${item.command}\x1b[0m\n`);
    }
  } else if (event.type === "item.completed") {
    const item = event.item;
    if (item.type === "agent_message") {
      console.log(item.text);
    } else if (item.type === "command_execution") {
      if (item.aggregated_output) {
        process.stderr.write(`\x1b[2m${item.aggregated_output}\x1b[0m`);
        if (!item.aggregated_output.endsWith("\n")) process.stderr.write("\n");
      }
      if (item.exit_code !== 0) {
        process.stderr.write(`\x1b[31m→ exit ${item.exit_code}\x1b[0m\n`);
      }
    } else if (item.type === "reasoning" && item.text) {
      process.stderr.write(`\x1b[2;3m${item.text}\x1b[0m\n`);
    } else if (item.type === "todo_list") {
      for (const todo of item.items) {
        const mark = todo.completed ? "✓" : "○";
        process.stderr.write(`\x1b[2m  ${mark} ${todo.text}\x1b[0m\n`);
      }
    }
  }
}

export async function runImp(rawConfig: ImpConfig) {
  // Fold any accumulated self-improvement lessons into developerInstructions
  // before anything reads them (interactive + cold paths). Idempotent.
  const config = applyLessonOverlay(rawConfig);
  const { interactive, quiet, help, serve, noWarm, effort, prompt, noArgs } = parseArgs(process.argv);

  if (help || noArgs) {
    console.log(`${config.name} — isolated codex imp (spark)

Usage:
  ${config.name} <prompt>            Run with streaming (auto-warms the imp for instant responses)
  ${config.name} -q <prompt>         Quiet mode (buffered, final answer only)
  ${config.name} -i [prompt]         Interactive codex TUI in this terminal
  ${config.name} --no-warm <prompt>  Opt out: force a cold in-process run (no warm imp)
  ${config.name} --serve             Run the warm imp server in the foreground (for supervisors)
  ${config.name} --effort <level>    Reasoning effort: none|minimal|low|medium|high|xhigh (warm imp)
  ${config.name} --help              Show this help

By default the first call auto-starts a background warm imp and every call routes
through it for ~2x lower latency. Use --no-warm to bypass it for a one-off run.`);
    process.exit(0);
  }

  if (serve) {
    await serveImp(config);
    return;
  }

  if (interactive) {
    // Launch the codex interactive TUI right here in the current terminal.
    // No cmux, no surfaces — the profile knows nothing about any terminal manager.
    const realHome = process.env.HOME!;
    const isolatedHome = `/tmp/codex-imp-${config.name}-${process.pid}-interactive`;
    const runtime = prepareIsolatedCodexHome(config, isolatedHome, realHome);
    const flags = buildInteractiveFlags(config, runtime.hooksEnabled);
    const args = [
      ...flags,
      "-c", `developer_instructions="${tomlEscape(config.developerInstructions)}"`,
      ...(prompt ? [prompt] : []),
    ];
    const cleanupInteractive = () => {
      try {
        rmSync(isolatedHome, { recursive: true, force: true });
      } catch {}
    };
    const child = spawn("codex", args, {
      stdio: "inherit",
      cwd: process.cwd(),
      env: {
        ...process.env,
        HOME: realHome,
        CODEX_HOME: isolatedHome,
        ...config.extraEnv,
        ...runtime.extraEnv,
      },
    });
    child.on("exit", (code, signal) => {
      cleanupInteractive();
      process.exit(signal ? 1 : code ?? 0);
    });
    child.on("error", (e) => {
      cleanupInteractive();
      console.error(`${config.name}: failed to launch codex: ${e.message}`);
      process.exit(1);
    });
    return;
  }

  if (!prompt) {
    console.error(`${config.name}: no prompt provided (use -i for interactive mode)`);
    process.exit(1);
  }

  // Piped stdin becomes a temp file the imp can read with shell commands, so
  // `cat data.json | imp-jq "count users"` just works. TTY stdin is ignored;
  // an open-but-empty pipe yields "" and is also ignored.
  let effectivePrompt = prompt;
  let stdinFile: string | undefined;
  if (!process.stdin.isTTY) {
    const data = await Bun.stdin.text();
    if (data.trim()) {
      stdinFile = `/tmp/codex-imp-stdin-${config.name}-${process.pid}`;
      writeFileSync(stdinFile, data);
      effectivePrompt = prompt + stdinPromptSuffix(stdinFile);
    }
  }
  const cleanupStdin = () => {
    if (stdinFile) { try { unlinkSync(stdinFile); } catch {} }
  };

  const ac = new AbortController();

  // Warm by default: auto-start (and reuse) a background imp so every call
  // routes through the persistent app-server — skips process spawn + auth/config
  // load + WebSocket prewarm (paid once at imp start). Opt out with --no-warm.
  // If the imp can't be brought up, fall through to a cold in-process run.
  if (!noWarm && (await ensureWarmImp(config))) {
    const onSignal = () => { ac.abort(); cleanupStdin(); process.exit(130); };
    process.on("SIGINT", onSignal);
    process.on("SIGTERM", onSignal);
    let streamedAnswer = false;
    let routed = true;
    try {
      await runViaWarmImp(
        config.name,
        { prompt: effectivePrompt, quiet, cwd: process.cwd(), effort },
        {
          onNotification: (method, params) => {
            if (method === "item/agentMessage/delta") streamedAnswer = true;
            renderAppServerNotif(method, params);
          },
          onFinal: (text) => {
            // In streaming mode the answer already printed via deltas; just close the line.
            if (streamedAnswer) process.stdout.write("\n");
            else if (text) console.log(text);
          },
          onError: (message) => { process.stderr.write(`\x1b[31mimp error: ${message}\x1b[0m\n`); },
        },
        ac.signal,
      );
    } catch {
      // Imp died or the connection dropped mid-flight — fall back to cold.
      routed = false;
    } finally {
      process.off("SIGINT", onSignal);
      process.off("SIGTERM", onSignal);
    }
    if (routed) {
      cleanupStdin();
      return;
    }
  }

  const { startThread, cleanup } = createIsolatedCodex(config);
  const onSignal = () => { ac.abort(); cleanupStdin(); cleanup(); process.exit(130); };
  process.on("SIGINT", onSignal);
  process.on("SIGTERM", onSignal);

  const thread = startThread();
  const observer = createSelfImproveObserver(config);

  try {
    if (quiet) {
      const turn = await thread.run(effectivePrompt, { signal: ac.signal });
      if (turn.finalResponse) console.log(turn.finalResponse);
      observer.finish({ status: "completed", transport: "sdk-quiet" });
    } else {
      const { events } = await thread.runStreamed(effectivePrompt, { signal: ac.signal });
      for await (const event of events) {
        observer.onSdkEvent(event);
        renderEvent(event);
      }
      observer.finish({ status: "completed", transport: "sdk-stream" });
    }
  } finally {
    process.off("SIGINT", onSignal);
    process.off("SIGTERM", onSignal);
    cleanupStdin();
    cleanup();
  }
}
