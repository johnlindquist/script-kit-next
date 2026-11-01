"""Check the shipped arm64 images, not just the deployment setting in Cargo.toml.

ScreenCaptureKit shipped in macOS 12.3. An earlier minimum is possible if that
framework is optional, but not if dyld must load it to start the program. This
checks this known dependency and Mach-O minima; it isn't an older-OS launch test.
"""
import argparse
import json
from pathlib import Path
import plistlib
import re
import subprocess
import sys


def version(value):
    if not isinstance(value, str) or not re.fullmatch(r"[0-9]+(?:\.[0-9]+){0,2}", value):
        raise ValueError(f"invalid macOS minimum: {value!r}")
    parts = tuple(map(int, value.split(".")))
    return parts + (0,) * (3 - len(parts))


def arm64_requirements(image):
    tool = subprocess.run(["/usr/bin/otool", "-arch", "arm64", "-l", str(image)],
                          capture_output=True, text=True, timeout=30, check=True)
    minima = []
    needs_capture = False
    for block in re.split(r"(?m)^Load command [0-9]+$", tool.stdout)[1:]:
        fields = dict(line.strip().split(maxsplit=1) for line in block.splitlines()
                      if len(line.strip().split(maxsplit=1)) == 2)
        cmd = fields.get("cmd")
        if cmd == "LC_BUILD_VERSION":
            if fields.get("platform", "").lower() not in {"1", "macos"}:
                raise ValueError(f"{image.name}: not a macOS image")
            minima.append(fields.get("minos"))
        elif cmd == "LC_VERSION_MIN_MACOSX":
            minima.append(fields.get("version"))
        elif cmd in {"LC_LOAD_DYLIB", "LC_REEXPORT_DYLIB", "LC_LOAD_UPWARD_DYLIB"}:
            # New dylib_use_command can encode a weak link in an LC_LOAD_DYLIB.
            name = fields.get("name", "").partition(" (offset ")[0]
            if ("weak" not in fields.get("options", "").lower().split() and
                    re.fullmatch(r"/System/Library/Frameworks/ScreenCaptureKit\.framework/"
                                 r"(?:Versions/[^/]+/)?ScreenCaptureKit", name)):
                needs_capture = True
    if len(minima) != 1:
        raise ValueError(f"{image.name}: missing or ambiguous arm64 macOS deployment command")
    version(minima[0])
    return {"machOMinimum": minima[0], "requiresScreenCaptureKitAtLaunch": needs_capture}


def verify(app):
    contents = app / "Contents"
    with (contents / "Info.plist").open("rb") as f:
        info = plistlib.load(f)
    if not isinstance(info, dict):
        raise ValueError("invalid bundle Info.plist")
    arch_minima = info.get("LSMinimumSystemVersionByArchitecture", {})
    if not isinstance(arch_minima, dict):
        raise ValueError("invalid per-architecture deployment metadata")
    declared = arch_minima.get("arm64", info.get("LSMinimumSystemVersion"))
    floor = version(declared)
    images = {}
    for name in ("script-kit-gpui", "goose"):
        image = images[name] = arm64_requirements(contents / "MacOS" / name)
        if floor < version(image["machOMinimum"]):
            raise ValueError(f"{name}: Mach-O requires {image['machOMinimum']}; bundle declares {declared}")
        if image["requiresScreenCaptureKitAtLaunch"] and floor < (12, 3, 0):
            raise ValueError(f"{name}: ScreenCaptureKit requires macOS 12.3 at launch; bundle declares {declared}")
    return {"status": "pass", "architecture": "arm64", "declaredMinimum": declared, "images": images}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("app", type=Path)
    try:
        print(json.dumps(verify(parser.parse_args().app)))
    except (ValueError, OSError, plistlib.InvalidFileException, subprocess.SubprocessError) as exc:
        print(json.dumps({"status": "fail", "reason": str(exc)}), file=sys.stderr)
        sys.exit(1)
