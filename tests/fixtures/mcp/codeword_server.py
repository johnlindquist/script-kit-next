#!/usr/bin/env python3
"""A minimal stdio MCP server for live agent tests: one tool, `lookup_codeword`,
that returns a fixed word the model could not otherwise know."""

import json
import sys

CODEWORD = "OTTER-7"


def reply(message_id, result):
    sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": message_id, "result": result}) + "\n")
    sys.stdout.flush()


for line in sys.stdin:
    try:
        message = json.loads(line)
    except json.JSONDecodeError:
        continue
    method = message.get("method")
    message_id = message.get("id")
    if message_id is None:
        continue
    if method == "initialize":
        reply(message_id, {
            "protocolVersion": message.get("params", {}).get("protocolVersion", "2025-03-26"),
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "codeword", "version": "1.0.0"},
        })
    elif method == "tools/list":
        reply(message_id, {"tools": [{
            "name": "lookup_codeword",
            "description": "Look up today's team codeword.",
            "inputSchema": {"type": "object", "properties": {}},
            "annotations": {"readOnlyHint": True},
        }]})
    elif method == "tools/call":
        reply(message_id, {"content": [{"type": "text", "text": f"The codeword is {CODEWORD}."}]})
    else:
        sys.stdout.write(json.dumps({
            "jsonrpc": "2.0", "id": message_id,
            "error": {"code": -32601, "message": f"unknown method {method}"},
        }) + "\n")
        sys.stdout.flush()
