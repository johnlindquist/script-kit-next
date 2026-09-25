#!/usr/bin/env python3
"""Inspect or prepare the mounted, repository-owned human development workspace."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import plistlib
import shlex
import subprocess
import sys

MIN_FREE_BYTES = 25 * 1024**3
MARKER = ".script-kit-dev-storage.json"


class Refused(Exception):
    def __init__(self, kind, message, **details):
        super().__init__(message)
        self.detail = {"type": kind, "message": message, **details}


def observe_volume(volume):
    if not os.path.ismount(volume):
        raise Refused("missing_mount", "SSD is not mounted", volume=str(volume))
    try:
        result = subprocess.run(["/usr/sbin/diskutil", "info", "-plist", str(volume)],
                                capture_output=True, timeout=10, check=True)
        info = plistlib.loads(result.stdout)
    except (OSError, subprocess.SubprocessError, ValueError) as error:
        raise Refused("mount_unavailable", "Cannot identify mounted SSD") from error
    if info.get("MountPoint") != str(volume) or not info.get("VolumeUUID"):
        raise Refused("missing_mount", "Mounted SSD identity is unavailable")
    if info.get("Internal") is not False:
        raise Refused("not_external", "Development storage must be on an external volume")
    if info.get("FilesystemType", "").lower() not in {"apfs", "hfs", "hfsx"}:
        raise Refused("unsupported_filesystem", "Use a native APFS or HFS+ volume")
    if info.get("WritableVolume") is not True or not os.access(volume, os.W_OK):
        raise Refused("not_writable", "SSD is not writable")
    space = os.statvfs(volume)
    return {"path": str(volume), "uuid": info["VolumeUUID"],
            "filesystem": info.get("FilesystemName", info["FilesystemType"]),
            "external": True, "writable": True,
            "available_bytes": space.f_bavail * space.f_frsize}


def reject_symlinks(volume, paths):
    if volume.is_symlink():
        raise Refused("symlink_target", "SSD mount path must not be a symlink")
    for path in paths:
        current = volume
        for part in Path(path).relative_to(volume).parts:
            current /= part
            if current.is_symlink():
                raise Refused("symlink_target", "Owned storage must not follow symlinks", path=str(current))


def resolve_layout(repo, env):
    if not Path(repo).is_absolute() or ".." in Path(repo).parts:
        raise Refused("invalid_repo", "Use the absolute repository path without traversal")
    repo = Path(repo).resolve(strict=True)
    if not repo.is_dir():
        raise Refused("invalid_repo", "Repository must be a directory")
    volume = Path(env.get("SCRIPT_KIT_DEV_VOLUME") or "/Volumes/OMEGA SSD")
    if not volume.is_absolute() or ".." in volume.parts:
        raise Refused("invalid_volume", "Use an absolute SSD mount path without traversal")
    root = volume / "sk" / hashlib.sha256(str(repo).encode()).hexdigest()[:12]
    target, goose = root / "target", root / "goose-sidecar"
    paths = {
        "SCRIPT_KIT_DEV_VOLUME": str(volume),
        "SCRIPT_KIT_DEV_STORAGE_ROOT": str(root),
        "CARGO_TARGET_DIR": str(target),
        "SCRIPT_KIT_DEV_GOOSE_DIR": str(goose),
        "GOOSE_SIDECAR_CACHE_DIR": str(goose / "cache"),
        "GOOSE_SIDECAR_TARGET_DIR": str(goose / "cache/cargo-target"),
        "SCRIPT_KIT_METAL_MODULE_CACHE_DIR": str(root / "clang-modules"),
        "CLANG_MODULE_CACHE_PATH": str(root / "clang-modules"),
        "SCCACHE_DIR": str(root / "sccache"),
        "TMPDIR": str(root / "tmp") + "/",
        "SCRIPT_KIT_GPUI_BINARY": str(target / "debug/script-kit-gpui"),
        "SCRIPT_KIT_GHOST_LLM_HELPER": str(target / "debug/script-kit-ghost-llm-helper"),
    }
    controlled = {key: value for key, value in paths.items()
                  if key not in {"TMPDIR", "SCRIPT_KIT_GPUI_BINARY", "SCRIPT_KIT_GHOST_LLM_HELPER"}}
    controlled["CARGO_BUILD_TARGET_DIR"] = str(target)
    for key, expected in controlled.items():
        value = env.get(key)
        if value and (".." in Path(value).parts or os.path.normpath(value) != expected):
            raise Refused("conflicting_target", f"REFUSED conflicting {key}", name=key, expected=expected)
    for key in ("CARGO_BUILD_BUILD_DIR", "CARGO_RESOLVER_LOCKFILE_PATH"):
        if env.get(key):
            raise Refused("conflicting_target", f"REFUSED conflicting {key}", name=key)
    return repo, volume, root, paths


def inspect_layout(repo, volume, root, paths, env):
    marker = root / MARKER
    owned_paths = [value for key, value in paths.items() if key != "SCRIPT_KIT_DEV_VOLUME_UUID"]
    reject_symlinks(volume, [*owned_paths, marker, root / ".locks", root / "target/release",
                             root / "goose-sidecar/goose", root / "goose-sidecar/cache/cargo-target/release"])
    mount = observe_volume(volume)
    paths["SCRIPT_KIT_DEV_VOLUME_UUID"] = mount["uuid"]
    expected_uuid = env.get("SCRIPT_KIT_DEV_VOLUME_UUID")
    if expected_uuid and expected_uuid != mount["uuid"]:
        raise Refused("swapped_mount", "SSD UUID differs from the running watcher")
    if mount["available_bytes"] < MIN_FREE_BYTES:
        raise Refused("insufficient_space", "SSD must retain at least 25 GiB free",
                      available_bytes=mount["available_bytes"])
    expected = {"repo": str(repo), "volume_uuid": mount["uuid"]}
    if marker.exists():
        try:
            if not marker.is_file() or marker.stat().st_size > 4096:
                raise ValueError("invalid marker")
            actual = json.loads(marker.read_text())
        except (OSError, ValueError) as error:
            raise Refused("foreign_marker", "Storage ownership marker is unreadable") from error
        if actual != expected:
            kind = "swapped_mount" if isinstance(actual, dict) and actual.get("repo") == str(repo) else "foreign_marker"
            raise Refused(kind, "SSD workspace belongs to a different repository or volume")
    elif root.exists() and (not root.is_dir() or any(root.iterdir())):
        raise Refused("unowned_root", "Refusing to adopt nonempty unowned storage")
    return mount, expected, marker.exists()


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", required=True)
    parser.add_argument("command", choices=("inspect", "prepare"))
    parser.add_argument("--shell", action="store_true")
    args = parser.parse_args(argv)
    try:
        if args.command == "prepare" and os.environ.get("SCRIPT_KIT_NONINTERACTIVE", "0") != "0":
            raise Refused("noninteractive", "SSD preparation is an interactive development operation")
        repo, volume, root, paths = resolve_layout(args.repo, os.environ)
        mount, expected, prepared = inspect_layout(repo, volume, root, paths, os.environ)
        if args.command == "prepare":
            # Observe again before mutation. Never create the mountpoint itself.
            second = observe_volume(volume)
            if second["uuid"] != mount["uuid"]:
                raise Refused("swapped_mount", "SSD changed during preparation")
            root.parent.mkdir(mode=0o700, exist_ok=True)
            root.mkdir(mode=0o700, exist_ok=True)
            if not prepared:
                with (root / MARKER).open("x") as output:
                    output.write(json.dumps(expected) + "\n")
                    output.flush()
                    os.fsync(output.fileno())
            for child in ("target", "goose-sidecar/cache/cargo-target", "clang-modules", "sccache", "tmp", ".locks"):
                (root / child).mkdir(mode=0o700, parents=True, exist_ok=True)
            mount, expected, prepared = inspect_layout(repo, volume, root, paths, os.environ)
        if args.shell:
            print("\n".join(f"export {key}={shlex.quote(value)}" for key, value in paths.items()))
        else:
            print(json.dumps({"ok": True, "command": args.command, "repo": str(repo),
                              "storage_root": str(root), "volume": str(volume), "mount": mount,
                              "available_bytes": mount["available_bytes"], "prepared": prepared,
                              "paths": paths}, indent=2))
        return 0
    except (Refused, OSError, ValueError) as error:
        detail = error.detail if isinstance(error, Refused) else {"type": "storage_unavailable", "message": str(error)}
        print(json.dumps({"ok": False, "command": args.command, "error": detail}),
              file=sys.stderr if args.shell else sys.stdout)
        return 78


if __name__ == "__main__":
    sys.exit(main())
