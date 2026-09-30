#!/usr/bin/env python3
"""stress-report.py <frames dir> [--json out.json]

The prototype's stability harness read from owned-evaluator frames of the Rust app.

Hard, per run:
  anchor      the input's y never changes
  first row   whenever rows show, the first header and the first row sit at one y
  header      rows never show without a header above them
  steady      the window keeps one height: every frame is as tall as the first, whatever was typed or arrived
  fidelity    the window is the height the host keeps (state.oneWindow.heightPx, ±1 px) in every frame
  rug         while the text only grows, the ↵ label never returns to an earlier label (A → B → A)
  shift       layout shift score: moved px of the input and first row over window height, summed (must be 0)
Reported:
  ↵ changes with no key (a wait-only step), slot line text per step, window heights.
"""
import json, os, sys
from importlib.machinery import SourceFileLoader

HERE = os.path.dirname(os.path.abspath(__file__))
enter_label = SourceFileLoader("check_frames", os.path.join(HERE, "check-frames.py")).load_module().enter_label


def load(d):
    steps = []
    for f in sorted(os.listdir(d)):
        if f.endswith(".json") and f[:2].isdigit() and f.count(".") == 1:
            m = json.load(open(os.path.join(d, f)))
            cap = os.path.join(d, f[:-5] + ".capture.json")
            m["_ow"] = (json.load(open(cap)).get("state") or {}).get("oneWindow") or {} if os.path.exists(cap) else {}
            steps.append(m)
    return steps


def main():
    d = sys.argv[1]
    out_json = sys.argv[sys.argv.index("--json") + 1] if "--json" in sys.argv else None
    steps = load(d)
    if not steps:
        print("no steps in", d)
        sys.exit(1)
    y0 = steps[0]["input"]["y"]
    head0 = row0 = None
    fails, unprompted, table = [], [], []
    shift = 0.0
    labels, prev_text, prev = [], "", None
    for m in steps:
        st = m["step"]
        name = st["name"]
        typed = "type" in st or "append" in st
        waited_only = not (typed or "key" in st)
        iy = m["input"]["y"]
        if iy != y0:
            fails.append(f"{name}: input moved {iy - y0:+g}")
        fh, fr = m.get("firstHeader"), m.get("firstRow")
        if fr and (not fh or fh["y"] > fr["y"]):
            fails.append(f"{name}: rows without a header")
        if fh and fr:
            if head0 is None:
                head0, row0 = fh["y"], fr["y"]
            else:
                if fh["y"] != head0:
                    fails.append(f"{name}: header moved {fh['y'] - head0:+g}")
                if fr["y"] != row0:
                    fails.append(f"{name}: first row moved {fr['y'] - row0:+g}")
        h = m.get("windowHeight") or 0
        label = enter_label(m)
        placed = m["_ow"].get("heightPx")
        if placed is not None and h and abs(h - placed) > 1:
            fails.append(f"{name}: window {h} px but the host keeps {placed:g}")
        if prev is not None:
            ph = prev.get("windowHeight") or 0
            if h and ph and h != ph:
                fails.append(f"{name}: window height changed {ph}→{h}")
            if h:
                pfr = prev.get("firstRow")
                moved = abs(iy - prev["input"]["y"])
                if pfr and fr:
                    moved += abs(fr["y"] - pfr["y"])
                shift += moved / h
            if waited_only and label != enter_label(prev):
                unprompted.append(f"{name}: “{enter_label(prev)}” → “{label}”")
        text = m.get("inputValue") or ""
        if prev_text and text.startswith(prev_text) and len(text) > len(prev_text):
            if label in [l for l in labels[:-1] if l != labels[-1]]:
                fails.append(f"{name}: ↵ returned to “{label}”")
        elif not text.startswith(prev_text):
            labels = []
        if not labels or labels[-1] != label:
            labels.append(label)
        prev_text = text
        slot = next((a for a in m.get("aboveInputAll") or [] if a["name"].startswith("main-view-slot-line")), None)
        table.append({"step": name, "text": text, "enter": label, "windowH": h, "placed": placed,
                      "slot": slot and slot.get("text"), "header": fh and fh.get("text"),
                      "firstRowY": fr and fr["y"]})
        prev = m
    if shift > 0:
        fails.append(f"layout shift {shift:.4f}")
    print("| step | text | ↵ | window h | kept | header | first row y |")
    print("|---|---|---|---|---|---|---|")
    for r in table:
        first = r["firstRowY"] if r["firstRowY"] is not None else "—"
        placed = f"{r['placed']:g}" if r["placed"] is not None else "—"
        print(f"| {r['step']} | {r['text'][:36]} | {(r['enter'] or '—')[:36]} | {r['windowH']} | {placed} | "
              f"{(r['header'] or '—')[:28]} | {first} |")
    print()
    print(f"anchor · first row · header · steady · fidelity · rug · shift: {'PASS' if not fails else 'FAIL'}")
    for f in fails:
        print("  FAIL", f)
    for u in unprompted:
        print("  note: ↵ changed with no key,", u)
    if out_json:
        json.dump({"pass": not fails, "fails": fails, "shift": shift, "unprompted": unprompted, "steps": table},
                  open(out_json, "w"), indent=1, ensure_ascii=False)
    sys.exit(1 if fails else 0)


if __name__ == "__main__":
    main()
