#!/usr/bin/env python3
"""check-frames.py <frames dir> [--json out.json]

Grades every step a one-window-frames.ts run wrote against the one-window targets and prints a
markdown table. Exit 1 when any hard target fails.

Hard targets per step: the input never moves (inputDy 0), the line above the input is present and
34 pt and nothing else is drawn above it, a 28 pt header leads whenever rows show and one of them is highlighted, rows are 44 pt,
the selected row has no fill (its color is the backdrop's, read from the strip above it), no left bar, no row keycaps, the focused tile is 24 pt, no popup
window opened, and the input draws all of its text (ink reaches at least 4.5 pt per character, up to
the input's width). The ↵ label sequence is flagged when it returns to an earlier label while the text
only grew (A → B → A).

--expect FILE holds what each step showed when its frames were last reviewed: the first header, the ↵
label and the open view. A step that shows something else fails as changed, so a chapter that takes a
different path (a row that stopped leading, a view that didn't open) fails even when every pixel
target holds. --bless FILE writes this run's values as the new expectations.
"""
import json, os, sys

TILE = (251, 191, 36)
TILE_ALPHA = 0.95
# A view's own content (a form field, an answer, a chat turn) is not a list: no header leads it and
# nothing in it is the selected row. Views that are lists (history, learned, shortcuts) keep the list rules.
VIEW_CONTENT = ("answer:", "form-field:", "note-line:", "chat:", "chat-next:", "step:", "connect:",
                "connect-can:", "agent:")


def in_view_content(m):
    return str((m.get("firstRow") or {}).get("semanticId") or "").startswith(VIEW_CONTENT)


def tile_target(backdrop):
    """The tile's color as read back: #FBBF24 at 95% over what the window draws behind the list.
    With native glass the backdrop reads clear and the tile reads as #FBBF24 itself."""
    if not backdrop or backdrop[3] == 0:
        return TILE
    return tuple(round(t * TILE_ALPHA + b * (1 - TILE_ALPHA)) for t, b in zip(TILE, backdrop[:3]))


def near_tile(rgba, backdrop=None):
    return bool(rgba) and all(abs(a - b) <= 12 for a, b in zip(rgba[:3], tile_target(backdrop)))


def rgba_image(d, m):
    path = os.path.join(d, m.get("_file", "")[:-5] + ".rgba.png")
    if not os.path.exists(path):
        return None
    try:
        from PIL import Image
    except ImportError:
        return None
    return Image.open(path).convert("RGBA")


def modal(pixels):
    """Mean of the pixels in the most common 8-level bucket: the bucket picks the color, the mean
    keeps its value unbiased."""
    buckets = {}
    for c in pixels:
        buckets.setdefault(tuple(v // 8 for v in c), []).append(c)
    if not buckets:
        return None
    top = max(buckets.values(), key=len)
    return [round(sum(c[i] for c in top) / len(top)) for i in range(len(top[0]))]


def row_band(im, m, y0, y1):
    """The common color of a horizontal strip across the right half of the list, where rows draw
    no text: a row's fill, or the backdrop."""
    k = m.get("scale") or 1
    w = m.get("windowWidth") or im.size[0] / k
    return modal([im.getpixel((int(x * k), int(y * k)))
                  for y in range(int(y0), int(y1)) for x in range(int(w * 0.55), int(w * 0.95), 3)])


def row_fill(im, m):
    """(selected row, backdrop) colors: the selected row's strip and the strip just above it, which
    is the header or another row. A fill is the selected strip differing from its backdrop."""
    row = m.get("selectedRow")
    if not im or not row:
        return None, None
    return (row_band(im, m, row["y"] + 8, row["y"] + row["h"] - 8),
            row_band(im, m, row["y"] - 14, row["y"] - 4))


def tile_band(im, m):
    """The common color in a 2 pt band just inside the tile's edge; the glyph owns the center."""
    icon = m.get("selectedIcon")
    if not icon or not im:
        return icon and icon.get("centerRgba")
    k = m.get("scale") or 1
    x, y, w, h = icon["x"], icon["y"], icon["w"], icon["h"]
    pixels = []
    for t in range(int(w)):
        for px, py in ((x + t, y + 3), (x + t, y + h - 3), (x + 3, y + t), (x + w - 3, y + t)):
            if x + 6 <= px <= x + w - 6 or y + 6 <= py <= y + h - 6:
                pixels.append(im.getpixel((int(px * k), int(py * k)))[:3])
    return modal(pixels) or icon.get("centerRgba")


def input_reach(d, m):
    """How far right the input's text is drawn, in pt from where text starts; None without pixels."""
    i = m.get("input")
    path = os.path.join(d, m.get("_file", "")[:-5] + ".png")
    if not i or not os.path.exists(path):
        return None
    try:
        from PIL import Image
    except ImportError:
        return None
    im = Image.open(path).convert("RGB")
    k = m.get("scale") or 1
    x0, y0 = int(i["x"] * k), int(i["y"] * k)
    x1, y1 = int((i["x"] + i["w"]) * k), int((i["y"] + i["h"]) * k)
    bg = im.getpixel((x1 - 4, (y0 + y1) // 2))
    first = last = None
    for x in range(x0, x1):
        if any(sum(abs(a - b) for a, b in zip(im.getpixel((x, y)), bg)) > 90 for y in range(y0, y1)):
            first = x if first is None else first
            last = x
    return None if first is None else (last - first) / k


def view_name(m):
    """The view on top, from the host's own report; None on the list."""
    path = os.path.join(m.get("_dir", ""), m.get("_file", "")[:-5] + ".capture.json")
    if not os.path.exists(path):
        return None
    return ((json.load(open(path)).get("state") or {}).get("oneWindow") or {}).get("view")


def slot_words(m):
    """What the line above the input says, "LABEL · title", from the host's slot-line element."""
    path = os.path.join(m.get("_dir", ""), m.get("_file", "")[:-5] + ".capture.json")
    if not os.path.exists(path):
        return None
    elements = ((json.load(open(path)).get("elements") or {}).get("elements")) or []
    el = next((e for e in elements if e.get("semanticId") == "main-view-slot-line"), None)
    if not el:
        return None
    words = " · ".join(w for w in (el.get("value"), el.get("text")) if w)
    return words or None


def enter_label(m):
    for s in m.get("footerSemantics") or []:
        if s.startswith("footer-action:run="):
            return s.split("=", 1)[1]
    return None


def main():
    d = sys.argv[1]
    out_json = sys.argv[sys.argv.index("--json") + 1] if "--json" in sys.argv else None
    # --known NAME: an element another lane retires; reported, not failed (never used on a final run)
    known = {sys.argv[i + 1] for i, a in enumerate(sys.argv) if a == "--known" and i + 1 < len(sys.argv)}
    expect_path = sys.argv[sys.argv.index("--expect") + 1] if "--expect" in sys.argv else None
    bless_path = sys.argv[sys.argv.index("--bless") + 1] if "--bless" in sys.argv else None
    expected = json.load(open(expect_path)) if expect_path else {}
    shown = {}
    steps = []
    for f in sorted(os.listdir(d)):
        if f.endswith(".json") and f[:2].isdigit() and f.count(".") == 1:
            m = json.load(open(os.path.join(d, f)))
            m["_file"] = f
            m["_dir"] = d
            steps.append(m)
    if not steps:
        print("no steps in", d)
        sys.exit(1)
    y0 = steps[0]["input"]["y"]
    rows, hard = [], 0
    labels, prev_text = [], ""
    for m in steps:
        name = m["step"]["name"]
        fails, notes = [], []
        dy = round(m["input"]["y"] - y0, 2)
        if dy != 0:
            fails.append(f"inputDy {dy}")
        above = m.get("aboveInputAll") or []
        slot = next((a for a in above if a["name"] == "main-view-slot-line"), None)
        others = [a["name"] for a in above if not a["name"].startswith("main-view-slot-line")]
        if not slot:
            fails.append("no slot line")
        elif abs(slot["h"] - 34) > 0.5:
            fails.append(f"slot {slot['h']}")
        known_above = [n for n in others if n in known]
        others = [n for n in others if n not in known]
        if others:
            fails.append("above input: " + ",".join(others))
        if known_above:
            notes.append("known above input: " + ",".join(known_above))
        fh, fr = m.get("firstHeader"), m.get("firstRow")
        view = in_view_content(m)
        if view:
            notes.append("view " + str(fr.get("semanticId")).split(":")[0])
        if fr and not view and (not fh or fh["y"] > fr["y"]):
            fails.append("rows without a leading header")
        if fr and not view and not m.get("selectedRow"):
            fails.append("rows with none highlighted")
        if fh and abs(fh["h"] - 28) > 0.5:
            fails.append(f"header {fh['h']}")
        if fr and abs(fr["h"] - 44) > 0.5:
            fails.append(f"row {fr['h']}")
        im = rgba_image(d, m)
        selected_band, backdrop = row_fill(im, m)
        if selected_band and backdrop:
            fill = max(abs(a - b) for a, b in zip(selected_band, backdrop))
            if fill > 3:
                fails.append(f"fill {selected_band} over backdrop {backdrop}")
        else:
            fill = ((m.get("selectedRowFill") or {}).get("rgba") or [None] * 4)[3]
            if fill:
                fails.append(f"fill a={fill}")
        if m.get("leftEdgeBar"):
            fails.append("left bar")
        if m.get("rowKeycaps"):
            fails.append(f"keycaps {m['rowKeycaps']}")
        icon = m.get("selectedIcon")
        band = tile_band(im, m) if icon else None
        if m.get("selectedRow") and icon and not near_tile(band, backdrop):
            fails.append(f"tile color {band}, want {list(tile_target(backdrop))}")
        if m.get("selectedRow") and icon and (abs(icon["w"] - 24) > 0.5 or abs(icon["h"] - 24) > 0.5):
            fails.append(f"tile {icon['w']}x{icon['h']}")
        popups = m.get("popups") or []
        known_popups = [p for p in popups if str(p.get("target", "")).split(":")[0] in known]
        if len(popups) > len(known_popups):
            fails.append(f"popups {len(popups) - len(known_popups)}")
        if known_popups:
            notes.append("known popup: " + ",".join(str(p.get("target")) for p in known_popups))
        issues = [i for i in m.get("issues") or [] if not (view and i == "selected_row_not_painted")]
        if issues:
            fails.append("issues: " + ";".join(map(str, issues))[:80])
        text = m.get("inputValue") or ""
        reach = input_reach(d, m) if text.strip() else None
        want = min(4.5 * len(text), m["input"]["w"] - 40)
        if reach is not None and reach < want:
            fails.append(f"input text cut at {reach:.0f} pt of {len(text)} chars")
        label = enter_label(m)
        line = slot_words(m)
        now = {"header": fh and fh.get("text"), "enter": label, "view": view_name(m)}
        shown[name] = now
        for k, want in (expected.get(name) or {}).items():
            if now.get(k) != want:
                fails.append(f"changed {k}: “{want}” → “{now.get(k)}”")
        if expect_path and name not in expected:
            notes.append("no expectation")
        if prev_text and text.startswith(prev_text) and len(text) > len(prev_text):
            if label in [l for l in labels[:-1] if l != labels[-1]]:
                fails.append(f"↵ returned to “{label}”")
        elif not text.startswith(prev_text):
            labels = []
        labels.append(label)
        prev_text = text
        hard += bool(fails)
        rows.append({
            "step": name, "text": text, "inputY": m["input"]["y"], "inputDy": dy,
            "slot": slot and slot["h"], "header": fh and fh["h"], "headerText": fh and fh.get("text"),
            "row": fr and fr["h"], "fill": fill, "bar": bool(m.get("leftEdgeBar")),
            "keycaps": m.get("rowKeycaps"), "tile": icon and f"{icon['w']:g}", "tileRgb": band,
            "tileYellow": bool(icon and near_tile(band, backdrop)), "backdrop": backdrop, "enter": label,
            "windowH": m.get("windowHeight"), "popups": len(m.get("popups") or []), "fails": fails,
            "notes": notes, "line": line,
        })
    print("| step | text | inputDy | slot | line | header | row | fill α | bar | keycaps | tile | ↵ | window h | result |")
    print("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|")
    for r in rows:
        tile = f"{r['tile']} {'yellow' if r['tileYellow'] else r['tileRgb']}" if r["tile"] else "—"
        head = f"{r['header']:g} {r['headerText'] or ''}".strip() if r["header"] else "—"
        print(f"| {r['step']} | {r['text'][:28]} | {r['inputDy']:g} | {r['slot'] or '—'} | {(r['line'] or '—')[:32]} | {head[:34]} | {r['row'] or '—'} | "
              f"{r['fill'] if r['fill'] is not None else '—'} | {'yes' if r['bar'] else 'no'} | {r['keycaps']} | {tile} | "
              f"{(r['enter'] or '—')[:40]} | {r['windowH']} | {'PASS' if not r['fails'] else 'FAIL: ' + '; '.join(r['fails'])}{' (' + '; '.join(r['notes']) + ')' if r['notes'] else ''} |")
    print(f"\n{len(rows) - hard} of {len(rows)} steps meet every target")
    if out_json:
        json.dump({"steps": rows, "pass": hard == 0}, open(out_json, "w"), indent=1, ensure_ascii=False)
    if bless_path:
        json.dump(shown, open(bless_path, "w"), indent=1, ensure_ascii=False)
        print(f"blessed {len(shown)} steps into {bless_path}")
    sys.exit(1 if hard else 0)


if __name__ == "__main__":
    main()
