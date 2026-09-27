#!/usr/bin/env python3
"""marks-video.py <frames dir> <out stem> [--hold MS] [--zoom Z]

Turns a one-window-frames.ts run into a video that plays each step as it was captured: the whole
window above, a close-up of the input row under it (so the word marks read at a glance), and the
step's name with what was typed. Writes <out stem>.mp4 and <out stem>.gif; typed-word steps hold
for their waitMs (at least --hold, default 700 ms), every other step for 1.6 s.
"""
import json, os, subprocess, sys, tempfile

from PIL import Image, ImageDraw, ImageFont

WIDTH = 1200
BG = (18, 19, 23)


def font(size):
    for path in ("/System/Library/Fonts/SFNS.ttf", "/System/Library/Fonts/Helvetica.ttc"):
        if os.path.exists(path):
            return ImageFont.truetype(path, size)
    return ImageFont.load_default()


def frames(d):
    out = []
    for name in sorted(os.listdir(d)):
        if not name.endswith(".json") or name.endswith(".capture.json"):
            continue
        m = json.load(open(os.path.join(d, name)))
        png = os.path.join(d, name[:-5] + ".png")
        if "step" in m and os.path.exists(png):
            out.append((m, png))
    return out


def card(m, png, zoom):
    img = Image.open(png).convert("RGB")
    s = m.get("scale", 2)
    full = img.resize((WIDTH - 80, round(img.height * (WIDTH - 80) / img.width)), Image.LANCZOS)
    inp = m.get("input")
    close = None
    if inp:
        pad = 6
        box = (0, int((inp["y"] - pad) * s), img.width, int((inp["y"] + inp["h"] + pad) * s))
        row = img.crop(box)
        w = WIDTH - 80
        h = round(row.height * w / row.width)
        close = row.resize((w, max(h, 1)), Image.LANCZOS)
        if zoom > 1:
            cw = round(row.width / zoom * 1.25)
            left = max(0, int(inp["x"] * s) - 10)
            row = img.crop((left, box[1], min(img.width, left + cw), box[3]))
            close = row.resize((w, round(row.height * w / row.width)), Image.LANCZOS)
    head = 64
    height = head + full.height + 24 + (close.height + 24 if close else 0) + 24
    out = Image.new("RGB", (WIDTH, height), BG)
    d = ImageDraw.Draw(out)
    step = m["step"]
    d.text((40, 16), step.get("name", ""), fill=(236, 236, 240), font=font(22))
    typed = m.get("inputValue", "")
    d.text((40, 42), f"input: {typed!r}" if typed else "input: (empty)", fill=(150, 152, 160), font=font(15))
    out.paste(full, (40, head))
    if close:
        y = head + full.height + 24
        out.paste(close, (40, y))
        d.rectangle((39, y - 1, 40 + close.width, y + close.height), outline=(70, 72, 80))
    return out


def main(argv):
    if len(argv) < 2:
        sys.exit(__doc__)
    d, stem = argv[0], argv[1]
    hold, zoom = 700, 2.5
    if "--hold" in argv:
        hold = int(argv[argv.index("--hold") + 1])
    if "--zoom" in argv:
        zoom = float(argv[argv.index("--zoom") + 1])
    shots = frames(d)
    if not shots:
        sys.exit(f"no frames in {d}")
    tmp = tempfile.mkdtemp(prefix="marks-video-")
    cards = [card(m, png, zoom) for m, png in shots]
    height = max(c.height for c in cards)
    height += height % 2
    lines = []
    for i, ((m, _), c) in enumerate(zip(shots, cards)):
        frame = Image.new("RGB", (WIDTH, height), BG)
        frame.paste(c, (0, 0))
        path = os.path.join(tmp, f"{i:04}.png")
        frame.save(path)
        step = m["step"]
        ms = max(hold, step.get("waitMs", 0)) if step.get("append") or step.get("type") else 1600
        lines += [f"file '{path}'", f"duration {ms / 1000:.3f}"]
    lines.append(f"file '{os.path.join(tmp, f'{len(cards) - 1:04}.png')}'")
    listing = os.path.join(tmp, "list.txt")
    open(listing, "w").write("\n".join(lines) + "\n")
    run = lambda *a: subprocess.run(["ffmpeg", "-y", "-loglevel", "error", *a], check=True)
    run("-f", "concat", "-safe", "0", "-i", listing, "-vf", "fps=30,format=yuv420p",
        "-c:v", "libx264", "-crf", "18", "-movflags", "+faststart", stem + ".mp4")
    run("-i", stem + ".mp4", "-vf",
        "fps=10,scale=900:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=128[p];[b][p]paletteuse",
        stem + ".gif")
    for name in os.listdir(tmp):
        os.remove(os.path.join(tmp, name))
    os.rmdir(tmp)
    print(stem + ".mp4")


if __name__ == "__main__":
    main(sys.argv[1:])
