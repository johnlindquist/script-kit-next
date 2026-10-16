#!/usr/bin/env python3
"""Build src/emoji/emoji_data.tsv from Unicode's public emoji data.

Inputs (download them first; they are not checked in):
  emoji-test.txt   https://unicode.org/Public/emoji/latest/emoji-test.txt
  annotations.json https://raw.githubusercontent.com/unicode-org/cldr-json/main/cldr-json/cldr-annotations-full/annotations/en/annotations.json
  derived.json     https://raw.githubusercontent.com/unicode-org/cldr-json/main/cldr-json/cldr-annotations-derived-full/annotationsDerived/en/annotations.json

Usage:
  python3 scripts/emoji/generate_emoji_data.py emoji-test.txt annotations.json derived.json

Output columns (tab separated, one emoji per line, Unicode order):
  emoji, name, group, emoji version, five skin-tone variants (space separated, or empty),
  search keywords (| separated)

Skin-tone variants are recorded only when the sequence has exactly one variant per tone
(light to dark) that uses that single tone throughout. Extra Script Kit keywords from
src/emoji/extra_keywords.tsv are appended to the CLDR keywords.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "src/emoji/emoji_data.tsv"
EXTRA = ROOT / "src/emoji/extra_keywords.tsv"

TONES = ["\U0001F3FB", "\U0001F3FC", "\U0001F3FD", "\U0001F3FE", "\U0001F3FF"]
GROUPS = {
    "Smileys & Emotion": "SmileysEmotion",
    "People & Body": "PeopleBody",
    "Animals & Nature": "AnimalsNature",
    "Food & Drink": "FoodDrink",
    "Travel & Places": "TravelPlaces",
    "Activities": "Activities",
    "Objects": "Objects",
    "Symbols": "Symbols",
    "Flags": "Flags",
}
LINE = re.compile(r"^([0-9A-F ]+?)\s*;\s*fully-qualified\s*#\s*(\S+)\s+E(\d+\.\d+)\s+(.+)$")


def strip(sequence: str) -> str:
    return "".join(ch for ch in sequence if ch not in TONES and ch != "\ufe0f")


def main() -> int:
    if len(sys.argv) != 4:
        print(__doc__)
        return 2
    test_path, ann_path, derived_path = (Path(arg) for arg in sys.argv[1:])
    annotations = json.loads(ann_path.read_text())["annotations"]["annotations"]
    derived = json.loads(derived_path.read_text())["annotationsDerived"]["annotations"]

    extra: dict[str, list[str]] = {}
    if EXTRA.exists():
        for line in EXTRA.read_text().splitlines():
            if line.strip() and not line.startswith("#"):
                emoji, words = line.split("\t", 1)
                extra[emoji] = [word.lower() for word in words.split("|") if word]

    group = None
    bases: list[dict] = []
    toned: dict[str, dict[str, str]] = {}
    for raw in test_path.read_text().splitlines():
        if raw.startswith("# group:"):
            group = GROUPS.get(raw.split(":", 1)[1].strip())
            continue
        match = LINE.match(raw)
        if not match or group is None:
            continue
        emoji, version, name = match.group(2), match.group(3), match.group(4).strip()
        tones_used = {ch for ch in emoji if ch in TONES}
        if tones_used:
            if len(tones_used) == 1:
                toned.setdefault(strip(emoji), {}).setdefault(tones_used.pop(), emoji)
            continue
        bases.append({"emoji": emoji, "name": name, "group": group, "version": version})

    def keywords(emoji: str, name: str) -> list[str]:
        words: list[str] = []
        for key in (emoji, emoji.replace("\ufe0f", "")):
            for table in (annotations, derived):
                if key in table:
                    words = [word.lower() for word in table[key].get("default", [])]
                    break
            if words:
                break
        for word in extra.get(emoji, []):
            if word not in words:
                words.append(word)
        return [word.replace("|", " ").replace("\t", " ") for word in words if word != name.lower()]

    lines = []
    for base in bases:
        variants = toned.get(strip(base["emoji"]), {})
        tones = " ".join(variants[tone] for tone in TONES) if len(variants) == 5 else ""
        lines.append(
            "\t".join(
                [
                    base["emoji"],
                    base["name"],
                    base["group"],
                    base["version"],
                    tones,
                    "|".join(keywords(base["emoji"], base["name"])),
                ]
            )
        )
    OUT.write_text("\n".join(lines) + "\n")
    print(f"wrote {len(lines)} emoji to {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
