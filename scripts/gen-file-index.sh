#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_PATH="${1:-${ROOT_DIR}/.ai/file-index.json}"

mkdir -p "$(dirname "${OUTPUT_PATH}")"

python3 - "${ROOT_DIR}" "${OUTPUT_PATH}" <<'PY'
import json
import re
import sys
from collections import defaultdict
from pathlib import Path
from typing import Dict, List, Sequence, Set, Tuple

root_dir = Path(sys.argv[1])
output_path = Path(sys.argv[2])

src_root = root_dir / "src"
test_root = root_dir / "tests"

source_files = sorted(p for p in src_root.rglob("*.rs") if p.is_file())
test_files = []
if test_root.exists():
    test_files = sorted(
        p.relative_to(root_dir).as_posix()
        for p in test_root.rglob("*")
        if p.is_file() and p.suffix in {".rs", ".ts", ".js", ".sh"}
    )

PUBLIC_TYPE_RE = re.compile(
    r"(?m)^\s*pub\s+(?:unsafe\s+)?(?:struct|enum|trait)\s+([A-Za-z_][A-Za-z0-9_]*)\b"
)
USE_RE = re.compile(r"(?ms)^\s*use\s+([^;]+);")
MOD_RE = re.compile(r"(?m)^\s*(?:pub\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;")


def rel(path: Path) -> str:
    return path.relative_to(root_dir).as_posix()


def module_path_for_file(rel_path: str) -> Tuple[str, ...]:
    parts = Path(rel_path).parts
    if not parts or parts[0] != "src":
        return tuple()

    tail = list(parts[1:])
    if len(tail) == 1:
        if tail[0] in {"lib.rs", "main.rs"}:
            return tuple()
        stem = tail[0][:-3] if tail[0].endswith(".rs") else tail[0]
        if stem == "mod":
            return tuple()
        return (stem,)

    if tail[-1] == "mod.rs":
        return tuple(tail[:-1])
    return tuple(tail[:-1] + [tail[-1][:-3]])


def top_level_module(rel_path: str) -> str:
    parts = Path(rel_path).parts
    tail = list(parts[1:])
    if len(tail) == 1:
        if tail[0] in {"lib.rs", "main.rs"}:
            return "root"
        return tail[0][:-3]
    return tail[0]


def extract_description(source: str, rel_path: str, module: str) -> str:
    lines = source.splitlines()
    docs: List[str] = []
    i = 0

    while i < len(lines):
        line = lines[i].strip()
        if not line:
            if docs:
                break
            i += 1
            continue
        if line.startswith("#!["):
            i += 1
            continue
        if line.startswith("//!"):
            docs.append(line[3:].strip())
            i += 1
            while i < len(lines):
                next_line = lines[i].strip()
                if next_line.startswith("//!"):
                    docs.append(next_line[3:].strip())
                    i += 1
                    continue
                if not next_line:
                    docs.append("")
                    i += 1
                    continue
                break
            break
        break

    if docs:
        first_line = next((line.strip() for line in docs if line.strip()), "")
        if first_line:
            cleaned = first_line.strip(" -")
            cleaned = cleaned.lstrip("#").strip()
            cleaned = cleaned.replace("`", "")
            cleaned = re.sub(r"\s+", " ", cleaned)
            if cleaned and cleaned[-1] not in ".!?":
                cleaned += "."
            if cleaned:
                return cleaned

    file_name = Path(rel_path).name
    stem = Path(rel_path).stem.replace("_", " ")
    if file_name == "mod.rs":
        parent = Path(rel_path).parent.name or "root"
        return f"Defines the {parent} module and wires related submodules."
    if module == "root":
        return f"Implements root-level {stem} functionality."
    return f"Implements {stem} logic for the {module} module."


def add_resolved_target(
    base_segments: Tuple[str, ...],
    suffix_segments: Sequence[str],
    module_primary: Dict[Tuple[str, ...], str],
    out: Set[str],
) -> None:
    if not suffix_segments:
        return

    full_segments = base_segments + tuple(suffix_segments)
    for size in range(len(full_segments), 0, -1):
        probe = full_segments[:size]
        if probe in module_primary:
            out.add(module_primary[probe])
            return


def split_top_level_commas(value: str) -> List[str]:
    items: List[str] = []
    depth = 0
    start = 0
    for index, char in enumerate(value):
        if char == "{":
            depth += 1
        elif char == "}":
            depth = max(0, depth - 1)
        elif char == "," and depth == 0:
            piece = value[start:index].strip()
            if piece:
                items.append(piece)
            start = index + 1
    tail = value[start:].strip()
    if tail:
        items.append(tail)
    return items


def expand_use_tree(use_expr: str) -> List[str]:
    queue = [use_expr.strip()]
    expanded: List[str] = []

    while queue:
        current = queue.pop()
        if not current:
            continue

        open_idx = current.find("{")
        if open_idx == -1:
            expanded.append(current)
            continue

        depth = 0
        close_idx = -1
        for index in range(open_idx, len(current)):
            char = current[index]
            if char == "{":
                depth += 1
            elif char == "}":
                depth -= 1
                if depth == 0:
                    close_idx = index
                    break

        if close_idx == -1:
            expanded.append(current.replace("{", "").replace("}", ""))
            continue

        prefix = current[:open_idx]
        inner = current[open_idx + 1 : close_idx]
        suffix = current[close_idx + 1 :]

        for item in split_top_level_commas(inner):
            candidate = f"{prefix}{item}{suffix}".strip()
            if candidate:
                queue.append(candidate)

    return expanded


all_rel_sources = [rel(path) for path in source_files]
module_candidates: Dict[Tuple[str, ...], List[str]] = defaultdict(list)
module_path_by_file: Dict[str, Tuple[str, ...]] = {}

for rel_path in all_rel_sources:
    module_path = module_path_for_file(rel_path)
    module_path_by_file[rel_path] = module_path
    module_candidates[module_path].append(rel_path)

module_primary: Dict[Tuple[str, ...], str] = {}
for module_path, candidates in module_candidates.items():
    module_primary[module_path] = sorted(
        candidates, key=lambda p: (0 if p.endswith("/mod.rs") else 1, p)
    )[0]

imports_by_file: Dict[str, Set[str]] = {path: set() for path in all_rel_sources}
public_types_by_file: Dict[str, List[str]] = {}
description_by_file: Dict[str, str] = {}


def is_internal_test_source(rel_path: str) -> bool:
    if not rel_path.startswith("src/"):
        return False
    path_obj = Path(rel_path)
    name = path_obj.name
    stem = path_obj.stem
    return (
        name == "tests.rs"
        or name.endswith("_tests.rs")
        or "_tests_" in name
        or stem.startswith("test_")
        or "/tests/" in rel_path
    )

for rel_path in all_rel_sources:
    file_path = root_dir / rel_path
    source = file_path.read_text(encoding="utf-8")
    module_path = module_path_by_file[rel_path]
    module_name = top_level_module(rel_path)

    public_types = sorted(set(PUBLIC_TYPE_RE.findall(source)))
    public_types_by_file[rel_path] = public_types
    description_by_file[rel_path] = extract_description(source, rel_path, module_name)

    direct_imports: Set[str] = set()

    for use_block in USE_RE.findall(source):
        compact = " ".join(use_block.split())
        for expanded in expand_use_tree(compact):
            cleaned = expanded.split(" as ", 1)[0].strip()
            cleaned = cleaned.replace(" ", "")
            if cleaned.endswith("::*"):
                cleaned = cleaned[:-3]
            if not cleaned:
                continue

            if cleaned.startswith("crate::"):
                suffix = tuple(part for part in cleaned[7:].split("::") if part and part != "*")
                add_resolved_target(tuple(), suffix, module_primary, direct_imports)
            elif cleaned.startswith("super::"):
                suffix = tuple(part for part in cleaned[7:].split("::") if part and part != "*")
                add_resolved_target(module_path[:-1], suffix, module_primary, direct_imports)
            elif cleaned.startswith("self::"):
                suffix = tuple(part for part in cleaned[6:].split("::") if part and part != "*")
                add_resolved_target(module_path, suffix, module_primary, direct_imports)

    for mod_name in MOD_RE.findall(source):
        target_segments = module_path + (mod_name,)
        if target_segments in module_primary:
            direct_imports.add(module_primary[target_segments])

    imports_by_file[rel_path] = {item for item in direct_imports if item != rel_path}

imported_by_file: Dict[str, Set[str]] = {path: set() for path in all_rel_sources}
for source_path, targets in imports_by_file.items():
    for target in targets:
        imported_by_file[target].add(source_path)


def associated_test_files(rel_path: str) -> List[str]:
    module_path = module_path_by_file[rel_path]
    path_obj = Path(rel_path)

    tokens: Set[str] = set()
    if path_obj.name == "mod.rs":
        if module_path:
            tokens.add(module_path[-1].lower())
        if path_obj.parent.name:
            tokens.add(path_obj.parent.name.lower())
    else:
        tokens.add(path_obj.stem.lower())

    if module_path:
        joined_dash = "-".join(module_path).lower()
        joined_underscore = "_".join(module_path).lower()
        if joined_dash:
            tokens.add(joined_dash)
        if joined_underscore:
            tokens.add(joined_underscore)

    result: Set[str] = set()
    for test_path in test_files:
        test_stem = Path(test_path).stem.lower()
        for token in tokens:
            if not token:
                continue
            if len(token) < 3 and token not in {"ai", "ui"}:
                continue
            if (
                test_stem == token
                or test_stem.startswith(f"test-{token}")
                or test_stem.startswith(f"test_{token}")
                or test_stem.endswith(f"-{token}")
                or test_stem.endswith(f"_{token}")
                or f"-{token}-" in test_stem
                or f"_{token}_" in test_stem
            ):
                result.add(test_path)
                break

    path_obj = Path(rel_path)
    file_stem = path_obj.stem
    if path_obj.suffix == ".rs":
        sibling_candidates = {
            path_obj.with_name(f"{file_stem}_tests.rs").as_posix(),
            path_obj.with_name(f"{file_stem}_test.rs").as_posix(),
            path_obj.with_name("tests.rs").as_posix(),
        }
        if path_obj.name == "mod.rs" and path_obj.parent.name:
            sibling_candidates.add(
                path_obj.with_name(f"{path_obj.parent.name}_tests.rs").as_posix()
            )
        for candidate in sibling_candidates:
            if candidate in all_rel_sources and is_internal_test_source(candidate):
                result.add(candidate)

    internal_couplings = imports_by_file[rel_path] | imported_by_file[rel_path]
    for candidate in internal_couplings:
        if is_internal_test_source(candidate):
            result.add(candidate)

    return sorted(result)


entries = []
for rel_path in sorted(all_rel_sources):
    coupled = sorted(
        candidate
        for candidate in (imports_by_file[rel_path] | imported_by_file[rel_path]) - {rel_path}
        if not is_internal_test_source(candidate)
    )
    entries.append(
        {
            "path": rel_path,
            "module": top_level_module(rel_path),
            "description": description_by_file[rel_path],
            "public_types": public_types_by_file[rel_path],
            "test_files": associated_test_files(rel_path),
            "coupled_with": coupled,
        }
    )

payload = {
    "schema_version": 1,
    "source_root": "src",
    "files": entries,
}

output_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
PY

echo "Generated ${OUTPUT_PATH}"
