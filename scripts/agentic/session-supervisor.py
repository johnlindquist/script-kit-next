#!/usr/bin/env python3
"""Launch a Script Kit GPUI session app and write structured exit receipts."""

from __future__ import annotations

import argparse
import json
import os
import signal
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path


def utc_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def atomic_json(path: Path, payload: dict) -> None:
    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_text(json.dumps(payload, separators=(",", ":")) + "\n", encoding="utf-8")
    tmp.replace(path)


def append_lifecycle(path: Path, payload: dict) -> None:
    with path.open("a", encoding="utf-8") as file:
        file.write(json.dumps(payload, separators=(",", ":")) + "\n")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True)
    parser.add_argument("--stdin-path", required=True)
    parser.add_argument("--stdout-path", required=True)
    parser.add_argument("--session-dir", required=True)
    parser.add_argument("--session-name", required=True)
    parser.add_argument("--generation", required=True)
    args = parser.parse_args()

    session_dir = Path(args.session_dir)
    pid_path = session_dir / "pid"
    exit_path = session_dir / "app-exit.json"
    lifecycle_path = session_dir / "lifecycle.ndjson"

    stdin_file = open(args.stdin_path, "rb", buffering=0)
    stdout_file = open(args.stdout_path, "ab", buffering=0)

    child = subprocess.Popen(
        [args.binary],
        stdin=stdin_file,
        stdout=stdout_file,
        stderr=subprocess.STDOUT,
        env=os.environ.copy(),
        start_new_session=True,
    )
    pid_path.write_text(f"{child.pid}\n", encoding="utf-8")

    def forward_signal(signum: int, _frame) -> None:
        try:
            os.killpg(child.pid, signum)
        except ProcessLookupError:
            pass

    signal.signal(signal.SIGTERM, forward_signal)
    signal.signal(signal.SIGINT, forward_signal)

    return_code = child.wait()
    if return_code < 0:
        exit_status = f"signal:{-return_code}"
    else:
        exit_status = str(return_code)

    payload = {
        "schemaVersion": 1,
        "event": "app_process_exited",
        "session": args.session_name,
        "pid": child.pid,
        "exitStatus": exit_status,
        "exitCode": return_code if return_code >= 0 else None,
        "signal": -return_code if return_code < 0 else None,
        "cleanExit": return_code == 0,
        "sessionGeneration": args.generation,
        "timestamp": utc_now(),
        "monotonicSeconds": round(time.monotonic(), 3),
    }
    atomic_json(exit_path, payload)
    append_lifecycle(lifecycle_path, payload)
    return 0


if __name__ == "__main__":
    sys.exit(main())
