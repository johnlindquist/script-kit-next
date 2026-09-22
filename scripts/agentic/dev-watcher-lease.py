#!/usr/bin/env python3
"""Serialize watcher registration and preserve stale leases without signalling processes."""
import ctypes
import fcntl
import hashlib
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tempfile
import uuid

FIELDS = {"pid", "process-start", "generation", "root", "session"}


def rename_exclusive(source: Path, destination: Path) -> None:
    libc = ctypes.CDLL(None, use_errno=True)
    if sys.platform == "darwin":
        rename = libc.renamex_np
        rename.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_uint]
        args = (os.fsencode(source), os.fsencode(destination), 0x4)  # RENAME_EXCL
    elif sys.platform.startswith("linux"):
        rename = libc.renameat2
        rename.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_uint]
        args = (-100, os.fsencode(source), -100, os.fsencode(destination), 1)  # RENAME_NOREPLACE
    else:
        raise RuntimeError("exclusive watcher lease publication unsupported")
    rename.restype = ctypes.c_int
    if rename(*args) != 0:
        error = ctypes.get_errno()
        raise OSError(error, os.strerror(error), str(destination))


def process_start(pid: int) -> str:
    result = subprocess.run(
        ["/bin/ps", "-p", str(pid), "-o", "lstart="],
        capture_output=True, text=True, env={**os.environ, "LC_ALL": "C"}, check=False,
    )
    return result.stdout.strip() if result.returncode == 0 else ""


def owned_directory(path: Path) -> None:
    info = path.lstat()
    if not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid():
        raise RuntimeError(f"untrusted watcher directory: {path}")


def read_lease(path: Path) -> dict[str, str]:
    owned_directory(path)
    fields = {}
    for entry in path.iterdir():
        info = entry.lstat()
        if entry.name not in FIELDS or not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid():
            raise RuntimeError(f"untrusted watcher record: {entry}")
        fields[entry.name] = entry.read_text().strip()
    return fields


def main() -> int:
    action, repo, pid_text, generation, session, recover, multi = sys.argv[1:]
    repo = str(Path(repo).resolve(strict=True))
    pid = int(pid_text)
    if pid <= 0 or action not in {"claim", "release"} or recover not in {"0", "1"} or multi not in {"0", "1"}:
        raise RuntimeError("invalid watcher lease arguments")
    base = Path("/tmp/sk-dev-launcher-locks")
    base.mkdir(mode=0o700, exist_ok=True)
    owned_directory(base)
    key = hashlib.sha1(repo.encode()).hexdigest()
    if multi == "1":
        key += f"-{pid}"
    lock = base / f"{key}.lock"
    guard = base / f"{key}.guard"
    fd = os.open(guard, os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW, 0o600)
    try:
        info = os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid():
            raise RuntimeError("untrusted watcher serialization guard")
        fcntl.flock(fd, fcntl.LOCK_EX)
        if action == "release":
            if not lock.exists() and not lock.is_symlink():
                return 0
            fields = read_lease(lock)
            if fields.get("pid") != pid_text or fields.get("generation") != generation or fields.get("root") != repo or fields.get("process-start") != process_start(pid):
                raise RuntimeError("watcher cleanup identity changed; retaining record")
            retired = base / f"{key}.closed-{uuid.uuid4()}"
            rename_exclusive(lock, retired)
            for name in fields:
                (retired / name).unlink()
            retired.rmdir()
            return 0

        if lock.exists() or lock.is_symlink():
            fields = read_lease(lock)
            if fields.get("root") != repo:
                raise RuntimeError("watcher repository identity unknown; retaining record")
            old_pid_text = fields.get("pid", "")
            old_pid = int(old_pid_text) if old_pid_text.isdecimal() else None
            if old_pid is not None and old_pid > 0:
                try:
                    os.kill(old_pid, 0)
                except ProcessLookupError:
                    pass
                else:
                    if fields.get("process-start") and fields.get("process-start") == process_start(old_pid) and fields.get("generation"):
                        print(f"[dev.sh] ERROR another ./dev.sh is already running for this repo: pid={old_pid} session={fields.get('session', '?')}", file=sys.stderr)
                        return 2
                    raise RuntimeError("live/reused watcher PID remains protected")
            elif old_pid_text:
                raise RuntimeError("malformed watcher PID remains protected")
            complete = FIELDS == fields.keys() and all(fields.values()) and old_pid is not None
            if not complete and recover != "1":
                raise RuntimeError("incomplete watcher registration; use ./dev.sh --takeover for explicit recovery")
            retired = base / f"{key}.stale-{uuid.uuid4()}"
            rename_exclusive(lock, retired)
            print(f"[dev.sh] Recovered stale watcher registration; preserved at {retired}", file=sys.stderr)

        start = process_start(pid)
        if not start or not generation:
            raise RuntimeError("new watcher process identity unavailable")
        staged = Path(tempfile.mkdtemp(prefix=f"{key}.register-", dir=base))
        try:
            fields = {"pid": pid_text, "process-start": start, "generation": generation, "root": repo, "session": session}
            for name, value in fields.items():
                (staged / name).write_text(value + "\n")
            rename_exclusive(staged, lock)
        finally:
            if staged.exists():
                shutil.rmtree(staged)
        print(generation)
        return 0
    finally:
        os.close(fd)


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, RuntimeError) as error:
        print(f"[dev.sh] REFUSED: {error}", file=sys.stderr)
        sys.exit(78)
