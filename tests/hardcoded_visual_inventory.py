"""Reject new hardcoded GPUI color-constructor calls in Rust application source.

The scanner deliberately reuses the tested lightweight Rust lexer in
source_audit_inventory.py. It is a semantic ratchet, not a Rust parser: calls
are compared as normalized per-file multisets against the exact Git base tree.
"""

from __future__ import annotations

import argparse
import collections
import dataclasses
import io
import subprocess
import sys
import tarfile
from pathlib import Path
from typing import Iterable, Sequence

import source_audit_inventory as rust_lex


REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_ROOT = REPO_ROOT / "src"
COLOR_CONSTRUCTORS = {"rgb", "rgba", "hsl", "hsla"}

# Theme helpers own the few constructor literals that intentionally synthesize
# fallback colors. Keep this exception file-specific so renderer directories do
# not become implicit escape hatches from the ratchet.
TOKEN_OWNER_PATHS = frozenset({"src/theme/helpers.rs"})


@dataclasses.dataclass(frozen=True)
class VisualCall:
    path: str
    signature: str
    line: int


def _cfg_test_ranges(
    tokens: Sequence[rust_lex.Token], pairs: dict[int, int]
) -> list[tuple[int, int]]:
    """Return token-index ranges for items carrying an exact cfg(test) attribute."""
    ranges: list[tuple[int, int]] = []
    index = 0
    while index + 2 < len(tokens):
        if tokens[index].value != "#" or tokens[index + 1].value != "[":
            index += 1
            continue
        attribute_end = pairs.get(index + 1)
        if attribute_end is None:
            index += 1
            continue
        attribute = tokens[index + 2 : attribute_end]
        is_cfg_test = [token.value for token in attribute] == ["cfg", "(", "test", ")"]
        if not is_cfg_test:
            index = attribute_end + 1
            continue

        cursor = attribute_end + 1
        item_end = len(tokens) - 1
        while cursor < len(tokens):
            token = tokens[cursor]
            if token.value == "{" and cursor in pairs:
                item_end = pairs[cursor]
                break
            if token.value == ";":
                item_end = cursor
                break
            cursor += 1
        ranges.append((index, item_end))
        index = item_end + 1
    return ranges


def _inside(index: int, ranges: Sequence[tuple[int, int]]) -> bool:
    return any(start <= index <= end for start, end in ranges)


def _is_test_module_path(path: str) -> bool:
    parts = path.split("/")
    name = parts[-1]
    return (
        "tests" in parts[1:-1]
        or name == "tests.rs"
        or name.endswith(("_test.rs", "_tests.rs"))
    )


def _leading_expression(tokens: Sequence[rust_lex.Token]) -> tuple[rust_lex.Token, ...]:
    result = tuple(tokens)
    while result:
        if result[0].value in {"+", "-"}:
            result = result[1:]
            continue
        if result[0].value in rust_lex.OPEN_TO_CLOSE:
            closing = rust_lex.delimiter_pairs(result).get(0)
            if closing is None:
                break
            result = result[1:closing]
            continue
        break
    return result


def _starts_with_numeric_literal(tokens: Sequence[rust_lex.Token]) -> bool:
    candidate = _leading_expression(tokens)
    return bool(candidate) and candidate[0].kind == "number"


def _normalized_token(token: rust_lex.Token) -> str:
    if token.kind == "number":
        return token.value.replace("_", "").lower()
    if token.kind == "string":
        return repr(token.value)
    return token.value


def _normalized_call(
    tokens: Sequence[rust_lex.Token], pairs: dict[int, int], call: rust_lex.Call
) -> str:
    opening = pairs[call.end]
    arguments = "".join(_normalized_token(token) for token in tokens[opening + 1 : call.end])
    return f"{call.name}({arguments})"


def scan_source(path: str, source: str) -> list[VisualCall]:
    if path in TOKEN_OWNER_PATHS or _is_test_module_path(path):
        return []
    tokens = rust_lex.lex_rust(source)
    pairs = rust_lex.delimiter_pairs(tokens)
    ignored = _cfg_test_ranges(tokens, pairs)
    calls: list[VisualCall] = []
    for call in rust_lex.extract_calls(tokens, pairs):
        if (
            call.name not in COLOR_CONSTRUCTORS
            or not call.argument
            or not _starts_with_numeric_literal(call.argument)
            or _inside(call.start, ignored)
        ):
            continue
        calls.append(
            VisualCall(
                path=path,
                signature=_normalized_call(tokens, pairs, call),
                line=call.line,
            )
        )
    return sorted(calls, key=lambda call: (call.path, call.line, call.signature))


def working_tree_sources(root: Path = SRC_ROOT) -> dict[str, str]:
    return {
        path.relative_to(REPO_ROOT).as_posix(): path.read_text(encoding="utf-8")
        for path in sorted(root.rglob("*.rs"))
    }


def git_sources(revision: str) -> dict[str, str]:
    archived = subprocess.run(
        ["git", "archive", "--format=tar", revision, "src"],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
    )
    sources: dict[str, str] = {}
    with tarfile.open(fileobj=io.BytesIO(archived.stdout), mode="r:") as archive:
        for member in archive.getmembers():
            if not member.isfile() or not member.name.endswith(".rs"):
                continue
            extracted = archive.extractfile(member)
            if extracted is not None:
                sources[member.name] = extracted.read().decode("utf-8")
    return sources


def git_renames(revision: str) -> dict[str, str]:
    changed = subprocess.run(
        ["git", "diff", "--name-status", "--find-renames", revision, "--", "src"],
        cwd=REPO_ROOT,
        check=True,
        text=True,
        capture_output=True,
    )
    renames: dict[str, str] = {}
    for line in changed.stdout.splitlines():
        fields = line.split("\t")
        if len(fields) == 3 and fields[0].startswith("R"):
            _, old_path, new_path = fields
            renames[new_path] = old_path
    return renames


def inventory(sources: dict[str, str]) -> list[VisualCall]:
    return [call for path, source in sources.items() for call in scan_source(path, source)]


def signature_counters(
    calls: Iterable[VisualCall],
) -> dict[str, collections.Counter[str]]:
    counters: dict[str, collections.Counter[str]] = collections.defaultdict(collections.Counter)
    for call in calls:
        counters[call.path][call.signature] += 1
    return dict(counters)


def added_calls(
    current: Iterable[VisualCall],
    baseline: Iterable[VisualCall],
    renames: dict[str, str] | None = None,
) -> list[tuple[str, str, int]]:
    renames = renames or {}
    current_counts = signature_counters(current)
    baseline_counts = signature_counters(baseline)
    additions: list[tuple[str, str, int]] = []
    for current_path, counter in current_counts.items():
        prior_path = renames.get(current_path, current_path)
        for signature, count in (
            counter - baseline_counts.get(prior_path, collections.Counter())
        ).items():
            additions.append((current_path, signature, count))
    return sorted(additions)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", help="Git revision used as the grandfathered baseline")
    parser.add_argument("--check", action="store_true", help="Reject additions relative to --base")
    parser.add_argument("--list", action="store_true", help="Print detected calls")
    args = parser.parse_args()

    current = inventory(working_tree_sources())
    if args.list:
        for call in current:
            print(f"{call.path}:{call.line}\t{call.signature}")

    if args.check and not args.base:
        parser.error("--check requires --base")
    if not args.base:
        print(f"Hardcoded visual calls: {len(current)}")
        return 0

    baseline = inventory(git_sources(args.base))
    additions = added_calls(current, baseline, git_renames(args.base))
    if additions:
        print(
            "new or replaced hardcoded visual constructor calls are not allowed:",
            file=sys.stderr,
        )
        for path, signature, count in additions:
            suffix = f" x{count}" if count > 1 else ""
            print(f"  {path}: {signature}{suffix}", file=sys.stderr)
        return 1
    print(f"No hardcoded visual call additions relative to {args.base}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
