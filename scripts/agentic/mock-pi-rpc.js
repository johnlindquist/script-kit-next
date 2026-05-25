#!/usr/bin/env bun
/**
 * Deterministic local Pi RPC shim for Script Kit GPUI runtime proof.
 *
 * It implements only the stdio JSON commands used by the Agent Chat Pi
 * adapter: get_available_models, set_model, prompt, and abort. The shim never
 * echoes raw prompt text; stdout carries protocol events and stderr carries
 * length-only diagnostics.
 */

const decoder = new TextDecoder();
const encoder = new TextEncoder();

let buffer = "";
let activePromptId = null;

function writeJson(value) {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}

function respond(id, command, data = {}) {
  writeJson({
    type: "response",
    id,
    command,
    success: true,
    data,
  });
}

function fail(id, command, error) {
  writeJson({
    type: "response",
    id,
    command,
    success: false,
    error,
  });
}

function logMeta(event, fields = {}) {
  const payload = Object.entries(fields)
    .map(([key, value]) => `${key}=${value}`)
    .join(" ");
  process.stderr.write(`[mock-pi-rpc] event=${event}${payload ? ` ${payload}` : ""}\n`);
}

function handleCommand(command) {
  const id = typeof command.id === "string" ? command.id : "missing-id";
  const type = typeof command.type === "string" ? command.type : "unknown";

  if (type === "get_available_models") {
    logMeta("get_available_models", { id });
    respond(id, "get_available_models", {
      models: [
        {
          provider: "openai-codex",
          id: "gpt-5.4",
          name: "Mock GPT 5.4",
          contextWindow: 256000,
        },
      ],
    });
    return;
  }

  if (type === "set_model") {
    const provider = typeof command.provider === "string" ? command.provider : "";
    const modelId = typeof command.modelId === "string" ? command.modelId : "";
    logMeta("set_model", {
      id,
      providerChars: [...provider].length,
      modelChars: [...modelId].length,
    });
    respond(id, "set_model", {});
    return;
  }

  if (type === "prompt") {
    const message = typeof command.message === "string" ? command.message : "";
    activePromptId = id;
    logMeta("prompt", { id, messageChars: [...message].length });
    writeJson({
      type: "message_update",
      assistantMessageEvent: {
        type: "thinking_delta",
        delta: "reading",
      },
    });
    writeJson({
      type: "message_update",
      assistantMessageEvent: {
        type: "text_delta",
        delta: "Bonjour le monde.",
      },
    });
    writeJson({
      type: "agent_end",
    });
    activePromptId = null;
    return;
  }

  if (type === "abort") {
    logMeta("abort", { id, active: activePromptId ?? "none" });
    activePromptId = null;
    respond(id, "abort", {});
    return;
  }

  fail(id, type, `Unsupported mock Pi RPC command: ${type}`);
}

for await (const chunk of Bun.stdin.stream()) {
  buffer += decoder.decode(chunk, { stream: true });
  let newlineIndex = buffer.indexOf("\n");
  while (newlineIndex !== -1) {
    const line = buffer.slice(0, newlineIndex).trim();
    buffer = buffer.slice(newlineIndex + 1);
    if (line.length > 0) {
      try {
        handleCommand(JSON.parse(line));
      } catch (_error) {
        writeJson({
          type: "event_serialize_error",
          error: "mock Pi RPC received invalid JSON",
        });
      }
    }
    newlineIndex = buffer.indexOf("\n");
  }
}

if (buffer.trim().length > 0) {
  try {
    handleCommand(JSON.parse(buffer));
  } catch (_error) {
    process.stdout.write(encoder.encode(""));
  }
}
