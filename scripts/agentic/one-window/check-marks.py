#!/usr/bin/env python3
"""check-marks.py <frames dir> [--expect FILE.marks.json] [--json out.json] [--evidence DIR]

Reads the word marks under the main input in every frame a one-window-frames.ts run wrote: a solid
bar in the word's color under each marked phrase, while the text itself stays white (S04). Prints a
markdown table and exits 1 when a step misses what it expects or breaks a global rule.

Global rules, every frame: the input's text is not recolored (saturated ink under 3% of the glyphs),
and every bar is 1-4.5 pt thick with its top 4-9.5 pt under the text's baseline.

Every frame's bars are held to the marks the app handed the input in the same capture
(state.oneWindow.marks in <stem>.capture.json), exactly: each bar sits under its text (covers 85% of
it, reaches no more than 2 pt past either end) in its word's color, and no other bar shows.

--expect is the spec: per step, the marks the sentence should get, left to right:
{"steps": {"<step>": {"marks": [{"word": "action", "text": "send"}], "exact": true}}}. The app's
marks must include each, in order; with "exact" (the default) nothing else. Without a recorded app
decision the spec is checked against the bars instead.

The Owner pill ⇥ puts ahead of the words (S04) shows exactly when the app says the agent owns the
sentence (state.oneWindow.agentOwns), inside the input's row, left of the text, in the Owner's
color; a step's "owner": true/false in the spec says which it must be. Text is read right of it,
and beside the subject chip: right of it once attached, left of it while offered at the input's end.

--evidence DIR writes a close-up of the input row per step with each bar boxed and named, a contact
sheet of all of them, and report.md.
"""
import json, os, sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

WORDS = {
    "subject": (0xFF, 0xB8, 0x6B),
    "action": (0xF5, 0xC5, 0x42),
    "target": (0x6F, 0xDC, 0x9B),
    "details": (0x8F, 0xB8, 0xFF),
    "owner": (0xC9, 0xA7, 0xFF),
    "focused": (0xFF, 0x9E, 0xCB),
}
NAMES = list(WORDS)
PALETTE = np.array([WORDS[n] for n in NAMES], dtype=float)
COLOR_TOLERANCE = 58
MIN_SATURATION = 40
INK = 110
WORD_GAP_PT = 3.5
MERGE_GAP_PT = 1.0
RECOLOR_LIMIT = 0.03


def load_frames(d):
    frames = []
    for name in sorted(os.listdir(d)):
        if not name.endswith(".json") or name.endswith(".capture.json"):
            continue
        m = json.load(open(os.path.join(d, name)))
        if "step" not in m or not m.get("input"):
            continue
        png = os.path.join(d, name[:-5] + ".png")
        if os.path.exists(png):
            m["_png"], m["_stem"] = png, name[:-5]
            m["_app_marks"] = app_marks(os.path.join(d, name[:-5] + ".capture.json"))
            m["_agent_owns"] = app_owns(os.path.join(d, name[:-5] + ".capture.json"))
            capture = os.path.join(d, name[:-5] + ".capture.json")
            if os.path.exists(capture):
                state = (json.load(open(capture)).get("state") or {}).get("oneWindow") or {}
                m["_chips"] = [chip["token"] for chip in state.get("context", [])]
                # Mark ranges index the editable words, not the composed sentence with
                # invisible @tokens appended for intent routing.
                if "inputWords" in state:
                    m["inputValue"] = state["inputWords"]
            frames.append(m)
    return frames


def app_marks(path):
    """The marks the app handed the input when the frame was captured, left to right."""
    if not os.path.exists(path):
        return None
    marks = ((json.load(open(path)).get("state") or {}).get("oneWindow") or {}).get("marks")
    if marks is None:
        return None
    return sorted(({"word": m["word"], "text": m["text"], "range": m["range"]} for m in marks),
                  key=lambda m: m["range"][0])


def app_owns(path):
    """Whether the app said ⇥ made the agent the Owner when the frame was captured."""
    if not os.path.exists(path):
        return None
    return ((json.load(open(path)).get("state") or {}).get("oneWindow") or {}).get("agentOwns")


def owner_fails(img, m, s, want):
    """Where the Owner pill differs from what the app and the spec say it should be."""
    pill, inp, fails = m.get("ownerPill"), m["input"], []
    for who, owns in (("app", m.get("_agent_owns")), ("spec", want)):
        if owns is not None and bool(pill) != bool(owns):
            state = "missing" if owns else "drawn"
            says = "owns" if owns else "doesn't own"
            fails.append(f"owner pill {state} ({who} says the agent {says} it)")
    if not pill:
        return fails
    if pill["y"] < inp["y"] - 0.5 or pill["y"] + pill["h"] > inp["y"] + inp["h"] + 0.5:
        fails.append("owner pill leaves the input's row")
    box = img[int(pill["y"] * s):int((pill["y"] + pill["h"]) * s), int(pill["x"] * s):int((pill["x"] + pill["w"]) * s)]
    near = np.linalg.norm(box.astype(float) - np.array(WORDS["owner"], dtype=float), axis=2) <= COLOR_TOLERANCE
    if near.sum() < 6 * s * s:
        fails.append("owner pill isn't in the Owner's color")
    return fails


def words_only(m):
    """The frame with its input narrowed to the typed words: past the Owner pill or the chips
    attached at its start, and short of a subject chip offered at its end."""
    inp = dict(m["input"])
    end = inp["x"] + inp["w"]
    chip = m.get("subjectChip")
    for lead in (m.get("ownerPill"), chip):
        if lead and lead["x"] + lead["w"] / 2 < inp["x"] + inp["w"] / 2:
            inp["x"] = max(inp["x"], lead["x"] + lead["w"] + 1)
    # The attached chips always lead the words, however wide they are.
    if m.get("inputChips"):
        inp["x"] = max(inp["x"], m["inputChips"]["x"] + m["inputChips"]["w"] + 1)
    if chip and chip["x"] + chip["w"] / 2 >= m["input"]["x"] + m["input"]["w"] / 2:
        end = min(end, chip["x"] - 1)
    inp["w"] = max(0, end - inp["x"])
    return {**m, "input": inp}


def clusters(columns, gap):
    out, start, last = [], None, None
    for x in np.flatnonzero(columns):
        if start is None:
            start = last = x
        elif x - last > gap:
            out.append([start, last + 1])
            start = last = x
        else:
            last = x
    if start is not None:
        out.append([start, last + 1])
    return out


def text_geometry(img, m, s):
    """Baseline row and one x-range per typed word, in device pixels."""
    inp = m["input"]
    y0, y1 = int(inp["y"] * s), int((inp["y"] + inp["h"]) * s)
    x0, x1 = int(inp["x"] * s), int((inp["x"] + inp["w"]) * s)
    band = img[y0:y1, x0:x1]
    ink = (band.max(axis=2) > INK) & ((band.max(axis=2) - band.min(axis=2)) <= 60)
    rows = ink.sum(axis=1)
    if rows.max() == 0:
        return None
    dense = np.flatnonzero(rows >= rows.max() * 0.25)
    top, baseline = y0 + dense[0], y0 + dense[-1] + 1
    cols = ink[: baseline - y0 + int(3 * s)].any(axis=0)
    found = clusters(cols, int(WORD_GAP_PT * s))
    found = [c for c in found if c[1] - c[0] > 3 * s / 2]
    words = m.get("inputValue", "").split()
    while len(found) > len(words) >= 1:
        gaps = [found[i + 1][0] - found[i][1] for i in range(len(found) - 1)]
        i = int(np.argmin(gaps))
        found[i:i + 2] = [[found[i][0], found[i + 1][1]]]
    spans = [(x0 + a, x0 + b) for a, b in found]
    return {"top": top, "baseline": baseline, "band": (y0, y1, x0, x1),
            "words": spans if len(spans) == len(words) else None}


def word_offsets(text):
    out, at = [], 0
    for w in text.split():
        at = text.index(w, at)
        out.append((at, at + len(w)))
        at += len(w)
    return out


def expected_span(m, geo, text, byte_range=None):
    value = m.get("inputValue", "")
    if byte_range is not None:
        at = len(value.encode()[:byte_range[0]].decode(errors="ignore"))
        if value[at:at + len(text)] != text:
            at = -1
    else:
        at = value.find(text)
    if at < 0 or not geo["words"]:
        return None
    end = at + len(text)
    xs = []
    for (a, b), (gx0, gx1) in zip(word_offsets(value), geo["words"]):
        lo, hi = max(a, at), min(b, end)
        if lo >= hi:
            continue
        per = (gx1 - gx0) / max(1, b - a)
        xs += [gx0 + (lo - a) * per, gx0 + (hi - a) * per]
    return (min(xs), max(xs)) if xs else None


def find_bars(img, m, geo, s):
    inp = m["input"]
    y0 = geo["baseline"] + int(2 * s)
    limit = (inp["y"] + inp["h"] + 8) * s
    header = m.get("firstHeader") or {}
    if header.get("y"):
        limit = min(limit, header["y"] * s)
    y1 = int(limit)
    x0, x1 = int(inp["x"] * s), int((inp["x"] + inp["w"]) * s)
    band = img[y0:y1, x0:x1].astype(float)
    if band.size == 0:
        return []
    dist = np.linalg.norm(band[:, :, None, :] - PALETTE[None, None, :, :], axis=3)
    nearest = dist.argmin(axis=2)
    ok = (dist.min(axis=2) <= COLOR_TOLERANCE) & ((band.max(axis=2) - band.min(axis=2)) >= MIN_SATURATION)
    labels = np.where(ok, nearest, -1)
    col_label = np.full(labels.shape[1], -1)
    for x in range(labels.shape[1]):
        col = labels[:, x]
        col = col[col >= 0]
        if len(col) >= 2:
            col_label[x] = np.bincount(col).argmax()
    bars, gap = [], max(1, int(MERGE_GAP_PT * s))
    x = 0
    while x < len(col_label):
        if col_label[x] < 0:
            x += 1
            continue
        lab, start, last = col_label[x], x, x
        x += 1
        while x < len(col_label):
            if col_label[x] == lab:
                last = x
            elif x - last > gap:
                break
            x += 1
        x = last + 1
        if last - start + 1 < int(2 * s):
            continue
        rows = np.flatnonzero((labels[:, start:last + 1] == lab).any(axis=1))
        bars.append({"word": NAMES[lab], "x0": (x0 + start) / s, "x1": (x0 + last + 1) / s,
                     "y0": (y0 + rows[0]) / s, "y1": (y0 + rows[-1] + 1) / s})
    return bars


def recolor_share(img, geo, s):
    y0, _, x0, x1 = geo["band"]
    band = img[y0:geo["baseline"] + int(s), x0:x1]
    ink = band.max(axis=2) > INK
    if not ink.any():
        return 0.0, None
    sat = ink & ((band.max(axis=2) - band.min(axis=2)) > 60)
    share = sat.sum() / ink.sum()
    dominant = None
    if sat.any():
        dominant = "#%02x%02x%02x" % tuple(int(v) for v in np.median(band[sat], axis=0))
    return float(share), dominant


def text_under(m, geo, bar, s):
    if not geo or not geo["words"]:
        return "?"
    value = m.get("inputValue", "")
    picked = [value[a:b] for (a, b), (gx0, gx1) in zip(word_offsets(value), geo["words"])
              if min(gx1, bar["x1"] * s) - max(gx0, bar["x0"] * s) > (gx1 - gx0) * 0.5]
    return " ".join(picked)


def spec_fails(spec, app):
    """Where the marks the app decided differ from the spec's."""
    fails, at = [], 0
    for w in spec.get("marks", []):
        hit = next((i for i in range(at, len(app)) if (app[i]["word"], app[i]["text"]) == (w["word"], w["text"])), None)
        if hit is None:
            fails.append(f"spec: app doesn't mark “{w['text']}” as {w['word']}")
            continue
        if spec.get("exact", True) and hit != at:
            fails.append("spec: app marks " + ", ".join(f"{a['word']} “{a['text']}”" for a in app[at:hit]) +
                         f" before “{w['text']}”")
        at = hit + 1
    if spec.get("exact", True) and app[at:]:
        fails.append("spec: app also marks " + ", ".join(f"{a['word']} “{a['text']}”" for a in app[at:]))
    return fails


def grade(m, expect):
    s = m.get("scale", 2)
    img = np.asarray(Image.open(m["_png"]).convert("RGB")).astype(int)
    pill_fails = owner_fails(img, m, s, (expect or {}).get("owner"))
    if expect and "chips" in expect:
        if expect["chips"] != m.get("_chips") or bool(expect["chips"]) != bool(m.get("inputChips")):
            pill_fails.append("attached context chips differ from the expected tokens or paint bounds")
    m = words_only(m)
    geo = text_geometry(img, m, s)
    app = m.get("_app_marks")
    out = {"step": m["step"]["name"], "text": m.get("inputValue", ""), "bars": [], "fails": pill_fails, "warns": [],
           "app": app, "spec": expect is not None, "owner": bool(m.get("ownerPill"))}
    if expect is not None and app is not None:
        out["fails"] += spec_fails(expect, app)
    if app is not None:
        expect = {"marks": app, "exact": True}
    if geo is None:
        if expect and expect.get("marks"):
            out["fails"].append("no text drawn")
        return out, img, None
    bars = find_bars(img, m, geo, s)
    for b in bars:
        b["under"] = text_under(m, geo, b, s)
        thick, below = b["y1"] - b["y0"], b["y0"] - geo["baseline"] / s
        if not 1 <= thick <= 4.5:
            out["fails"].append(f"{b['word']} bar is {thick:.1f} pt thick")
        if not 4 <= below <= 9.5:
            out["fails"].append(f"{b['word']} bar top is {below:.1f} pt under the baseline")
    out["bars"] = bars
    share, dominant = recolor_share(img, geo, s)
    out["recolor"] = round(share, 4)
    if share >= RECOLOR_LIMIT:
        out["fails"].append(f"text recolored ({share:.0%} of glyph ink is {dominant})")
    if not geo["words"]:
        out["warns"].append("glyph words don't line up with the typed words; positions unchecked")
    if expect is not None:
        exact = expect.get("exact", True)
        remaining = list(bars)
        for w in expect.get("marks", []):
            match = next((b for b in remaining if b["word"] == w["word"]), None)
            if match is None:
                out["fails"].append(f"missing {w['word']} under “{w['text']}”")
                continue
            if exact and remaining[0] is not match:
                out["fails"].append(f"{w['word']} “{w['text']}” is not next left to right")
            remaining.remove(match)
            span = expected_span(m, geo, w["text"], w.get("range"))
            if span is None:
                if geo["words"]:
                    out["fails"].append(f"“{w['text']}” is not in the input")
                continue
            lo, hi = span[0] / s, span[1] / s
            cover = max(0.0, min(hi, match["x1"]) - max(lo, match["x0"])) / max(0.1, hi - lo)
            if cover < 0.85 or match["x0"] < lo - 2 or match["x1"] > hi + 2:
                out["fails"].append(
                    f"{w['word']} bar {match['x0']:.0f}–{match['x1']:.0f} pt isn't under “{w['text']}” "
                    f"({lo:.0f}–{hi:.0f} pt)")
        if exact and remaining:
            extra = ", ".join(f"{b['word']} under “{b['under']}”" for b in remaining)
            out["fails"].append(f"extra marks: {extra}")
    return out, img, geo


def font(size):
    for path in ("/System/Library/Fonts/SFNS.ttf", "/System/Library/Fonts/Supplemental/Arial.ttf"):
        if os.path.exists(path):
            return ImageFont.truetype(path, size)
    return ImageFont.load_default()


def evidence(m, img, geo, result, out_dir):
    s = m.get("scale", 2)
    top = int(((m.get("aboveInput") or {}).get("y", m["input"]["y"] - 36)) * s)
    header = m.get("firstHeader") or {}
    bottom = int((header["y"] + header["h"]) * s) if header.get("y") else int((m["input"]["y"] + m["input"]["h"] + 40) * s)
    bottom = max(bottom, int((m["input"]["y"] + m["input"]["h"] + 12) * s))
    right = img.shape[1]
    if geo and geo["words"]:
        right = min(img.shape[1], int(max(geo["words"][-1][1] + 60 * s, 420 * s)))
    crop = Image.fromarray(img[top:bottom, :right].astype(np.uint8))
    zoom = 1.5
    crop = crop.resize((int(crop.width * zoom), int(crop.height * zoom)), Image.LANCZOS)
    draw = ImageDraw.Draw(crop)
    for b in result["bars"]:
        box = [b["x0"] * s * zoom - 4, (b["y0"] * s - top) * zoom - 4,
               b["x1"] * s * zoom + 4, (b["y1"] * s - top) * zoom + 4]
        draw.rectangle(box, outline=WORDS[b["word"]], width=1)
    ok = not result["fails"]
    f, small = font(26), font(22)
    strip_h = 56 + 34 * len(result["fails"])
    canvas = Image.new("RGB", (max(crop.width, 900), crop.height + strip_h), (18, 18, 22))
    canvas.paste(crop, (0, 0))
    d = ImageDraw.Draw(canvas)
    y = crop.height + 12
    title = ("PASS" if ok else "FAIL") + f"  {result['step']}"
    d.text((16, y), title, fill=(110, 220, 150) if ok else (255, 110, 110), font=f)
    x = 16 + d.textlength(title, font=f) + 28
    for b in result["bars"]:
        label = f"{b['word']}: “{b['under']}”"
        w = d.textlength(label, font=small)
        d.rounded_rectangle([x - 8, y - 2, x + w + 8, y + 30], radius=8, outline=WORDS[b["word"]], width=2)
        d.text((x, y + 2), label, fill=WORDS[b["word"]], font=small)
        x += w + 28
    if not result["bars"]:
        d.text((x, y + 2), "no marks", fill=(150, 150, 160), font=small)
    for i, fail in enumerate(result["fails"]):
        d.text((16, y + 44 + 34 * i), fail, fill=(255, 140, 140), font=small)
    path = os.path.join(out_dir, m["_stem"] + ".marks.png")
    canvas.save(path)
    return path


def main(argv):
    if not argv or argv[0].startswith("-"):
        sys.exit(__doc__)
    d, opts, i = argv[0], {}, 1
    while i < len(argv):
        if argv[i] not in ("--expect", "--json", "--evidence") or i + 1 >= len(argv):
            sys.exit(f"unknown argument {argv[i]}\n{__doc__}")
        opts[argv[i]] = argv[i + 1]
        i += 2
    expectations = json.load(open(opts["--expect"]))["steps"] if "--expect" in opts else {}
    frames = load_frames(d)
    if not frames:
        sys.exit(f"no frames in {d}")
    unknown = set(expectations) - {m["step"]["name"] for m in frames}
    ev_dir = opts.get("--evidence")
    if ev_dir:
        os.makedirs(ev_dir, exist_ok=True)
    results, shots = [], []
    for m in frames:
        result, img, geo = grade(m, expectations.get(m["step"]["name"]))
        results.append(result)
        if ev_dir:
            shots.append(evidence(m, img, geo, result, ev_dir))
    lines = ["| step | input | app decided | painted | held to | result |", "|---|---|---|---|---|---|"]
    for r in results:
        marks = ", ".join(f"{b['word']} “{b['under']}”" for b in r["bars"]) or "—"
        app = "?" if r["app"] is None else (", ".join(f"{w['word']} “{w['text']}”" for w in r["app"]) or "—")
        verdict = "ok" if not r["fails"] else "FAIL: " + "; ".join(r["fails"])
        if r["warns"]:
            verdict += " (" + "; ".join(r["warns"]) + ")"
        held = ("app" if r["app"] is not None else "rules") + (" + spec" if r["spec"] else "")
        owner = "[Owner agent ⇥] " if r.get("owner") else ""
        lines.append(f"| {r['step']} | {owner}`{r['text']}` | {app} | {marks} | {held} | {verdict} |")
    failed = [r for r in results if r["fails"]]
    if unknown:
        lines.append(f"\nExpected steps that never ran: {', '.join(sorted(unknown))}")
    lines.append(f"\n{len(results) - len(failed)}/{len(results)} steps pass the word-mark checks.")
    report = "\n".join(lines)
    print(report)
    if "--json" in opts:
        json.dump(results, open(opts["--json"], "w"), indent=1)
    if ev_dir:
        open(os.path.join(ev_dir, "report.md"), "w").write(report + "\n")
        images = [Image.open(p) for p in shots]
        width = max(im.width for im in images)
        sheet = Image.new("RGB", (width, sum(im.height + 8 for im in images)), (10, 10, 12))
        y = 0
        for im in images:
            sheet.paste(im, (0, y))
            y += im.height + 8
        sheet.save(os.path.join(ev_dir, "contact.png"))
    sys.exit(1 if failed or unknown else 0)


if __name__ == "__main__":
    main(sys.argv[1:])
