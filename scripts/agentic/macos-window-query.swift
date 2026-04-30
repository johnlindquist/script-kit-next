#!/usr/bin/env swift
//
// scripts/agentic/macos-window-query.swift
//
// Enumerate on-disk windows for a given process-owner name (default:
// "script-kit-gpui") using CGWindowListCopyWindowInfo. Unlike
// `python3 -c "import Quartz"` (PyObjC is NOT installed in the system
// python on this machine — see Pass #6 Run 2 audit) and unlike JXA's
// ObjC bridge (CFArray returned from CGWindowListCopyWindowInfo does NOT
// bridge to NSArray in JXA — count/objectAtIndex are undefined), Swift
// accesses CoreGraphics directly and returns correct results.
//
// The agentic window.ts tool shells out to this script for NonactivatingPanel
// visibility: Script Kit GPUI's main window uses
//   setActivationPolicy(.accessory) + WindowKind::PopUp (NSPanel w/
//   NonactivatingPanel style bit) + NSFloatingWindowLevel, which is
// invisible to CGWindowListOptionOnScreenOnly but appears under
// .optionAll.
//
// Output: JSON on stdout describing matched windows. Diagnostics to stderr
// only on parse/runtime errors (the happy path is silent on stderr so the
// caller can parse stdout cleanly).
//
// Usage:
//   swift scripts/agentic/macos-window-query.swift                 # owner=script-kit-gpui (default)
//   swift scripts/agentic/macos-window-query.swift --owner NAME    # override owner filter (substring, case-insensitive)
//   swift scripts/agentic/macos-window-query.swift --pid 12345     # restrict to a specific PID
//
// Exit 0 always; the JSON envelope carries the result and errors.

import Foundation
import CoreGraphics

let SCHEMA_VERSION = 1

// -----------------------------------------------------------------------------
// Argument parsing
// -----------------------------------------------------------------------------

var ownerFilter: String = "script-kit-gpui"
var pidFilter: Int? = nil

let args = Array(CommandLine.arguments.dropFirst())
var i = 0
while i < args.count {
    let a = args[i]
    switch a {
    case "--owner":
        if i + 1 < args.count { ownerFilter = args[i + 1]; i += 1 }
    case "--pid":
        if i + 1 < args.count, let p = Int(args[i + 1]) { pidFilter = p; i += 1 }
    case "--help", "-h":
        print("""
        Usage: swift macos-window-query.swift [--owner SUBSTR] [--pid PID]
        Output: JSON {schemaVersion, windows: [...]}
        """)
        exit(0)
    default:
        break
    }
    i += 1
}

// -----------------------------------------------------------------------------
// Enumerate all windows (incl. NonactivatingPanels)
// -----------------------------------------------------------------------------

let ownerNeedle = ownerFilter.lowercased()
let options: CGWindowListOption = [.optionAll]

guard let raw = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]] else {
    let envelope: [String: Any] = [
        "schemaVersion": SCHEMA_VERSION,
        "status": "error",
        "error": ["code": "CGWINDOW_LIST_FAILED", "message": "CGWindowListCopyWindowInfo returned nil"],
    ]
    if let data = try? JSONSerialization.data(withJSONObject: envelope, options: [.prettyPrinted]) {
        print(String(data: data, encoding: .utf8)!)
    }
    exit(0)
}

var windows: [[String: Any]] = []
for w in raw {
    let owner = (w[kCGWindowOwnerName as String] as? String) ?? ""
    if !owner.lowercased().contains(ownerNeedle) { continue }
    let pid = (w[kCGWindowOwnerPID as String] as? Int) ?? 0
    if let filter = pidFilter, filter != pid { continue }

    let wid = (w[kCGWindowNumber as String] as? Int) ?? 0
    let name = (w[kCGWindowName as String] as? String) ?? ""
    let layer = (w[kCGWindowLayer as String] as? Int) ?? 0
    let alpha = (w[kCGWindowAlpha as String] as? Double) ?? 0
    let onscreenInt = (w[kCGWindowIsOnscreen as String] as? Int) ?? 0
    let onscreen = onscreenInt == 1
    let store = (w[kCGWindowStoreType as String] as? Int) ?? 0

    var bounds: [String: Int] = ["x": 0, "y": 0, "width": 0, "height": 0]
    if let b = w[kCGWindowBounds as String] as? [String: Any] {
        bounds["x"] = (b["X"] as? Int) ?? Int((b["X"] as? Double) ?? 0)
        bounds["y"] = (b["Y"] as? Int) ?? Int((b["Y"] as? Double) ?? 0)
        bounds["width"] = (b["Width"] as? Int) ?? Int((b["Width"] as? Double) ?? 0)
        bounds["height"] = (b["Height"] as? Int) ?? Int((b["Height"] as? Double) ?? 0)
    }

    windows.append([
        "windowId": wid,
        "ownerName": owner,
        "ownerPid": pid,
        "title": name,
        "layer": layer,
        "alpha": alpha,
        "onscreen": onscreen,
        "storeType": store,
        "bounds": bounds,
    ])
}

let envelope: [String: Any] = [
    "schemaVersion": SCHEMA_VERSION,
    "status": "ok",
    "ownerFilter": ownerFilter,
    "pidFilter": pidFilter as Any,
    "windows": windows,
]

do {
    let data = try JSONSerialization.data(withJSONObject: envelope, options: [.sortedKeys])
    if let s = String(data: data, encoding: .utf8) { print(s) }
} catch {
    FileHandle.standardError.write("json serialization failed: \(error)\n".data(using: .utf8)!)
    exit(1)
}
