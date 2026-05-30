// tahoe_window_geometry.swift
//
// Reports the screen geometry needed to crop a single window's region out of a
// full-display `screencapture`, plus the session lock state. We cannot use
// CGDisplayCreateImage (obsoleted in macOS 15) nor window-id capture
// (CGWindowListCreateImage -> "could not create image from window" for the
// non-activating NSPanel). The working path is:
//
//   caffeinate -u (wake display)  ->  screencapture -D<n> (full display, real
//   compositor output)  ->  sips crop to the window rect.
//
// HARD ENVIRONMENTAL REQUIREMENT: the login session must be UNLOCKED and the
// display awake. While CGSSessionScreenIsLocked == 1, the user session's
// windows are NOT composited to the physical display (screencapture only sees
// the lock screen), so no compositor capture of app content is possible — this
// is identical for ScreenCaptureKit. This helper surfaces `screenLocked` so the
// capture wrapper refuses to emit fake OS-screenshot proof.
//
// Usage:
//   tahoe_window_geometry <windowId>
//   tahoe_window_geometry --owner <ownerNameSubstr>   (picks largest matching on-screen window)
// Output: one JSON object on stdout. Exit non-zero with {"ok":false,...} on error.

import Foundation
import CoreGraphics

func emit(_ obj: [String: Any]) {
    let data = try! JSONSerialization.data(withJSONObject: obj, options: [.sortedKeys])
    FileHandle.standardOutput.write(data)
    FileHandle.standardOutput.write("\n".data(using: .utf8)!)
}

func sessionLocked() -> Bool {
    guard let d = CGSessionCopyCurrentDictionary() as? [String: Any] else { return false }
    if let v = d["CGSSessionScreenIsLocked"] as? Int { return v != 0 }
    return false
}

func fail(_ message: String) -> Never {
    emit(["ok": false, "error": message, "screenLocked": sessionLocked()])
    exit(1)
}

let args = Array(CommandLine.arguments.dropFirst())
guard !args.isEmpty else { fail("usage: tahoe_window_geometry <windowId> | --owner <substr>") }

let onScreen = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as? [[String: Any]] ?? []

func bounds(_ w: [String: Any]) -> CGRect {
    let b = w[kCGWindowBounds as String] as? [String: CGFloat] ?? [:]
    return CGRect(x: b["X"] ?? 0, y: b["Y"] ?? 0, width: b["Width"] ?? 0, height: b["Height"] ?? 0)
}

var chosen: [String: Any]?
if args[0] == "--owner", args.count >= 2 {
    let needle = args[1].lowercased()
    chosen = onScreen
        .filter { (($0[kCGWindowOwnerName as String] as? String) ?? "").lowercased().contains(needle) }
        .max(by: { bounds($0).width * bounds($0).height < bounds($1).width * bounds($1).height })
    if chosen == nil { fail("no on-screen window owned by '\(args[1])' (screenLocked=\(sessionLocked()))") }
} else if let widRaw = UInt32(args[0]) {
    let all = CGWindowListCopyWindowInfo([.optionIncludingWindow], CGWindowID(widRaw)) as? [[String: Any]] ?? []
    chosen = all.first(where: { ($0[kCGWindowNumber as String] as? Int).map { UInt32($0) } == widRaw }) ?? all.first
    if chosen == nil { fail("window \(widRaw) not found") }
} else {
    fail("invalid argument: \(args[0])")
}

let win = chosen!
let widResolved = win[kCGWindowNumber as String] as? Int ?? -1
let winRect = bounds(win)
if winRect.width < 2 || winRect.height < 2 { fail("window \(widResolved) degenerate bounds \(winRect)") }

// Active display list (screencapture -D is 1-based into this order).
var dispCount: UInt32 = 0
CGGetActiveDisplayList(0, nil, &dispCount)
var disps = [CGDirectDisplayID](repeating: 0, count: Int(dispCount))
CGGetActiveDisplayList(dispCount, &disps, &dispCount)

// Select the display with the largest overlap with the window rect. This is
// robust when the window straddles a display edge or sits in a gap between
// non-aligned displays (where contains(center) can mis-select the main display).
func overlapArea(_ a: CGRect, _ b: CGRect) -> CGFloat {
    let r = a.intersection(b)
    return r.isNull ? 0 : r.width * r.height
}
var targetIndex = 0
var bestOverlap: CGFloat = -1
for (i, d) in disps.enumerated() {
    let o = overlapArea(winRect, CGDisplayBounds(d))
    if o > bestOverlap { bestOverlap = o; targetIndex = i }
}
let target = disps.isEmpty ? CGMainDisplayID() : disps[targetIndex]
let dispBounds = CGDisplayBounds(target)

// True BACKING pixel dimensions come from the display mode (pixelWidth/Height),
// not CGDisplayPixelsWide which returns the scaled *point* resolution on retina
// displays. screencapture writes images at backing resolution, so the crop must
// be computed at backing scale.
let mode = CGDisplayCopyDisplayMode(target)
let backingW = mode?.pixelWidth ?? CGDisplayPixelsWide(target)
let backingH = mode?.pixelHeight ?? CGDisplayPixelsHigh(target)
let scaleX = Double(backingW) / Double(dispBounds.width)
let scaleY = Double(backingH) / Double(dispBounds.height)

emit([
    "ok": true,
    "screenLocked": sessionLocked(),
    "windowId": widResolved,
    "ownerName": win[kCGWindowOwnerName as String] as? String ?? "",
    "windowAlpha": win[kCGWindowAlpha as String] as? Double ?? -1,
    "winRect": ["x": winRect.minX, "y": winRect.minY, "w": winRect.width, "h": winRect.height],
    "displayId": Int(target),
    "displayIndex1Based": targetIndex + 1,
    "isMain": CGDisplayIsMain(target) != 0,
    "displayBounds": ["x": dispBounds.minX, "y": dispBounds.minY, "w": dispBounds.width, "h": dispBounds.height],
    // BACKING pixel dimensions — used by the wrapper to identify which captured
    // display image is this one (all displays here have distinct sizes).
    "displayBackingPixels": ["w": backingW, "h": backingH],
    "scaleX": scaleX,
    "scaleY": scaleY,
    // crop rect in display-local BACKING pixels for `sips --cropOffset Y X` + `-c H W`
    "cropPixels": [
        "x": Int(((winRect.minX - dispBounds.minX) * scaleX).rounded()),
        "y": Int(((winRect.minY - dispBounds.minY) * scaleY).rounded()),
        "w": Int((winRect.width * scaleX).rounded()),
        "h": Int((winRect.height * scaleY).rounded()),
    ],
])
