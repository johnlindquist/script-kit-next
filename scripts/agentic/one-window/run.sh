#!/usr/bin/env bash
# One-window frames suite: one owned-evaluation build of the app, then every chapter in chapters/ and every
# stress run in stress/, each captured by scripts/agentic/one-window-frames.ts (hidden Metal windows, GPU
# readback, owned GPUI key events only) and graded:
#   check-frames.py   per frame: the input never moves, the 34 pt line above it, a 28 pt header leads every
#                     list, 44 pt rows with the focused row's fill and no bar or keycaps, the 24 pt yellow tile, zero popups, and
#                     the ↵ label never returns to an earlier one while the text only grows
#   check-marks.py    per frame: every word mark the app handed the input is painted as a bar in its
#                     word's color under exactly its text, the text keeps its own color, and no other
#                     bar shows; chapters/<name>.marks.json is the spec for what each sentence is marked
#   stress-report.py  per run: anchor, first row, header, steady height, fidelity, rug, layout shift
#   marks-video.py    per run: <out>/<name>/run.mp4 and run.gif, every step in order (not a grade)
# chapters/<name>.args holds extra one-window-frames.ts options (e.g. --fixture prompt.arg);
# chapters/<name>.expect.json and stress/<name>.expect.json hold what each step showed when its frames were
# last reviewed (header, ↵, view): a step that shows something else fails. Every run writes
# <out>/<name>/shown.json; copy it over the expect file once its frames are reviewed.
#
# usage: scripts/agentic/one-window/run.sh [out-dir]    (default .test-output/one-window/<time>)
#   ARTIFACT=<app.reference.json>  reuse a build instead of building
#   CHAPTERS="m2 p2-b"  SUITES="typing"  limit the runs (SUITES="" skips stress)
# Exit 1 when any capture or grade fails; <out>/summary.md has every table.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
cd "$root" || exit 2
out="${1:-$root/.test-output/one-window/$(date +%Y%m%d-%H%M%S)}"
mkdir -p "$out"; out="$(cd "$out" && pwd)"
export SCRIPT_KIT_NONINTERACTIVE=1

ref="${ARTIFACT:-}"
if [ -z "$ref" ]; then
  ref="$out/app.reference.json"
  : >"$out/build.log"
  for attempt in 1 2 3; do
    SCRIPT_KIT_AGENT_ARTIFACT_KIND=application \
      ./scripts/agentic/agent-cargo.sh build --locked --bin script-kit-gpui --features owned-ui-evaluation \
      >>"$out/build.log" 2>&1
    code=$?
    [ "$code" = 75 ] || break
    echo "build refused for resources (75), retry $attempt" | tee -a "$out/build.log"
    sleep 20
  done
  [ "$code" = 0 ] || { echo "build failed ($code): $out/build.log"; exit "$code"; }
  python3 - "$out/build.log" "$ref" <<'PY' || exit 3
import json, sys
text = open(sys.argv[1]).read()
for line in reversed(text.splitlines()):
    line = line.strip()
    if line.startswith("{") and ('"artifact"' in line or '"artifacts"' in line):
        doc = json.loads(line)
        ref = doc.get("artifact") or (doc.get("artifacts") or [None])[0]
        if ref:
            json.dump(ref, open(sys.argv[2], "w"))
            sys.exit(0)
start = text.rfind("\n{\n")
json.dump(json.loads(text[start + 1:text.index("\n}\n", start) + 2])["artifact"], open(sys.argv[2], "w"))
PY
fi

status=0
{ echo "# One-window frames — $(git log --oneline -1)"; echo; } >"$out/summary.md"
capture() { # name steps extra-args-file -> frames in $out/name
  local name="$1" steps="$2" args="$3" dir="$out/$1" extra=()
  rm -rf "$dir"; mkdir -p "$dir"
  [ -f "$args" ] && read -r -a extra <"$args"
  bun scripts/agentic/one-window-frames.ts run --artifact "$ref" --out "$dir/frames" --steps "$steps" \
    ${extra[@]+"${extra[@]}"} >"$dir/frames-run.log" 2>&1
  local code=$?
  python3 "$here/marks-video.py" "$dir/frames" "$dir/run" >"$dir/run-video.log" 2>&1 \
    || echo "$1: no playback video, see $dir/run-video.log"
  return "$code"
}
for steps in "$here"/chapters/*.json; do
  case "$steps" in *.expect.json|*.marks.json) continue ;; esac
  name="$(basename "$steps" .json)"
  if [ -n "${CHAPTERS:-}" ] && [[ " $CHAPTERS " != *" $name "* ]]; then continue; fi
  capture "$name" "$steps" "${steps%.json}.args"; fcode=$?
  expect=(); [ -f "${steps%.json}.expect.json" ] && expect=(--expect "${steps%.json}.expect.json")
  python3 "$here/check-frames.py" "$out/$name/frames" --json "$out/$name/check.json" --bless "$out/$name/shown.json" \
    ${expect[@]+"${expect[@]}"} >"$out/$name/check.md" 2>&1; ccode=$?
  spec=(); [ -f "${steps%.json}.marks.json" ] && spec=(--expect "${steps%.json}.marks.json")
  python3 "$here/check-marks.py" "$out/$name/frames" --json "$out/$name/marks.json" --evidence "$out/$name/marks" \
    ${spec[@]+"${spec[@]}"} >"$out/$name/marks.md" 2>&1; mcode=$?
  if [ "$name" = p3-form-edit ]; then
    python3 "$here/check-form-edit.py" "$out/$name/frames" >"$out/$name/field-pixels.json" || ccode=1
  fi
  echo "$name capture=$fcode grade=$ccode marks=$mcode"
  { echo "## $name — capture $fcode, grade $ccode, marks $mcode"; echo; cat "$out/$name/check.md"; echo
    cat "$out/$name/marks.md"; echo; } >>"$out/summary.md"
  [ "$fcode" = 0 ] && [ "$ccode" = 0 ] && [ "$mcode" = 0 ] || status=1
done
for n in ${SUITES-typing fast views}; do
  name="stress-$n"
  capture "$name" "$here/stress/$n.json" ""; fcode=$?
  expect=(); [ -f "$here/stress/$n.expect.json" ] && expect=(--expect "$here/stress/$n.expect.json")
  python3 "$here/check-frames.py" "$out/$name/frames" --json "$out/$name/check.json" --bless "$out/$name/shown.json" \
    ${expect[@]+"${expect[@]}"} >"$out/$name/check.md" 2>&1; ccode=$?
  python3 "$here/stress-report.py" "$out/$name/frames" --json "$out/$name/stress.json" >"$out/$name/stress.md" 2>&1; scode=$?
  echo "$name capture=$fcode grade=$ccode stress=$scode"
  { echo "## $name — capture $fcode, grade $ccode, stress $scode"; echo; cat "$out/$name/check.md"; echo; cat "$out/$name/stress.md"; echo; } >>"$out/summary.md"
  [ "$fcode" = 0 ] && [ "$ccode" = 0 ] && [ "$scode" = 0 ] || status=1
done
echo "$out/summary.md"
exit "$status"
