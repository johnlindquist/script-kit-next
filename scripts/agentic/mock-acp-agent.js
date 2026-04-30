#!/usr/bin/env node

const readline = require("node:readline");

let nextSessionId = 1;
let pendingPrompt = null;

const rl = readline.createInterface({
  input: process.stdin,
  crlfDelay: Infinity,
});

function send(message) {
  process.stdout.write(`${JSON.stringify(message)}\n`);
}

function result(id, value) {
  send({ jsonrpc: "2.0", id, result: value });
}

function sessionUpdate(sessionId, text) {
  send({
    jsonrpc: "2.0",
    method: "sessionUpdate",
    params: {
      sessionId,
      update: {
        sessionUpdate: "agent_message_chunk",
        content: {
          type: "text",
          text,
        },
      },
    },
  });
}

function finishPending(stopReason) {
  if (!pendingPrompt) return;
  clearTimeout(pendingPrompt.timeout);
  const { id } = pendingPrompt;
  pendingPrompt = null;
  result(id, { stopReason });
}

rl.on("line", (line) => {
  if (!line.trim()) return;

  let message;
  try {
    message = JSON.parse(line);
  } catch (error) {
    return;
  }

  const { id, method, params } = message;

  if (method === "initialize") {
    result(id, {
      protocolVersion: params?.protocolVersion ?? 1,
      agentCapabilities: {},
      authMethods: [],
      agentInfo: {
        name: "script-kit-mock-acp",
        version: "0.0.0",
        title: "Script Kit Mock ACP",
      },
    });
    return;
  }

  if (method === "session/new") {
    result(id, { sessionId: `mock-session-${nextSessionId++}` });
    return;
  }

  if (method === "session/set_model") {
    result(id, {});
    return;
  }

  if (method === "session/prompt") {
    const sessionId = params?.sessionId;
    pendingPrompt = {
      id,
      sessionId,
      timeout: setTimeout(() => finishPending("end_turn"), 30000),
    };
    sessionUpdate(sessionId, "Mock agent is thinking until Escape cancels this turn.");
    return;
  }

  if (method === "cancel") {
    finishPending("cancelled");
  }
});
