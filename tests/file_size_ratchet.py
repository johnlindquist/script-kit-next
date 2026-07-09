#!/usr/bin/env python3
"""Keep production Rust files from crossing or growing past 2,000 lines."""

from __future__ import annotations

import argparse
import io
import json
import subprocess
import sys
import tarfile
from pathlib import Path, PurePosixPath
from typing import Mapping


REPO_ROOT = Path(__file__).resolve().parents[1]
GRANDFATHER_PATH = Path("tests/file_size_grandfather.json")
CRITICAL_LINES = 2_000


def is_production_rust(path: str) -> bool:
    candidate = PurePosixPath(path)
    if candidate.suffix != ".rs" or not candidate.parts or candidate.parts[0] != "src":
        return False
    if any(
        part == "tests" or part.endswith("_test") or part.endswith("_tests")
        for part in candidate.parts
    ):
        return False
    name = candidate.name
    return (
        name != "tests.rs"
        and not name.endswith("_test.rs")
        and not name.endswith("_tests.rs")
        and not name.startswith("test_")
    )


def line_count(data: bytes) -> int:
    return len(data.decode("utf-8").splitlines())


def working_tree_counts(root: Path = REPO_ROOT) -> dict[str, int]:
    counts: dict[str, int] = {}
    for path in sorted((root / "src").rglob("*.rs")):
        relative = path.relative_to(root).as_posix()
        if is_production_rust(relative):
            counts[relative] = line_count(path.read_bytes())
    return counts


def revision_counts(revision: str, root: Path = REPO_ROOT) -> dict[str, int]:
    archive = subprocess.run(
        ["git", "archive", "--format=tar", revision, "src"],
        cwd=root,
        check=True,
        stdout=subprocess.PIPE,
    ).stdout
    counts: dict[str, int] = {}
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:") as tar:
        for member in tar.getmembers():
            if not member.isfile() or not is_production_rust(member.name):
                continue
            extracted = tar.extractfile(member)
            if extracted is None:
                raise RuntimeError(f"could not read {member.name} from {revision}")
            counts[member.name] = line_count(extracted.read())
    return counts


def parse_allowlist(data: str, label: str) -> dict[str, int]:
    decoded = json.loads(data)
    if not isinstance(decoded, dict):
        raise ValueError(f"{label} must be a JSON object")
    allowlist: dict[str, int] = {}
    for path, limit in decoded.items():
        if not isinstance(path, str) or not isinstance(limit, int) or isinstance(limit, bool):
            raise ValueError(f"{label} entries must map paths to integer line limits")
        allowlist[path] = limit
    return allowlist


def working_allowlist(root: Path = REPO_ROOT) -> dict[str, int]:
    path = root / GRANDFATHER_PATH
    return parse_allowlist(path.read_text(), str(GRANDFATHER_PATH))


def revision_allowlist(revision: str, root: Path = REPO_ROOT) -> dict[str, int] | None:
    commit = subprocess.run(
        ["git", "cat-file", "-e", f"{revision}^{{commit}}"],
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if commit.returncode != 0:
        detail = commit.stderr.decode("utf-8", errors="replace").strip()
        raise RuntimeError(f"invalid base revision {revision}: {detail}")

    object_name = f"{revision}:{GRANDFATHER_PATH.as_posix()}"
    path_exists = subprocess.run(
        ["git", "cat-file", "-e", object_name],
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if path_exists.returncode != 0:
        return None

    result = subprocess.run(
        ["git", "show", object_name],
        cwd=root,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"failed to read {object_name}: {result.stderr.strip()}"
        )
    return parse_allowlist(result.stdout, f"{revision}:{GRANDFATHER_PATH}")


def evaluate(
    counts: Mapping[str, int],
    allowlist: Mapping[str, int],
    base_allowlist: Mapping[str, int] | None = None,
) -> list[str]:
    errors: list[str] = []

    for path, limit in sorted(allowlist.items()):
        if not is_production_rust(path):
            errors.append(f"invalid grandfather path: {path}")
            continue
        if limit <= CRITICAL_LINES:
            errors.append(f"invalid grandfather limit for {path}: {limit} must exceed {CRITICAL_LINES}")
        current = counts.get(path)
        if current is None:
            errors.append(f"stale grandfather entry: {path} no longer exists")
        elif current <= CRITICAL_LINES:
            errors.append(
                f"stale grandfather entry: {path} is now {current} lines; remove it from the allowlist"
            )
        elif current > limit:
            errors.append(f"critical file grew: {path} is {current} lines, grandfathered at {limit}")
        elif current < limit:
            errors.append(
                f"critical file shrank: {path} is {current} lines; lower its grandfather limit from {limit}"
            )

    for path, current in sorted(counts.items()):
        if current > CRITICAL_LINES and path not in allowlist:
            errors.append(
                f"new critical file: {path} is {current} lines; split it below {CRITICAL_LINES + 1}"
            )

    if base_allowlist is not None:
        for path in sorted(set(allowlist) - set(base_allowlist)):
            errors.append(f"grandfather allowlist may only shrink; added {path}")
        for path in sorted(set(allowlist) & set(base_allowlist)):
            if allowlist[path] > base_allowlist[path]:
                errors.append(
                    f"grandfather limit may not increase: {path} {base_allowlist[path]} -> {allowlist[path]}"
                )

    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", help="Git revision whose allowlist is the shrink-only baseline")
    parser.add_argument("--revision", help="Scan committed source from this revision, not the working tree")
    args = parser.parse_args(argv)

    allowlist = working_allowlist()
    counts = revision_counts(args.revision) if args.revision else working_tree_counts()
    base_allowlist = revision_allowlist(args.base) if args.base else None
    errors = evaluate(counts, allowlist, base_allowlist)

    critical = sum(lines > CRITICAL_LINES for lines in counts.values())
    print(
        f"Production Rust files: {len(counts)}; critical: {critical}; "
        f"grandfathered: {len(allowlist)}"
    )
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print("File-size ratchet passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
