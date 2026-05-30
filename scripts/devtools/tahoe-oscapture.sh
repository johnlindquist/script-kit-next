#!/usr/bin/env bash
#
# tahoe-oscapture.sh — Screen-rect OS compositor capture of a Script Kit window
# region, producing real osScreenshotProof evidence for the Liquid Glass proof
# matrix (artifacts/liquid-glass/receipts/liquid-glass-proof-matrix.json).
#
# WHY THIS EXISTS
#   Window-id capture of the non-activating NSPanel is dead on modern macOS
#   (CGWindowListCreateImage -> "could not create image from window"), and
#   CGDisplayCreateImage is obsoleted in macOS 15. The working path is to wake
#   the display, capture the FULL display it sits on (real compositor output),
#   and crop to the window rect. captureKind == "screen-rect".
#
# HARD ENVIRONMENTAL REQUIREMENT
#   The login session must be UNLOCKED and the display awake. While the screen
#   is locked (CGSSessionScreenIsLocked == 1) the user session's windows are NOT
#   composited to the physical display, so no app-content capture is possible —
#   identical for ScreenCaptureKit. This script DETECTS that and writes a
#   `blocked` receipt instead of fabricating proof. Run it from an interactive,
#   unlocked session.
#
# USAGE
#   scripts/devtools/tahoe-oscapture.sh <term> <SurfaceKind> [ownerSubstr]
#     term         filename/evidence term proof.ts matches (e.g. clipboard, about)
#     SurfaceKind  contract surface kind (for the receipt; informational)
#     ownerSubstr  CGWindow owner name substring (default: script-kit-gpui)
#
# OUTPUT
#   artifacts/liquid-glass/screenshots/lg-oscap-<term>.png
#   artifacts/liquid-glass/receipts/lg-oscap-<term>-screenshot.json
#   Exit 0 captured, 2 blocked (locked / no window / black), 1 usage/error.

set -uo pipefail

TERM_ARG="${1:-}"
SURFACE="${2:-}"
OWNER="${3:-script-kit-gpui}"
if [[ -z "$TERM_ARG" || -z "$SURFACE" ]]; then
  echo "usage: tahoe-oscapture.sh <term> <SurfaceKind> [ownerSubstr]" >&2
  exit 1
fi

ROOT="artifacts/liquid-glass"
GEO="scripts/devtools/bin/tahoe_window_geometry"
PNG="$ROOT/screenshots/lg-oscap-${TERM_ARG}.png"
RECEIPT="$ROOT/receipts/lg-oscap-${TERM_ARG}-screenshot.json"
mkdir -p "$ROOT/screenshots" "$ROOT/receipts"

if [[ ! -x "$GEO" ]]; then
  echo "geometry helper missing; build with: swiftc -O scripts/devtools/tahoe_window_geometry.swift -o $GEO" >&2
  exit 1
fi

# Hold display awake for the duration of this capture.
caffeinate -u -t 30 &
CAFF=$!
trap 'kill "$CAFF" 2>/dev/null || true' EXIT
sleep 0.6

GEO_JSON="$($GEO --owner "$OWNER" 2>/dev/null || true)"

read_field() { printf '%s' "$GEO_JSON" | python3 -c "import sys,json;d=json.load(sys.stdin);print(d$1)" 2>/dev/null; }

write_blocked() {
  local reason="$1" nonblack="${2:-null}"
  python3 - "$RECEIPT" "$TERM_ARG" "$SURFACE" "$reason" "$nonblack" "$GEO_JSON" <<'PY'
import json,sys
path,term,surface,reason,nonblack,geo=sys.argv[1:7]
try: geoj=json.loads(geo) if geo.strip() else {}
except Exception: geoj={}
json.dump({
  "schemaVersion":1,"label":f"lg-oscap-{term}","surface":surface,
  "status":"error","classification":"macos-screen-locked" if reason=="screen-locked" else "screenshot-capture-failed",
  "captureKind":"screen-rect",
  "visualEvidence":{
    "source":"os-window-capture","available":False,"classification":reason,
    "captureKind":"screen-rect","countsAsOsScreenshotEvidence":False,
    "countsAsCompositorEvidence":False,"blockerCode":reason,
    "limitation":"login session locked or display asleep; app windows are not composited to the physical display",
    "geometry":geoj,
    "attempts":[{"method":"screencapture-display-crop","status":"failed","reason":reason,
                 "nonBlackRatio":(float(nonblack) if nonblack not in ('null','') else None)}],
  },
  "screenshotReceipt":{"captured":False,"path":None,"error":reason,
    "contentAudit":{"nonBlackRatio":(float(nonblack) if nonblack not in ('null','') else None),"blank":True}},
}, open(path,"w"), indent=2)
print("BLOCKED",reason)
PY
}

OK_FIELD="$(read_field "['ok']" || true)"
LOCKED="$(read_field "['screenLocked']" || true)"

if [[ "$LOCKED" == "True" ]]; then
  write_blocked "screen-locked"
  echo "screen LOCKED (CGSSessionScreenIsLocked=1): wrote blocked receipt $RECEIPT" >&2
  exit 2
fi
if [[ "$OK_FIELD" != "True" ]]; then
  write_blocked "window-not-found"
  echo "no on-screen window for owner '$OWNER': wrote blocked receipt $RECEIPT" >&2
  exit 2
fi

CX="$(read_field "['cropPixels']['x']")"
CY="$(read_field "['cropPixels']['y']")"
CW="$(read_field "['cropPixels']['w']")"
CH="$(read_field "['cropPixels']['h']")"
WID="$(read_field "['windowId']")"
DID="$(read_field "['displayId']")"
TBW="$(read_field "['displayBackingPixels']['w']")"
TBH="$(read_field "['displayBackingPixels']['h']")"

# Capture EVERY display, then pick the image whose backing pixel size matches the
# target display. This avoids screencapture's -D index (which does not reliably
# match CGGetActiveDisplayList order) and works for retina + non-retina mixes.
TMPDIR_CAP="$(mktemp -d -t tahoe-disp)"
trap 'kill "$CAFF" 2>/dev/null || true; rm -rf "$TMPDIR_CAP"' EXIT
# Up to 8 displays; screencapture writes one file per display, extra args ignored.
screencapture -x -t png \
  "$TMPDIR_CAP/d1.png" "$TMPDIR_CAP/d2.png" "$TMPDIR_CAP/d3.png" "$TMPDIR_CAP/d4.png" \
  "$TMPDIR_CAP/d5.png" "$TMPDIR_CAP/d6.png" "$TMPDIR_CAP/d7.png" "$TMPDIR_CAP/d8.png" 2>/dev/null

FULL=""
for f in "$TMPDIR_CAP"/d*.png; do
  [[ -s "$f" ]] || continue
  fw="$(sips -g pixelWidth "$f" 2>/dev/null | awk '/pixelWidth/{print $2}')"
  fh="$(sips -g pixelHeight "$f" 2>/dev/null | awk '/pixelHeight/{print $2}')"
  if [[ "$fw" == "$TBW" && "$fh" == "$TBH" ]]; then FULL="$f"; break; fi
done
if [[ -z "$FULL" ]]; then write_blocked "display-match-failed"; echo "no captured display matched backing ${TBW}x${TBH}" >&2; exit 2; fi

cp "$FULL" "$PNG"
sips -c "$CH" "$CW" --cropOffset "$CY" "$CX" "$PNG" >/dev/null 2>&1 || { write_blocked "crop-failed"; echo "crop failed" >&2; exit 2; }

NONBLACK="$(python3 - "$PNG" <<'PY'
import sys
from PIL import Image
im=Image.open(sys.argv[1]).convert("RGB"); px=list(im.getdata()); n=len(px)
nb=sum(1 for r,g,b in px if r>10 or g>10 or b>10)
print(f"{nb/n:.4f}")
PY
)"

# A lock screen / blank desktop is mostly uniform; require real content.
awk "BEGIN{exit !($NONBLACK < 0.01)}" && { write_blocked "low-content-capture" "$NONBLACK"; rm -f "$PNG"; echo "nonBlackRatio $NONBLACK < 0.01 -> blocked" >&2; exit 2; }

W="$(sips -g pixelWidth "$PNG" 2>/dev/null | awk '/pixelWidth/{print $2}')"
H="$(sips -g pixelHeight "$PNG" 2>/dev/null | awk '/pixelHeight/{print $2}')"

python3 - "$RECEIPT" "$TERM_ARG" "$SURFACE" "$PNG" "$WID" "$DID" "$W" "$H" "$NONBLACK" "$GEO_JSON" <<'PY'
import json,sys
path,term,surface,png,wid,did,w,h,nb,geo=sys.argv[1:11]
geoj=json.loads(geo)
json.dump({
  "schemaVersion":1,"label":f"lg-oscap-{term}","surface":surface,"status":"ok",
  "classification":"captured","captureKind":"screen-rect",
  "visualEvidence":{
    "source":"os-window-capture","available":True,"classification":"captured",
    "captureKind":"screen-rect","countsAsOsScreenshotEvidence":True,
    "countsAsCompositorEvidence":True,"blockerCode":"none",
    "screenshotPath":png,"windowId":int(wid),"displayId":int(did),
    "winRect":geoj.get("winRect"),"displayBackingPixels":geoj.get("displayBackingPixels"),
    "displayBounds":geoj.get("displayBounds"),"cropPixels":geoj.get("cropPixels"),
    "attempts":[{"method":"screencapture-alldisplays-match-crop","status":"captured",
                 "displayId":int(did),"nonBlackRatio":float(nb)}],
  },
  "screenshotReceipt":{"captured":True,"path":png,"width":int(w),"height":int(h),
    "captureMethod":"tahoe-oscapture.sh","windowCaptureMethod":"screencapture-display-crop",
    "contentAudit":{"nonBlackRatio":float(nb),"blank":False}},
}, open(path,"w"), indent=2)
print("CAPTURED",png,"nonBlackRatio",nb)
PY
echo "captured $PNG ($W x $H, nonBlackRatio=$NONBLACK) -> $RECEIPT"
exit 0
