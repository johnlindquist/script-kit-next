#!/usr/bin/env python3
"""Inventory Rust test readers and reject new app-source read sites.

The scanner is intentionally dependency-free so the ordinary PR gate can run on
Ubuntu without compiling the application.  It is a small Rust lexer, not a Rust
parser: comments are discarded, cooked/raw strings are decoded, delimiters are
balanced, and common path expressions plus local reader wrappers are resolved.
"""

from __future__ import annotations

import argparse
import collections
import dataclasses
import io
import itertools
import json
import posixpath
import subprocess
import sys
import tarfile
from pathlib import Path
from typing import Iterable, Sequence


REPO_ROOT = Path(__file__).resolve().parents[1]
TESTS_ROOT = REPO_ROOT / "tests"
INVENTORY_DOC = TESTS_ROOT / "source_audit_inventory.md"
REPO_MARKER = "$REPO"

OPEN_TO_CLOSE = {"(": ")", "[": "]", "{": "}"}
CLOSE_TO_OPEN = {value: key for key, value in OPEN_TO_CLOSE.items()}
PRIMITIVE_NAMES = {
    "include_str",
    "include_bytes",
    "read_to_string",
    "read_dir",
    "fs_read",
    "file_open",
    "glob",
    "walkdir",
}
COMMON_READER_WRAPPERS = {"read", "read_source", "source", "repo_file", "production_source"}


@dataclasses.dataclass(frozen=True)
class Token:
    kind: str
    value: str
    line: int


@dataclasses.dataclass(frozen=True)
class Call:
    start: int
    end: int
    callee: str
    name: str
    argument: tuple[Token, ...]
    line: int
    primitive: str | None


@dataclasses.dataclass(frozen=True)
class FunctionInfo:
    name: str
    params: frozenset[str]
    body_start: int
    body_end: int


@dataclasses.dataclass(frozen=True)
class Binding:
    name: str
    expression: tuple[Token, ...]
    index: int
    scope_start: int | None
    scope_end: int | None


@dataclasses.dataclass(frozen=True)
class ReaderSite:
    test_path: str
    reader: str
    target: str
    category: str
    line: int

    @property
    def signature(self) -> tuple[str, str, str]:
        # Line numbers and formatting are intentionally absent. Counter
        # multiplicity still catches a second identical read in the same file.
        return (self.reader, self.target, self.category)


def _raw_string_at(source: str, start: int) -> tuple[str, int] | None:
    """Return (contents, end) for a Rust raw string beginning at start."""
    cursor = start
    if source.startswith(("br", "rb", "cr", "rc"), cursor):
        cursor += 1
    if cursor >= len(source) or source[cursor] != "r":
        return None
    cursor += 1
    hashes = 0
    while cursor < len(source) and source[cursor] == "#":
        hashes += 1
        cursor += 1
    if cursor >= len(source) or source[cursor] != '"':
        return None
    content_start = cursor + 1
    terminator = '"' + ("#" * hashes)
    content_end = source.find(terminator, content_start)
    if content_end < 0:
        raise ValueError(f"unterminated raw string at byte {start}")
    return source[content_start:content_end], content_end + len(terminator)


def _cooked_string_at(source: str, start: int) -> tuple[str, int] | None:
    cursor = start
    if cursor + 1 < len(source) and source[cursor] in "bc" and source[cursor + 1] == '"':
        cursor += 1
    if cursor >= len(source) or source[cursor] != '"':
        return None
    cursor += 1
    chars: list[str] = []
    while cursor < len(source):
        char = source[cursor]
        if char == '"':
            return "".join(chars), cursor + 1
        if char != "\\":
            chars.append(char)
            cursor += 1
            continue
        cursor += 1
        if cursor >= len(source):
            break
        escape = source[cursor]
        simple = {"n": "\n", "r": "\r", "t": "\t", "0": "\0", "\\": "\\", '"': '"'}
        if escape in simple:
            chars.append(simple[escape])
            cursor += 1
        elif escape == "x" and cursor + 2 < len(source):
            try:
                chars.append(chr(int(source[cursor + 1 : cursor + 3], 16)))
                cursor += 3
            except ValueError:
                chars.extend(["\\", escape])
                cursor += 1
        elif escape == "u" and cursor + 1 < len(source) and source[cursor + 1] == "{":
            close = source.find("}", cursor + 2)
            if close < 0:
                raise ValueError(f"unterminated unicode escape at byte {start}")
            chars.append(chr(int(source[cursor + 2 : close].replace("_", ""), 16)))
            cursor = close + 1
        elif escape == "\n":
            cursor += 1
            while cursor < len(source) and source[cursor] in " \t":
                cursor += 1
        else:
            chars.extend(["\\", escape])
            cursor += 1
    raise ValueError(f"unterminated string at byte {start}")


def lex_rust(source: str) -> list[Token]:
    """Lex enough Rust to distinguish code from comments and string contents."""
    tokens: list[Token] = []
    cursor = 0
    line = 1
    while cursor < len(source):
        char = source[cursor]
        if char.isspace():
            line += char == "\n"
            cursor += 1
            continue
        if source.startswith("//", cursor):
            end = source.find("\n", cursor + 2)
            if end < 0:
                break
            line += 1
            cursor = end + 1
            continue
        if source.startswith("/*", cursor):
            depth = 1
            end = cursor + 2
            while end < len(source) and depth:
                if source.startswith("/*", end):
                    depth += 1
                    end += 2
                elif source.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    line += source[end] == "\n"
                    end += 1
            if depth:
                raise ValueError(f"unterminated block comment at byte {cursor}")
            cursor = end
            continue

        raw = _raw_string_at(source, cursor)
        if raw is not None:
            value, end = raw
            tokens.append(Token("string", value, line))
            line += source[cursor:end].count("\n")
            cursor = end
            continue
        cooked = _cooked_string_at(source, cursor)
        if cooked is not None:
            value, end = cooked
            tokens.append(Token("string", value, line))
            line += source[cursor:end].count("\n")
            cursor = end
            continue

        if char == "'":
            # A quote followed by an identifier is normally a lifetime. A
            # quoted character is irrelevant to path resolution, so skip it.
            if cursor + 2 < len(source) and source[cursor + 2] == "'":
                cursor += 3
                continue
            tokens.append(Token("punct", char, line))
            cursor += 1
            continue
        if char.isalpha() or char == "_":
            end = cursor + 1
            while end < len(source) and (source[end].isalnum() or source[end] == "_"):
                end += 1
            tokens.append(Token("ident", source[cursor:end], line))
            cursor = end
            continue
        if char.isdigit():
            end = cursor + 1
            while end < len(source) and (source[end].isalnum() or source[end] in "_."):
                end += 1
            tokens.append(Token("number", source[cursor:end], line))
            cursor = end
            continue
        pair = source[cursor : cursor + 2]
        if pair in {"::", "->", "=>", "&&", "||", "==", "!=", "<=", ">=", ".."}:
            tokens.append(Token("punct", pair, line))
            cursor += 2
        else:
            tokens.append(Token("punct", char, line))
            cursor += 1
    return tokens


def delimiter_pairs(tokens: Sequence[Token]) -> dict[int, int]:
    pairs: dict[int, int] = {}
    stack: list[tuple[str, int]] = []
    for index, token in enumerate(tokens):
        if token.value in OPEN_TO_CLOSE:
            stack.append((token.value, index))
        elif token.value in CLOSE_TO_OPEN:
            if not stack or stack[-1][0] != CLOSE_TO_OPEN[token.value]:
                continue
            _, opening = stack.pop()
            pairs[opening] = index
            pairs[index] = opening
    return pairs


def _callee_before(tokens: Sequence[Token], opening: int) -> tuple[str, int] | None:
    if opening == 0 or tokens[opening - 1].kind != "ident":
        return None
    end = opening - 1
    start = end
    while start >= 2 and tokens[start - 1].value in {"::", "."} and tokens[start - 2].kind == "ident":
        start -= 2
    return "".join(token.value for token in tokens[start : end + 1]), start


def _first_argument(tokens: Sequence[Token]) -> tuple[Token, ...]:
    depth = 0
    result: list[Token] = []
    for token in tokens:
        if token.value in OPEN_TO_CLOSE:
            depth += 1
        elif token.value in CLOSE_TO_OPEN:
            depth -= 1
        if token.value == "," and depth == 0:
            break
        result.append(token)
    return tuple(result)


def _primitive_kind(callee: str, name: str, macro: bool) -> str | None:
    if macro and name in {"include_str", "include_bytes"}:
        return name
    if macro:
        return None
    if name in {"read_to_string", "read_dir"}:
        return name
    if name == "read" and (callee == "fs::read" or callee.endswith("::fs::read")):
        return "fs_read"
    if name == "open" and (callee == "File::open" or callee.endswith("::File::open")):
        return "file_open"
    if name in {"glob", "glob_with"}:
        return "glob"
    if name == "new" and (callee == "WalkDir::new" or callee.endswith("::WalkDir::new")):
        return "walkdir"
    return None


def extract_calls(tokens: Sequence[Token], pairs: dict[int, int]) -> list[Call]:
    calls: list[Call] = []
    for opening, closing in sorted(pairs.items()):
        if opening > closing or tokens[opening].value not in OPEN_TO_CLOSE:
            continue
        macro = opening >= 2 and tokens[opening - 1].value == "!" and tokens[opening - 2].kind == "ident"
        if macro:
            name = tokens[opening - 2].value
            callee = name
            start = opening - 2
        else:
            found = _callee_before(tokens, opening)
            if found is None:
                continue
            callee, start = found
            name = callee.split("::")[-1].split(".")[-1]
            if start > 0 and tokens[start - 1].value == "fn":
                continue
        calls.append(
            Call(
                start=start,
                end=closing,
                callee=callee,
                name=name,
                argument=_first_argument(tokens[opening + 1 : closing]),
                line=tokens[start].line,
                primitive=_primitive_kind(callee, name, macro),
            )
        )
    return calls


def extract_functions(tokens: Sequence[Token], pairs: dict[int, int]) -> list[FunctionInfo]:
    functions: list[FunctionInfo] = []
    for index, token in enumerate(tokens):
        if token.value != "fn" or index + 2 >= len(tokens) or tokens[index + 1].kind != "ident":
            continue
        opening = index + 2
        while opening < len(tokens) and tokens[opening].value != "(":
            opening += 1
        if opening not in pairs:
            continue
        params_end = pairs[opening]
        params: set[str] = set()
        cursor = opening + 1
        while cursor < params_end:
            if tokens[cursor].kind == "ident" and cursor + 1 < params_end and tokens[cursor + 1].value == ":":
                params.add(tokens[cursor].value)
            cursor += 1
        body_start = params_end + 1
        while body_start < len(tokens) and tokens[body_start].value != "{":
            body_start += 1
        if body_start not in pairs:
            continue
        functions.append(
            FunctionInfo(tokens[index + 1].value, frozenset(params), body_start, pairs[body_start])
        )
    return functions


def _expression_until(tokens: Sequence[Token], start: int, stops: set[str]) -> tuple[Token, ...]:
    depth = 0
    result: list[Token] = []
    for token in tokens[start:]:
        if depth == 0 and token.value in stops:
            break
        if token.value in OPEN_TO_CLOSE:
            depth += 1
        elif token.value in CLOSE_TO_OPEN:
            depth -= 1
        result.append(token)
    return tuple(result)


def extract_bindings(tokens: Sequence[Token], functions: Sequence[FunctionInfo]) -> list[Binding]:
    bindings: list[Binding] = []

    def scope_at(index: int) -> tuple[int | None, int | None]:
        for function in functions:
            if function.body_start < index < function.body_end:
                return function.body_start, function.body_end
        return None, None

    for index, token in enumerate(tokens):
        if token.value in {"const", "static", "let"}:
            cursor = index + 1
            while cursor < len(tokens) and tokens[cursor].value in {"mut", "ref"}:
                cursor += 1
            if cursor >= len(tokens) or tokens[cursor].kind != "ident":
                continue
            name = tokens[cursor].value
            while cursor < len(tokens) and tokens[cursor].value not in {"=", ";"}:
                cursor += 1
            if cursor < len(tokens) and tokens[cursor].value == "=":
                scope_start, scope_end = scope_at(index)
                bindings.append(
                    Binding(
                        name,
                        _expression_until(tokens, cursor + 1, {";"}),
                        index,
                        scope_start,
                        scope_end,
                    )
                )
        elif token.value == "for" and index + 2 < len(tokens) and tokens[index + 1].kind == "ident":
            cursor = index + 2
            while cursor < len(tokens) and tokens[cursor].value != "in":
                cursor += 1
            if cursor < len(tokens):
                scope_start, scope_end = scope_at(index)
                bindings.append(
                    Binding(
                        tokens[index + 1].value,
                        _expression_until(tokens, cursor + 1, {"{"}),
                        index,
                        scope_start,
                        scope_end,
                    )
                )
    return bindings


def bindings_at(
    all_bindings: Sequence[Binding],
    functions: Sequence[FunctionInfo],
    position: int,
) -> dict[str, list[tuple[Token, ...]]]:
    active_scope: tuple[int | None, int | None] = (None, None)
    for function in functions:
        if function.body_start < position < function.body_end:
            active_scope = (function.body_start, function.body_end)
            break

    grouped: dict[str, list[Binding]] = collections.defaultdict(list)
    for binding in all_bindings:
        is_module = binding.scope_start is None
        is_active_local = (
            binding.scope_start == active_scope[0]
            and binding.scope_end == active_scope[1]
            and binding.index < position
        )
        if is_module or is_active_local:
            grouped[binding.name].append(binding)

    resolved: dict[str, list[tuple[Token, ...]]] = {}
    for name, candidates in grouped.items():
        locals_for_name = [binding for binding in candidates if binding.scope_start is not None]
        if locals_for_name:
            nearest = max(locals_for_name, key=lambda binding: binding.index)
            resolved[name] = [nearest.expression]
        else:
            # Module constants can be referenced before declaration. Retain all
            # definitions only when their normalized expressions agree.
            unique = {_normalized_tokens(binding.expression): binding.expression for binding in candidates}
            resolved[name] = list(unique.values())
    return resolved


def _strip_outer(tokens: tuple[Token, ...]) -> tuple[Token, ...]:
    while tokens and tokens[0].value in {"&", "*"}:
        tokens = tokens[1:]
    changed = True
    while changed and len(tokens) >= 2 and tokens[0].value == "(" and tokens[-1].value == ")":
        depth = 0
        changed = False
        for index, token in enumerate(tokens):
            if token.value == "(":
                depth += 1
            elif token.value == ")":
                depth -= 1
                if depth == 0:
                    if index == len(tokens) - 1:
                        tokens = tokens[1:-1]
                        changed = True
                    break
    return tokens


def _split_top_level(tokens: Sequence[Token], separator: str = ",") -> list[tuple[Token, ...]]:
    parts: list[list[Token]] = [[]]
    depth = 0
    for token in tokens:
        if token.value in OPEN_TO_CLOSE:
            depth += 1
        elif token.value in CLOSE_TO_OPEN:
            depth -= 1
        if token.value == separator and depth == 0:
            parts.append([])
        else:
            parts[-1].append(token)
    return [tuple(part) for part in parts if part]


def _matching_local(tokens: Sequence[Token], opening: int) -> int | None:
    expected = OPEN_TO_CLOSE.get(tokens[opening].value)
    if expected is None:
        return None
    depth = 0
    for index in range(opening, len(tokens)):
        if tokens[index].value == tokens[opening].value:
            depth += 1
        elif tokens[index].value == expected:
            depth -= 1
            if depth == 0:
                return index
    return None


def _normalized_tokens(tokens: Sequence[Token]) -> str:
    return "".join(json.dumps(token.value) if token.kind == "string" else token.value for token in tokens)


def resolve_paths(
    expression: Sequence[Token],
    bindings: dict[str, list[tuple[Token, ...]]],
    seen: frozenset[str] = frozenset(),
    depth: int = 0,
) -> list[str]:
    if depth > 12:
        return []
    tokens = _strip_outer(tuple(expression))
    if not tokens:
        return []
    if len(tokens) == 1 and tokens[0].kind == "string":
        return [tokens[0].value]
    if len(tokens) == 1 and tokens[0].kind == "ident":
        name = tokens[0].value
        if name in seen:
            return []
        return list(
            dict.fromkeys(
                path
                for bound in bindings.get(name, [])
                for path in resolve_paths(bound, bindings, seen | {name}, depth + 1)
            )
        )

    # Arrays and tuples are path sets (scan roots are commonly expressed this way).
    if tokens[0].value in {"[", "("}:
        closing = _matching_local(tokens, 0)
        if closing == len(tokens) - 1:
            return list(
                dict.fromkeys(
                    path
                    for part in _split_top_level(tokens[1:-1])
                    for path in resolve_paths(part, bindings, seen, depth + 1)
                )
            )

    # concat!(env!("CARGO_MANIFEST_DIR"), "/src/foo.rs")
    if len(tokens) >= 4 and tokens[0].value == "concat" and tokens[1].value == "!" and tokens[2].value in OPEN_TO_CLOSE:
        close = _matching_local(tokens, 2)
        if close is not None:
            pieces = [resolve_paths(part, bindings, seen, depth + 1) for part in _split_top_level(tokens[3:close])]
            if pieces and all(pieces):
                return ["".join(combo) for combo in itertools.product(*pieces)]
    if len(tokens) >= 4 and tokens[0].value == "env" and tokens[1].value == "!":
        strings = [token.value for token in tokens if token.kind == "string"]
        if strings and strings[0] == "CARGO_MANIFEST_DIR":
            return [REPO_MARKER]

    # Resolve Path::new(...), PathBuf::from(...), then ordered .join(...) calls.
    first_join = next((i for i in range(len(tokens) - 1) if tokens[i].value == "." and tokens[i + 1].value == "join"), None)
    base_tokens = tokens if first_join is None else tokens[:first_join]
    base_paths: list[str] = []
    if len(base_tokens) >= 4 and base_tokens[-1].value == ")":
        opening = next((i for i, token in enumerate(base_tokens) if token.value == "("), None)
        if opening is not None:
            callee = "".join(token.value for token in base_tokens[:opening])
            close = _matching_local(base_tokens, opening)
            path_constructors = ("Path::new", "PathBuf::from", "Utf8Path::new", "Utf8PathBuf::from")
            if (
                close == len(base_tokens) - 1
                and any(callee == constructor or callee.endswith("::" + constructor) for constructor in path_constructors)
            ):
                base_paths = resolve_paths(base_tokens[opening + 1 : close], bindings, seen, depth + 1)
    if first_join is not None:
        if not base_paths:
            base_paths = resolve_paths(base_tokens, bindings, seen, depth + 1)
        cursor = first_join
        paths = base_paths
        while cursor + 2 < len(tokens) and tokens[cursor : cursor + 2][0].value == "." and tokens[cursor + 1].value == "join":
            opening = cursor + 2
            if tokens[opening].value != "(":
                break
            close = _matching_local(tokens, opening)
            if close is None:
                break
            components = resolve_paths(tokens[opening + 1 : close], bindings, seen, depth + 1)
            if not paths or not components:
                return []
            paths = [posixpath.join(base, component) for base in paths for component in components]
            cursor = close + 1
        if paths:
            return paths
    if base_paths:
        return base_paths

    # Conservative fallback for simple expressions: collect path-looking
    # literals, but do not chase every identifier in an arbitrary expression.
    # Bindings are resolved when the expression itself is a single identifier;
    # chasing names such as `path` here crosses Rust scopes and creates noise.
    resolved: list[str] = []
    for token in tokens:
        if token.kind == "string" and _looks_like_path(token.value):
            resolved.append(token.value)
    return list(dict.fromkeys(resolved))


def _looks_like_path(value: str) -> bool:
    normalized = value.replace("\\", "/")
    return (
        normalized == "src"
        or normalized.startswith(("src/", "../", "./", "/"))
        or "/" in normalized
        or normalized.endswith((".rs", ".md", ".json", ".jsonl", ".ts", ".tsx", ".sh", ".toml"))
    )


def normalize_target(test_path: str, path: str, reader: str) -> str:
    path = path.replace("\\", "/")
    repo_relative = path == REPO_MARKER or path.startswith(REPO_MARKER + "/")
    if path == REPO_MARKER:
        return "."
    if path.startswith(REPO_MARKER + "/"):
        path = path[len(REPO_MARKER) + 1 :]
    if reader in {"include_str", "include_bytes"} and not path.startswith("/") and not repo_relative:
        path = posixpath.join(posixpath.dirname(test_path), path)
    normalized = posixpath.normpath(path)
    if normalized.startswith(str(REPO_ROOT).replace("\\", "/") + "/"):
        normalized = normalized[len(str(REPO_ROOT)) + 1 :]
    return normalized.removeprefix("./")


def classify_target(test_path: str, target: str, source: str) -> str:
    if test_path == "tests/source_audit_ratchet.rs":
        return "meta-ratchet"
    if target.startswith("<dynamic:"):
        literal_paths = [token.value.replace("\\", "/") for token in lex_rust(source) if token.kind == "string"]
        # Exact `"src"` roots are the unresolved whole-tree walk shape. File
        # paths beginning with `src/` should normally resolve through a call or
        # binding; treating every such literal as evidence would reintroduce
        # false positives from diagnostics and embedded source fixtures.
        if "src" in literal_paths:
            return "app-source-audit"
        if any(
            value.startswith("tests/fixtures/")
            or value.startswith("tests/golden/")
            or "/fixtures/" in value
            or "/golden/" in value
            for value in literal_paths
        ):
            return "fixture-golden-reader"
        if any(
            value.endswith(".md")
            or value.startswith((".agents/", "docs/"))
            for value in literal_paths
        ):
            return "docs-policy-reader"
        if any(marker in source for marker in ("tempfile", "TempDir", "temp_dir", "File::create", "fs::write")):
            return "generated-runtime-artifact-reader"
        return "unresolved-reader"
    absolute_target = target.startswith("/")
    normalized = target.lstrip("/")
    if normalized == "src" or normalized.startswith("src/"):
        return "app-source-audit"
    if normalized.startswith("tests/fixtures/") or "/fixtures/" in normalized or "/golden/" in normalized:
        return "fixture-golden-reader"
    basename = posixpath.basename(normalized)
    if (
        normalized.endswith(".md")
        or normalized.startswith((".agents/", "docs/"))
        or basename in {"AGENTS.md", "CLAUDE.md", "FEATURES.md", "GLOSSARY.md", "README.md"}
    ):
        return "docs-policy-reader"
    if normalized.startswith("target/") or (absolute_target and normalized.startswith("tmp/")):
        return "generated-runtime-artifact-reader"
    return "other-repo-artifact-reader"


def discover_wrappers(calls: Sequence[Call], functions: Sequence[FunctionInfo]) -> set[str]:
    wrappers: set[str] = set()
    changed = True
    while changed:
        changed = False
        for function in functions:
            if not function.params or function.name in wrappers:
                continue
            for call in calls:
                if not (function.body_start < call.start < function.body_end):
                    continue
                calls_reader = call.primitive in PRIMITIVE_NAMES or call.name in wrappers
                # A helper can route its parameter through a local Path binding
                # before the primitive call. Requiring the parameter to appear
                # in the final call misses that common shape.
                if calls_reader:
                    wrappers.add(function.name)
                    changed = True
                    break
    return wrappers


def scan_source(test_path: str, source: str) -> list[ReaderSite]:
    tokens = lex_rust(source)
    pairs = delimiter_pairs(tokens)
    calls = extract_calls(tokens, pairs)
    functions = extract_functions(tokens, pairs)
    all_bindings = extract_bindings(tokens, functions)
    wrappers = discover_wrappers(calls, functions)
    sites: list[ReaderSite] = []
    for call in calls:
        bindings = bindings_at(all_bindings, functions, call.start)
        paths = resolve_paths(call.argument, bindings)
        reader = call.primitive
        if reader is None and (
            call.name in wrappers
            or (
                (call.name in COMMON_READER_WRAPPERS or (call.name.startswith("read_") and "source" in call.name))
                and bool(paths)
            )
        ):
            reader = f"wrapper:{call.name}"
        if reader is None:
            continue
        if not paths:
            paths = [f"<dynamic:{_normalized_tokens(call.argument)}>" ]
        for path in paths:
            target = path if path.startswith("<dynamic:") else normalize_target(test_path, path, reader)
            sites.append(
                ReaderSite(
                    test_path=test_path,
                    reader=reader,
                    target=target,
                    category=classify_target(test_path, target, source),
                    line=call.line,
                )
            )
    return sorted(sites, key=lambda site: (site.test_path, site.line, site.reader, site.target))


def working_tree_sources(root: Path = TESTS_ROOT) -> dict[str, str]:
    result: dict[str, str] = {}
    for path in sorted(root.rglob("*.rs")):
        rel = path.relative_to(REPO_ROOT).as_posix() if path.is_relative_to(REPO_ROOT) else path.name
        result[rel] = path.read_text(encoding="utf-8")
    return result


def git_sources(revision: str) -> dict[str, str]:
    """Read the base test tree through one git process, not one process per file."""
    archived = subprocess.run(
        ["git", "archive", "--format=tar", revision, "tests"],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
    )
    result: dict[str, str] = {}
    with tarfile.open(fileobj=io.BytesIO(archived.stdout), mode="r:") as archive:
        for member in archive.getmembers():
            if not member.isfile() or not member.name.endswith(".rs"):
                continue
            extracted = archive.extractfile(member)
            if extracted is None:
                continue
            result[member.name] = extracted.read().decode("utf-8")
    return result


def git_renames(revision: str) -> dict[str, str]:
    changed = subprocess.run(
        ["git", "diff", "--name-status", "--find-renames", revision, "--", "tests"],
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


def inventory(sources: dict[str, str]) -> list[ReaderSite]:
    return [site for path, source in sources.items() for site in scan_source(path, source)]


def signature_counters(sites: Iterable[ReaderSite]) -> dict[str, collections.Counter[tuple[str, str, str]]]:
    result: dict[str, collections.Counter[tuple[str, str, str]]] = collections.defaultdict(collections.Counter)
    for site in sites:
        if site.category == "app-source-audit" or site.target.startswith("<dynamic:"):
            result[site.test_path][site.signature] += 1
    return dict(result)


def added_guarded_sites(
    current: Iterable[ReaderSite],
    baseline: Iterable[ReaderSite],
    renames: dict[str, str] | None = None,
) -> list[tuple[str, tuple[str, str, str], int]]:
    renames = renames or {}
    current_counts = signature_counters(current)
    baseline_counts = signature_counters(baseline)
    additions: list[tuple[str, tuple[str, str, str], int]] = []
    for current_path, counter in current_counts.items():
        prior_path = renames.get(current_path, current_path)
        for signature, count in (counter - baseline_counts.get(prior_path, collections.Counter())).items():
            additions.append((current_path, signature, count))
    return sorted(additions)


def summary(sources: dict[str, str], sites: Sequence[ReaderSite]) -> dict[str, object]:
    file_categories: dict[str, set[str]] = collections.defaultdict(set)
    site_counts = collections.Counter(site.category for site in sites)
    for site in sites:
        file_categories[site.test_path].add(site.category)
    categories = [
        "app-source-audit",
        "unresolved-reader",
        "fixture-golden-reader",
        "docs-policy-reader",
        "generated-runtime-artifact-reader",
        "other-repo-artifact-reader",
        "meta-ratchet",
    ]
    return {
        "rust_files": len(sources),
        "reader_files": len(file_categories),
        "reader_sites": len(sites),
        "file_counts": {kind: sum(kind in values for values in file_categories.values()) for kind in categories},
        "site_counts": {kind: site_counts[kind] for kind in categories},
    }


def print_summary(data: dict[str, object]) -> None:
    print(f"Rust test files: {data['rust_files']}")
    print(f"Files invoking reader APIs: {data['reader_files']}")
    print(f"Reader sites: {data['reader_sites']}")
    file_counts = data["file_counts"]
    site_counts = data["site_counts"]
    assert isinstance(file_counts, dict) and isinstance(site_counts, dict)
    for kind in file_counts:
        print(f"{kind}: files={file_counts[kind]} sites={site_counts[kind]}")


def render_markdown(data: dict[str, object]) -> str:
    file_counts = data["file_counts"]
    site_counts = data["site_counts"]
    assert isinstance(file_counts, dict) and isinstance(site_counts, dict)
    meanings = {
        "app-source-audit": "Reads or walks `src/**`; guarded against additions",
        "unresolved-reader": "Dynamic target the scanner cannot prove non-app; guarded conservatively",
        "fixture-golden-reader": "Reads checked-in fixtures or golden cases",
        "docs-policy-reader": "Reads Markdown, skills, or policy documentation",
        "generated-runtime-artifact-reader": "Reads test/generated/runtime output",
        "other-repo-artifact-reader": "Reads scripts, workflows, assets, or other non-app files",
        "meta-ratchet": "Reads test source for the existing occurrence-count ratchet",
    }
    rows = "\n".join(
        f"| `{kind}` | {file_counts[kind]} | {site_counts[kind]} | {meaning} |"
        for kind, meaning in meanings.items()
    )
    return f"""# Source-audit reader inventory

Generated by:

```sh
python3 -B tests/source_audit_inventory.py --write
```

The inventory is site-based. A Rust test file can appear in several classes,
so category file counts are not expected to sum to the reader-file total.
Comments and Rust source embedded inside fixture strings are lexically excluded.

| Class | Files | Read sites | Meaning |
|---|---:|---:|---|
{rows}
| **All readers** | **{data['reader_files']}** | **{data['reader_sites']}** | Of {data['rust_files']} Rust files under `tests/` |

On pull requests, the checker scans both the working tree and the exact base
tree with the same scanner. It compares per-file multisets of normalized
`(reader, target, category)` signatures. Adding a source read inside an already
grandfathered audit therefore fails, and deleting one read cannot hide adding a
different target. Git rename detection maps a renamed file back to its base
identity before comparison.

`unresolved-reader` is conservative: a new dynamic reader is rejected unless
the expression is made resolvable or the policy tool receives a reviewed,
narrow exception. The lexer resolves cooked/raw strings, multiline delimiters,
`concat!`/`env!`, `Path`/`PathBuf` joins, simple bindings and scan-root arrays,
and local wrapper functions. It is not a complete Rust type checker or
cross-module data-flow engine.
"""


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", help="Git revision used as the grandfathered site baseline")
    parser.add_argument("--list", action="store_true", help="Print tab-separated read sites")
    parser.add_argument("--write", action="store_true", help="Regenerate the Markdown inventory")
    parser.add_argument("--check", action="store_true", help="Fail when the Markdown inventory is stale")
    args = parser.parse_args()

    sources = working_tree_sources()
    current = inventory(sources)
    data = summary(sources, current)
    print_summary(data)
    if args.list:
        for site in current:
            print(
                f"{site.category}\t{site.test_path}\t{site.line}\t"
                f"{site.reader}\t{site.target}"
            )

    if args.write:
        INVENTORY_DOC.write_text(render_markdown(data), encoding="utf-8")
        print(f"Wrote {INVENTORY_DOC.relative_to(REPO_ROOT)}.")

    if args.check:
        expected = render_markdown(data)
        actual = INVENTORY_DOC.read_text(encoding="utf-8")
        if actual != expected:
            print(
                "tests/source_audit_inventory.md is stale; regenerate it from "
                "render_markdown(summary(...))",
                file=sys.stderr,
            )
            return 1

    if args.base:
        baseline = inventory(git_sources(args.base))
        additions = added_guarded_sites(current, baseline, git_renames(args.base))
        if additions:
            print(
                "new app-source or unresolved reader sites are not allowed; move the "
                "invariant up the enforcement ladder:",
                file=sys.stderr,
            )
            for path, (reader, target, category), count in additions:
                suffix = f" x{count}" if count > 1 else ""
                print(f"  {path}: {reader} -> {target} [{category}]{suffix}", file=sys.stderr)
            return 1
        print(f"No new guarded reader sites relative to {args.base}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
